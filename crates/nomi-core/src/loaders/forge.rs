use std::{
    collections::HashMap,
    fmt::Debug,
    fs::File,
    io::{BufRead, Read},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail};
use itertools::Itertools;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, process::Command};
use tracing::{error, info, warn};
use zip::read::ZipFile;

use crate::{
    configs::profile::Loader,
    downloads::{
        download_file,
        progress::ProgressSender,
        traits::{DownloadResult, Downloader},
        DownloadQueue, FileDownloader, LibrariesDownloader, LibrariesMapper,
    },
    game_paths::GamePaths,
    instance::{launch::CLASSPATH_SEPARATOR, loader::LoaderProfile},
    loaders::vanilla::VanillaLibrariesMapper,
    maven_data::{MavenArtifact, MavenData},
    repository::{
        java_runner::JavaRunner,
        manifest::{Argument, Arguments, Library},
        simple_args::SimpleArgs,
        simple_lib::SimpleLib,
    },
    PinnedFutureWithBounds, DOT_NOMI_TEMP_DIR,
};

use super::ToLoaderProfile;

const FORGE_REPO_URL: &str = "https://maven.minecraftforge.net";

const _NEO_FORGE_REPO_URL: &str = "https://maven.neoforged.net/releases/";

/// Some versions require to have a suffix
const FORGE_SUFFIXES: &[(&str, &[&str])] = &[
    ("1.11", &["-1.11.x"]),
    ("1.10.2", &["-1.10.0"]),
    ("1.10", &["-1.10.0"]),
    ("1.9.4", &["-1.9.4"]),
    ("1.9", &["-1.9.0", "-1.9"]),
    ("1.8.9", &["-1.8.9"]),
    ("1.8.8", &["-1.8.8"]),
    ("1.8", &["-1.8"]),
    ("1.7.10", &["-1.7.10", "-1710ls", "-new"]),
    ("1.7.2", &["-mc172"]),
];

#[derive(Debug)]
pub struct Forge {
    profile: ForgeProfile,
    downloader: DownloadQueue,
    game_version: String,
    forge_version: String,
    game_paths: GamePaths,

    library_data: Option<ForgeLibraryExtractionData>,
    processors_data: Option<ProcessorsData>,
}

impl Forge {
    #[tracing::instrument(skip_all, err)]
    pub async fn get_versions(game_version: impl Into<String>) -> anyhow::Result<Vec<String>> {
        let game_version = game_version.into();

        let raw = reqwest::get(format!("{FORGE_REPO_URL}/net/minecraftforge/forge/maven-metadata.xml"))
            .await?
            .text()
            .await?;

        // Parsing the XML to get versions list
        let versions = raw
            .find("<version>")
            .and_then(|s| raw.find("</versions>").map(|e| (s, e)))
            .map(|(s, e)| &raw[s..e])
            .map(|s| {
                s.split("<version>")
                    .filter_map(|s| {
                        s.split("</version>")
                            .collect::<String>()
                            .split('-')
                            .map(String::from)
                            .collect_tuple::<(String, String)>()
                    })
                    .filter(|(g, _)| g == &game_version)
                    .map(|(_, f)| f)
                    .collect::<Vec<_>>()
            });

        match versions {
            Some(v) => Ok(v),
            None => Err(anyhow!("Error while matching forge versions")),
        }
    }

    /// Get forge versions that are recommended for specific game version
    #[tracing::instrument(err)]
    pub async fn get_promo_versions() -> anyhow::Result<ForgeVersions> {
        reqwest::get("https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json")
            .await?
            .json::<ForgeVersions>()
            .await
            .map_err(Into::into)
    }

    async fn proceed_version(game_version: &str, forge_version: ForgeVersion) -> Option<String> {
        let promo_versions = Self::get_promo_versions().await.ok()?;

        let from_promo = |version| {
            // Sometime one of those does not exist so we have a fallback.
            let next_version = match version {
                ForgeVersion::Recommended => ForgeVersion::Latest,
                ForgeVersion::Latest => ForgeVersion::Recommended,
                ForgeVersion::Specific(_) => unreachable!(),
            };

            promo_versions
                .promos
                .get(&version.format(game_version))
                .or_else(|| promo_versions.promos.get(&next_version.format(game_version)))
                .cloned()
        };

        match forge_version {
            ForgeVersion::Specific(v) => Some(v),
            version => from_promo(version),
        }
    }

    /// Get list of urls that we should try to get installer from
    fn get_urls(game_version: &str, forge_version: &str) -> Vec<String> {
        let mut suffixes = vec![""];

        if let Some((_, s)) = FORGE_SUFFIXES.iter().find(|(k, _)| k == &game_version) {
            suffixes.extend(s.iter());
        }

        suffixes.into_iter().map(|suffix| {
            format!(
                "{FORGE_REPO_URL}/net/minecraftforge/forge/{game_version}-{forge_version}{suffix}/forge-{game_version}-{forge_version}{suffix}-installer.jar",
            )
        }).collect_vec()
    }

    fn get_profile_from_installer(archive: &mut zip::ZipArchive<File>) -> anyhow::Result<ForgeProfile> {
        let index = archive
            .index_for_name("version.json")
            .or_else(|| archive.index_for_name("install_profile.json"));

        let Some(idx) = index else {
            bail!("Cannot find either `version.json` or `install_profile.json`")
        };

        let mut file = archive.by_index(idx)?;

        read_json_from_zip(&mut file)
    }

    fn get_install_profile(archive: &mut zip::ZipArchive<File>) -> anyhow::Result<ForgeInstallProfile> {
        let mut file = archive.by_name("install_profile.json")?;

        read_json_from_zip(&mut file)
    }

    pub fn installer_path(&self) -> PathBuf {
        forge_installer_path(&self.game_version, &self.forge_version)
    }

    pub fn binpatch_path(&self) -> PathBuf {
        forge_binpatch_path(&self.game_version, &self.forge_version)
    }

    #[tracing::instrument(skip(version), fields(game_version) err)]
    pub async fn new(version: impl Into<String>, forge_version: ForgeVersion, game_paths: GamePaths) -> anyhow::Result<Self> {
        let game_version: String = version.into();

        tracing::Span::current().record("game_version", &game_version);

        let Some(forge_version) = Self::proceed_version(&game_version, forge_version).await else {
            bail!("Cannot match version");
        };

        let installer_path = forge_installer_path(&game_version, &forge_version);

        let urls = Self::get_urls(&game_version, &forge_version);
        for url in &urls {
            if let Err(err) = download_file(&installer_path, url).await {
                warn!(
                    url = url,
                    error = ?err,
                    "Error while downloading Forge {}. Trying next suffix.",
                    &forge_version
                );
                continue;
            }

            break;
        }

        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let profile = Self::get_profile_from_installer(&mut archive)?;

        let vanilla_mapper = VanillaLibrariesMapper { path: &game_paths.libraries };

        let downloader = match &profile {
            ForgeProfile::New(new) => LibrariesDownloader::new(&vanilla_mapper, &new.libraries),
            ForgeProfile::Old(old) => {
                struct ForgeOldLibrariesMapper<'a> {
                    path: &'a Path,
                }

                impl LibrariesMapper<ForgeOldLibrary> for ForgeOldLibrariesMapper<'_> {
                    fn proceed(&self, library: &ForgeOldLibrary) -> Option<FileDownloader> {
                        let (name, url) = (library.name.as_str(), library.url.as_deref());

                        let maven_data = MavenData::new(name);
                        let url = url.map_or(format!("https://libraries.minecraft.net/{}", maven_data.url), |url| {
                            format!("{url}{}", &maven_data.url)
                        });

                        Some(FileDownloader::new(url, self.path.join(&maven_data.path)))
                    }
                }

                let mapper = ForgeOldLibrariesMapper { path: &game_paths.libraries };
                LibrariesDownloader::new(&mapper, &old.version_info.libraries)
            }
        };

        let mut downloader = DownloadQueue::new().with_downloader(downloader);
        let mut processors_data = None;

        if matches!(profile, ForgeProfile::New(_)) {
            let profile = Self::get_install_profile(&mut archive)?;
            downloader = downloader.with_downloader(LibrariesDownloader::new(&vanilla_mapper, &profile.libraries));

            processors_data = Some(ProcessorsData {
                data: profile.data,
                processors: profile.processors,
            });
        }

        let library_data = match &profile {
            ForgeProfile::New(_) => {
                let profile = Self::get_install_profile(&mut archive)?;
                profile
                    .data
                    .get("BINPATCH")
                    .map(|data| &data.client)
                    .map(|client| ForgeLibraryExtractionData {
                        library_path: client[1..].to_string(),
                        target_path: forge_binpatch_path(&game_version, &forge_version),
                    })
            }
            ForgeProfile::Old(old) => Some(ForgeLibraryExtractionData {
                library_path: old.install.file_path.clone(),
                target_path: game_paths.libraries.join(MavenData::new(&old.install.path).path),
            }),
        };

        Ok(Forge {
            profile,
            downloader,
            game_version,
            forge_version,
            game_paths,
            library_data,
            processors_data,
        })
    }
}

impl ToLoaderProfile for Forge {
    fn to_profile(&self) -> LoaderProfile {
        LoaderProfile {
            loader: Loader::Forge,
            main_class: self.profile.main_class().to_string(),
            args: self.profile.simple_args(),
            libraries: self.profile.simple_libraries(),
        }
    }
}

fn forge_installer_path(game_version: &str, forge_version: &str) -> PathBuf {
    Path::new(DOT_NOMI_TEMP_DIR).join(format!("{game_version}-{forge_version}.jar"))
}

fn forge_binpatch_path(game_version: &str, forge_version: &str) -> PathBuf {
    PathBuf::from(DOT_NOMI_TEMP_DIR)
        .join(format!("{game_version}-{forge_version}"))
        .join("BINPATCH")
}

fn read_json_from_zip<T: DeserializeOwned>(file: &mut ZipFile<'_>) -> anyhow::Result<T> {
    let mut string = String::new();
    file.read_to_string(&mut string)?;

    let mut deserializer = serde_json::Deserializer::from_str(&string);

    serde_path_to_error::deserialize(&mut deserializer)
        .map_err(|e| anyhow!("Path: {}. Error: {}", e.path().clone().to_string(), e.into_inner().to_string()))
}

#[derive(Debug, Clone)]
struct ForgeLibraryExtractionData {
    library_path: String,
    target_path: PathBuf,
}

#[async_trait::async_trait]
impl Downloader for Forge {
    type Data = DownloadResult;

    fn total(&self) -> u32 {
        self.downloader.total()
    }

    async fn download(self: Box<Self>, sender: &dyn ProgressSender<Self::Data>) {
        Box::new(self.downloader).download(sender).await;
    }

    fn io(&self) -> PinnedFutureWithBounds<anyhow::Result<()>> {
        #[tracing::instrument(name = "Forge IO", skip(lib_data, processors_data), err)]
        async fn inner(
            game_version: String,
            forge_version: String,
            installer_path: PathBuf,
            java_runner: JavaRunner,
            game_paths: GamePaths,
            processors_data: Option<ProcessorsData>,
            lib_data: Option<ForgeLibraryExtractionData>,
        ) -> anyhow::Result<()> {
            if let Some(lib_data) = lib_data {
                info!("Extracting {}", &lib_data.library_path);
                let file = tokio::fs::File::open(installer_path).await?;
                let mut archive = zip::ZipArchive::new(file.into_std().await)?;
                let mut library_bytes = Vec::new();

                // If it's not in it's own scope then the future cannot be send between thread safely
                {
                    let mut library = archive.by_name(&lib_data.library_path)?;
                    library.read_to_end(&mut library_bytes)?;
                }

                if let Some(parent) = lib_data.target_path.parent().filter(|p| !p.exists()) {
                    tokio::fs::create_dir_all(parent).await?;
                }

                let mut target = tokio::fs::File::create(lib_data.target_path).await?;
                target.write_all(library_bytes.as_slice()).await?;
            }

            if let Some(processors_data) = processors_data {
                processors_data
                    .run_processors(&java_runner, &game_version, &forge_version, &game_paths)
                    .await?;
            }

            // Remove temporary files
            if let Some(binpatch_dir) = forge_binpatch_path(&game_version, &forge_version).parent() {
                tokio::fs::remove_dir_all(binpatch_dir).await?;
            };

            let forge_installer = forge_installer_path(&game_version, &forge_version);
            tokio::fs::remove_file(forge_installer).await?;

            Ok(())
        }

        Box::pin(inner(
            self.game_version.clone(),
            self.forge_version.clone(),
            self.installer_path(),
            JavaRunner::nomi_default(),
            self.game_paths.clone(),
            self.processors_data.clone(),
            self.library_data.clone(),
        ))
    }
}

#[derive(Debug, Clone)]
struct ProcessorsData {
    processors: Vec<Processor>,
    data: HashMap<String, Datum>,
}

impl ProcessorsData {
    fn apply_data_rules(&mut self, game_version: &str, forge_version: &str, game_paths: &GamePaths) {
        macro_rules! processor_rules {
            ($dest:expr; $($name:literal : client = $client:expr, server = $server:expr)+) => {
                $(std::collections::HashMap::insert(
                    $dest,
                    String::from($name),
                    Datum {
                        client: String::from($client),
                        server: String::from($server),
                    },
                );)+
            }
        }

        processor_rules! {
            &mut self.data;
            "SIDE":
                client = "client",
                server = ""
            "MINECRAFT_JAR" :
                client = game_paths.profile.join(format!("{game_version}.jar")).to_string_lossy(),
                server = ""
            "MINECRAFT_VERSION":
                client = game_version,
                server = ""
            "ROOT":
                client = game_paths.game.to_string_lossy(),
                server = ""
            "LIBRARY_DIR":
                client = game_paths.libraries.to_string_lossy(),
                server = ""
            "BINPATCH":
                client = forge_binpatch_path(game_version, forge_version).to_string_lossy(),
                server = ""
        }
    }

    fn get_processor_classpath<'a>(libraries_dir: &Path, libraries: impl Iterator<Item = &'a str>) -> String {
        libraries
            .map(MavenData::new)
            .map(|m| m.path)
            .map(|p| libraries_dir.join(p))
            .map(|p| p.to_string_lossy().into_owned())
            .join(CLASSPATH_SEPARATOR)
    }

    #[tracing::instrument]
    async fn get_processor_main_class(processor_jar: PathBuf) -> anyhow::Result<String> {
        tokio::task::spawn_blocking(|| {
            let file = std::fs::File::open(processor_jar)?;
            let mut archive = zip::ZipArchive::new(file)?;

            let file = archive.by_name("META-INF/MANIFEST.MF")?;
            let reader = std::io::BufReader::new(file);

            let opt = reader
                .lines()
                .filter_map(|l| l.inspect_err(|err| error!(error= %err,"Error while reading line")).ok())
                .find(|line| line.starts_with("Main-Class:"))
                .and_then(|line| line.split(':').nth(1).map(str::trim).map(ToString::to_string));

            match opt {
                Some(main_class) => Ok(main_class),
                None => Err(anyhow!("Main class is not found")),
            }
        })
        .await?
    }

    fn get_processor_arguments<'a>(libraries_dir: &Path, arguments: impl Iterator<Item = &'a str>, data: &HashMap<String, Datum>) -> Vec<String> {
        let mut args = Vec::new();

        for argument in arguments {
            // Some arguments are enclosed in {} or [] so we need to get rid of them
            let trimmed_arg = &argument[1..argument.len() - 1];

            match argument {
                arg if arg.starts_with('{') => {
                    let Some(entry) = data.get(trimmed_arg) else {
                        continue;
                    };

                    let arg = if entry.client.starts_with('[') {
                        let data = MavenData::new(&entry.client[1..entry.client.len() - 1]);
                        let path = libraries_dir.join(data.path);
                        path.to_string_lossy().into_owned()
                    } else {
                        entry.client.clone()
                    };

                    args.push(arg);
                }
                arg if arg.starts_with('[') => {
                    let data = MavenData::new(trimmed_arg);
                    let path = libraries_dir.join(data.path);
                    args.push(path.to_string_lossy().into_owned());
                }
                arg => args.push(arg.to_string()),
            }
        }

        args
    }

    async fn run_processors(
        mut self,
        java_runner: &JavaRunner,
        game_version: &str,
        forge_version: &str,
        game_paths: &GamePaths,
    ) -> anyhow::Result<()> {
        self.apply_data_rules(game_version, forge_version, game_paths);

        let total = self
            .processors
            .iter()
            .filter(|p| p.sides.as_ref().is_some_and(|sides| sides.iter().any(|s| s == "client")))
            .count();
        let mut ok = 0;
        let mut err = 0;

        for mut processor in self.processors {
            if processor.sides.as_ref().is_some_and(|sides| !sides.iter().any(|s| s == "client")) {
                continue;
            }

            processor.classpath.push(processor.jar.clone());

            let processor_jar_path = MavenData::new(&processor.jar).path;

            let classpath = Self::get_processor_classpath(&game_paths.libraries, processor.classpath.iter().map(String::as_str));
            let main_class = Self::get_processor_main_class(game_paths.libraries.join(processor_jar_path)).await?;
            let arguments = Self::get_processor_arguments(&game_paths.libraries, processor.args.iter().map(String::as_str), &self.data);

            let output = dbg!(Command::new(java_runner.get()).arg("-cp").arg(classpath).arg(main_class).args(arguments))
                .output()
                .await?;

            if output.status.success() {
                ok += 1;
                info!("Processor finished successfully");
            } else {
                err += 1;
                let error = String::from_utf8_lossy(&output.stderr);
                error!(error = %error, "Processor failed");
            }
        }

        info!(total, ok, err, "Finished processors execution");

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct ForgeVersions {
    pub homepage: String,
    pub promos: HashMap<String, String>,
}

#[derive(Debug)]
pub enum ForgeVersion {
    Recommended,
    Latest,
    Specific(String),
}

impl ForgeVersion {
    pub fn format(&self, game_version: &str) -> String {
        format!("{game_version}-{}", self.as_str())
    }

    pub fn as_str(&self) -> &str {
        match self {
            ForgeVersion::Recommended => "recommended",
            ForgeVersion::Latest => "latest",
            ForgeVersion::Specific(v) => v,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ForgeProfile {
    New(Box<ForgeProfileNew>),
    Old(Box<ForgeProfileOld>),
}

impl ForgeProfile {
    pub fn main_class(&self) -> &str {
        match self {
            ForgeProfile::New(new) => &new.main_class,
            ForgeProfile::Old(old) => &old.version_info.main_class,
        }
    }

    pub fn simple_args(&self) -> SimpleArgs {
        match self {
            ForgeProfile::New(new) => {
                let Arguments::New { game, jvm } = &new.arguments else {
                    return SimpleArgs {
                        jvm: Vec::new(),
                        game: Vec::new(),
                    };
                };

                let filter_args = |args: &[Argument]| {
                    args.iter()
                        .filter_map(|a| match a {
                            Argument::String(s) => Some(s),
                            Argument::Struct { .. } => None,
                        })
                        .cloned()
                        .collect_vec()
                };

                SimpleArgs {
                    game: filter_args(game),
                    jvm: filter_args(jvm),
                }
            }
            ForgeProfile::Old(old) => SimpleArgs {
                game: old.version_info.minecraft_arguments.split_once("--tweakClass").map_or_else(
                    || {
                        warn!("Cannot find `--tweakClass` parameter in the Forge arguments list. Game might not launch.");
                        Vec::new()
                    },
                    |(_, val)| vec!["--tweakClass", val.trim()].into_iter().map(String::from).collect_vec(),
                ),
                jvm: Vec::new(),
            },
        }
    }

    fn simple_libraries(&self) -> Vec<SimpleLib> {
        match self {
            ForgeProfile::New(new) => new
                .libraries
                .iter()
                .map(|lib| SimpleLib {
                    jar: lib.downloads.artifact.as_ref().and_then(|a| a.path.as_ref()).map_or_else(
                        || {
                            warn!("Forge library does not have path. Game might not launch.");
                            PathBuf::new()
                        },
                        PathBuf::from,
                    ),
                    artifact: MavenArtifact::new(&lib.name),
                })
                .collect_vec(),
            ForgeProfile::Old(old) => old
                .version_info
                .libraries
                .iter()
                // .filter(|lib| lib.clientreq.is_some_and(|required| required))
                .map(|lib| lib.name.as_str())
                .map(MavenArtifact::new)
                .map(SimpleLib::from)
                .collect_vec(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForgeProfileNew {
    // #[serde(rename = "_comment_")]
    // pub comment: Vec<String>,
    pub id: String,
    pub time: String,
    pub release_time: String,
    #[serde(rename = "type")]
    pub forge_profile_type: String,
    pub main_class: String,
    pub inherits_from: String,
    pub logging: Logging,
    pub arguments: crate::repository::manifest::Arguments,
    pub libraries: Vec<Library>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Logging {}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForgeInstallProfile {
    // #[serde(rename = "_comment_")]
    // comment: Vec<String>,
    // spec: i64,
    // profile: String,
    // version: String,
    // path: Option<serde_json::Value>,
    // minecraft: String,
    // server_jar_path: String,
    data: HashMap<String, Datum>,
    processors: Vec<Processor>,
    libraries: Vec<Library>,
    // icon: String,
    // json: String,
    // logo: String,
    // mirror_list: String,
    // welcome: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Datum {
    client: String,
    server: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Processor {
    sides: Option<Vec<String>>,
    jar: String,
    classpath: Vec<String>,
    args: Vec<String>,
    outputs: Option<Outputs>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Outputs {
    #[serde(rename = "{MC_SLIM}")]
    mc_slim: String,
    #[serde(rename = "{MC_EXTRA}")]
    mc_extra: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForgeProfileOld {
    pub install: Install,
    pub version_info: VersionInfo,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Install {
    // pub profile_name: String,
    // pub target: String,
    pub path: String,
    // pub version: String,
    pub file_path: String,
    // pub welcome: String,
    // pub minecraft: String,
    // pub mirror_list: String,
    // pub logo: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub id: String,
    pub time: String,
    pub release_time: String,
    #[serde(rename = "type")]
    pub version_info_type: String,
    pub minecraft_arguments: String,
    pub main_class: String,
    pub minimum_launcher_version: i64,
    pub assets: String,
    pub inherits_from: String,
    pub jar: String,
    pub libraries: Vec<ForgeOldLibrary>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForgeOldLibrary {
    pub name: String,
    pub url: Option<String>,
    pub serverreq: Option<bool>,
    pub checksums: Option<Vec<String>>,
    pub clientreq: Option<bool>,
}

#[cfg(test)]
mod tests {
    use tracing::{debug, Level};

    use crate::instance::InstanceProfileId;

    use super::*;

    #[tokio::test]
    async fn get_versions_test() {
        let versions = Forge::get_versions("1.7.10").await.unwrap();
        println!("{versions:#?}");
    }

    #[tokio::test]
    async fn create_forge_test() {
        let recommended = Forge::new("1.7.10", ForgeVersion::Recommended, GamePaths::from_id(InstanceProfileId::ZERO))
            .await
            .unwrap();
        println!("{recommended:#?}");

        let latest = Forge::new("1.19.2", ForgeVersion::Latest, GamePaths::from_id(InstanceProfileId::ZERO))
            .await
            .unwrap();
        println!("{latest:#?}");
    }

    #[tokio::test]
    async fn download_installer_test() {
        let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt().with_max_level(Level::DEBUG).finish());

        debug!("Test");

        let recommended = Forge::new("1.7.10", ForgeVersion::Recommended, GamePaths::from_id(InstanceProfileId::ZERO))
            .await
            .unwrap();
        println!("{recommended:#?}");

        let io = recommended.io();

        let (tx, mut rx) = tokio::sync::mpsc::channel(5);
        Box::new(recommended).download(&tx).await;
        dbg!(rx.recv().await);

        io.await.unwrap();
    }
}
