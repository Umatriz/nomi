use std::{marker::PhantomData, path::PathBuf};

use itertools::Itertools;
use tracing::info;

use crate::{
    game_paths::GamePaths,
    instance::{
        launch::{macros::replace, rules::is_library_passes},
        loader::LoaderProfile,
    },
    markers::Undefined,
    maven_data::MavenArtifact,
    repository::{
        manifest::{Argument, Arguments, Classifiers, DownloadFile, Manifest, Value},
        username::Username,
    },
    utils::path_to_string,
    NOMI_NAME, NOMI_VERSION,
};

use super::{rules::is_rule_passes, LaunchInstance, CLASSPATH_SEPARATOR};

pub enum WithUserData {}
pub enum WithClasspath {}

pub struct ArgumentsBuilder<'a, S = Undefined, U = Undefined> {
    instance: &'a LaunchInstance,
    manifest: &'a Manifest,
    paths: &'a GamePaths,
    classpath: Vec<PathBuf>,
    classpath_string: String,
    native_libs: Vec<PathBuf>,
    user_data: UserData,

    _classpath_marker: PhantomData<S>,
    _user_data_marker: PhantomData<U>,
}

#[derive(Default, Debug)]
pub struct UserData {
    pub username: Username,
    pub uuid: Option<String>,
    pub access_token: Option<String>,
}

struct JvmArguments(Vec<Argument>);
struct GameArguments(Vec<Argument>);

pub struct LoaderArguments<'a, 'b> {
    builder: &'b ArgumentsBuilder<'a, WithClasspath, WithUserData>,
    profile: Option<&'a LoaderProfile>,
}

impl<'a, 'b> LoaderArguments<'a, 'b> {
    pub fn jvm_arguments(&self) -> Vec<String> {
        self.profile.map_or(Vec::new(), |profile| {
            profile.args.jvm.iter().map(|v| self.builder.parse_args_from_str(v)).collect_vec()
        })
    }

    pub fn game_arguments(&self) -> Vec<String> {
        self.profile.map_or(Vec::new(), |profile| {
            profile.args.game.iter().map(|v| self.builder.parse_args_from_str(v)).collect_vec()
        })
    }
}

impl<'a> ArgumentsBuilder<'a, Undefined, Undefined> {
    pub fn new(paths: &'a GamePaths, instance: &'a LaunchInstance, manifest: &'a Manifest) -> ArgumentsBuilder<'a, Undefined, Undefined> {
        ArgumentsBuilder {
            instance,
            manifest,
            paths,
            classpath: Vec::new(),
            classpath_string: String::new(),
            native_libs: Vec::new(),
            user_data: UserData::default(),
            _classpath_marker: PhantomData,
            _user_data_marker: PhantomData,
        }
    }
}

impl<'a, U> ArgumentsBuilder<'a, Undefined, U> {
    pub fn build_classpath(self) -> ArgumentsBuilder<'a, WithClasspath, U> {
        let (classpath, native_libs) = self.classpath();
        ArgumentsBuilder {
            instance: self.instance,
            manifest: self.manifest,
            paths: self.paths,
            user_data: self.user_data,
            classpath_string: itertools::intersperse(classpath.iter().map(|p| p.display().to_string()), CLASSPATH_SEPARATOR.to_string())
                .collect::<String>(),
            classpath,
            native_libs,
            _classpath_marker: PhantomData,
            _user_data_marker: PhantomData,
        }
    }
}

impl<'a, S> ArgumentsBuilder<'a, S, Undefined> {
    pub fn with_userdata(self, user_data: UserData) -> ArgumentsBuilder<'a, S, WithUserData> {
        ArgumentsBuilder {
            instance: self.instance,
            manifest: self.manifest,
            paths: self.paths,
            user_data,
            classpath_string: self.classpath_string,
            classpath: self.classpath,
            native_libs: self.native_libs,
            _classpath_marker: PhantomData,
            _user_data_marker: PhantomData,
        }
    }
}

impl<'a, U> ArgumentsBuilder<'a, WithClasspath, U> {
    pub fn classpath_as_str(&self) -> &str {
        &self.classpath_string
    }

    pub fn classpath_as_slice(&self) -> &[PathBuf] {
        &self.classpath
    }

    pub fn get_main_class(&self) -> &str {
        self.instance
            .loader_profile
            .as_ref()
            .map_or(&self.manifest.main_class, |profile| &profile.main_class)
    }

    pub fn get_native_libs(&self) -> &[PathBuf] {
        self.native_libs.as_slice()
    }

    pub fn custom_jvm_arguments(&self) -> &[String] {
        self.instance.jvm_args.as_slice()
    }
}

impl<'a> ArgumentsBuilder<'a, WithClasspath, WithUserData> {
    pub fn loader_arguments(&self) -> LoaderArguments<'a, '_> {
        LoaderArguments {
            builder: self,
            profile: self.instance.loader_profile.as_ref(),
        }
    }
    pub fn manifest_jvm_arguments(&self) -> Vec<String> {
        self.arguments_parser(
            |JvmArguments(jvm), _| jvm.clone(),
            |_| {
                vec![
                    format!("-Djava.library.path={}", self.paths.natives_dir().display()),
                    "-Dminecraft.launcher.brand=${launcher_name}".into(),
                    "-Dminecraft.launcher.version=${launcher_version}".into(),
                    format!(
                        "-Dminecraft.client.jar={}",
                        self.paths.version_jar_file(&self.instance.settings.version).display()
                    ),
                    "-cp".to_string(),
                    self.classpath_as_str().to_owned(),
                ]
            },
        )
    }

    pub fn manifest_game_arguments(&self) -> Vec<String> {
        self.arguments_parser(
            |_, GameArguments(game)| game.clone(),
            |arguments| arguments.split_whitespace().map(|arg| self.parse_args_from_str(arg)).collect(),
        )
    }

    fn parse_args_from_str(&self, source: &str) -> String {
        replace!(source,
            "${assets_root}" => &path_to_string(&self.paths.assets),
            "${game_assets}" => &path_to_string(&self.paths.assets),
            "${game_directory}" => &path_to_string(&self.paths.game),
            "${natives_directory}" => &path_to_string(self.paths.natives_dir()),
            "${library_directory}" => &path_to_string(&self.paths.libraries),
            "${launcher_name}" => NOMI_NAME,
            "${launcher_version}" => NOMI_VERSION,
            "${auth_access_token}" => self.user_data
                .access_token
                .clone()
                .unwrap_or("null".to_string())
                .as_str(),
            "${auth_session}" => "null",
            "${auth_player_name}" => self.user_data.username.get(),
            "${auth_uuid}" => self.user_data
                .uuid
                .clone()
                .unwrap_or(uuid::Uuid::new_v4().to_string())
                .as_str(),
            "${version_type}" => &self.instance.settings.version_type.as_str(),
            "${version_name}" => &self.instance.settings.version,
            "${assets_index_name}" => &self.manifest.asset_index.id,
            "${user_properties}" => "{}",
            "${classpath}" => &self.classpath_as_str(),
            "${classpath_separator}" => CLASSPATH_SEPARATOR
        )
    }

    fn arguments_parser(
        &self,
        new_arguments_parser: impl Fn(JvmArguments, GameArguments) -> Vec<Argument>,
        old_arguments_parser: impl Fn(String) -> Vec<String>,
    ) -> Vec<String> {
        match &self.manifest.arguments {
            Arguments::New { jvm, game } => self.parse_arguments(new_arguments_parser(JvmArguments(jvm.clone()), GameArguments(game.clone()))),
            Arguments::Old(arguments) => old_arguments_parser(arguments.clone()),
        }
    }

    fn parse_arguments(&self, args: Vec<Argument>) -> Vec<String> {
        args.into_iter()
            .flat_map(|arg| match arg {
                Argument::Struct { rules, value } => {
                    if !rules.iter().all(is_rule_passes) {
                        return vec![String::new()];
                    }

                    match value {
                        Value::String(v) => vec![self.parse_args_from_str(&v)],
                        Value::Array(arr) => arr.into_iter().map(|arg| self.parse_args_from_str(&arg)).collect(),
                    }
                }
                Argument::String(arg) => vec![self.parse_args_from_str(&arg)],
            })
            .filter(|arg| !arg.is_empty())
            .collect::<Vec<String>>()
    }
}

impl<'a, S, U> ArgumentsBuilder<'a, S, U> {
    #[tracing::instrument(skip(self), fields(result))]
    fn classpath(&self) -> (Vec<PathBuf>, Vec<PathBuf>) {
        fn match_natives(natives: &Classifiers) -> Option<&DownloadFile> {
            match std::env::consts::OS {
                "linux" => natives.natives_linux.as_ref(),
                "windows" => natives.natives_windows.as_ref(),
                "macos" => natives.natives_macos.as_ref(),
                _ => unreachable!(),
            }
        }

        let mut classpath = vec![Some(self.paths.version_jar_file(&self.instance.settings.version))];
        let mut native_libs = vec![];

        self.manifest
            .libraries
            .iter()
            .filter(|lib| is_library_passes(lib))
            .map(|lib| {
                let name = lib.name.as_str();
                (
                    lib.downloads
                        .artifact
                        .as_ref()
                        .filter(|_| {
                            let Some(loader_profile) = self.instance.loader_profile() else {
                                return true;
                            };

                            !loader_profile.libraries.iter().any(|lib| {
                                let value = lib.artifact.group == MavenArtifact::new(name).group;

                                if value {
                                    info!(vanilla = name, loader = %lib.artifact, "Found overlapping library. Using the one loader provides.");
                                }

                                value
                            })
                        })
                        .and_then(|artifact| artifact.path.as_ref())
                        .map(|path| self.paths.libraries.join(path)),
                    lib.downloads
                        .classifiers
                        .as_ref()
                        .and_then(|natives| match_natives(natives))
                        .and_then(|native_lib| native_lib.path.as_ref())
                        .map(|path| self.paths.libraries.join(path)),
                )
            })
            .for_each(|(lib, native)| {
                classpath.extend([lib, native.clone()]);
                native_libs.push(native);
            });

        let mut classpath = classpath.into_iter().flatten().collect::<Vec<_>>();

        let native_libs = native_libs.into_iter().flatten().collect::<Vec<_>>();

        if let Some(libs) = self
            .instance
            .loader_profile
            .as_ref()
            .map(|p| &p.libraries)
            .map(|libs| libs.iter().map(|lib| self.paths.libraries.join(&lib.jar)))
        {
            classpath.extend(libs);
        }

        let classpath = classpath.iter().cloned().collect_vec();

        (classpath, native_libs)
    }
}
