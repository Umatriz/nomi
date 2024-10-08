use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{mpsc::Sender, Arc},
};

use eframe::egui::Context;
use egui_task_manager::{Progress, TaskProgressShared};
use itertools::Itertools;
use nomi_core::{
    calculate_sha1,
    downloads::{progress::MappedSender, traits::Downloader, DownloadSet, FileDownloader},
    fs::read_toml_config,
    instance::{Instance, InstanceProfileId},
};
use nomi_modding::{
    modrinth::{
        project::{ProjectData, ProjectId},
        version::{ProjectVersionsData, Version, VersionId},
    },
    Query,
};
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{errors_pool::ErrorPoolExt, progress::UnitProgress, DOT_NOMI_MODS_STASH_DIR, NOMI_LOADED_LOCK_FILE, NOMI_LOADED_LOCK_FILE_NAME};

#[derive(Serialize, Deserialize, Default, PartialEq, Eq, Hash, Debug)]
#[serde(transparent)]
pub struct ModsConfig {
    pub mods: Vec<Mod>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Clone)]
pub struct Mod {
    pub project_id: ProjectId,
    pub name: String,
    pub version_id: VersionId,
    pub version_name: Option<String>,
    pub version_number: Option<String>,
    pub is_downloaded: bool,
    pub files: Vec<ModFile>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ModFile {
    pub sha1: String,
    pub url: String,
    pub filename: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleDependency {
    pub name: String,
    pub versions: Vec<Arc<Version>>,
    pub project_id: ProjectId,
    pub is_required: bool,
}

pub async fn download_added_mod(progress: TaskProgressShared, ctx: Context, target_path: PathBuf, files: Vec<ModFile>) {
    let _ = progress.set_total(files.len() as u32);

    let mut set = DownloadSet::new();

    for file in files {
        let downloader = FileDownloader::new(file.url, target_path.join(file.filename))
            .with_sha1(file.sha1)
            .into_retry();
        set.add(Box::new(downloader));
    }

    let sender = MappedSender::new_progress_mapper(Box::new(progress.sender())).with_side_effect(move || ctx.request_repaint());

    Box::new(set).download(&sender).await;
}

pub async fn get_and_proceed_deps(version: Arc<Version>, game_version: String, loader: String) -> Option<Vec<SimpleDependency>> {
    let mut deps = Vec::new();
    proceed_deps(&mut deps, version, game_version, loader).await.report_error().map(|_| deps)
}

pub async fn proceed_deps(dist: &mut Vec<SimpleDependency>, version: Arc<Version>, game_version: String, loader: String) -> anyhow::Result<()> {
    for dep in &version.dependencies {
        let query = Query::new(
            ProjectVersionsData::builder()
                .id_or_slug(dep.project_id.clone())
                .game_versions(vec![game_version.clone()])
                .loaders(vec![loader.clone()])
                .build(),
        );

        let data = query.query().await?;

        let versions = data.into_iter().map(Arc::new).collect_vec();

        let query = Query::new(ProjectData::new(dep.project_id.clone()));
        let project = query.query().await?;

        dist.push(SimpleDependency {
            name: project.title.clone(),
            versions: versions.clone(),
            is_required: dep.dependency_type.as_ref().is_some_and(|d| d == "required") || dep.dependency_type.is_none(),
            project_id: project.id,
        });
    }

    Ok(())
}

pub async fn download_mods(progress: TaskProgressShared, ctx: Context, versions: Vec<(Arc<Version>, PathBuf, String)>) -> anyhow::Result<Vec<Mod>> {
    let _ = progress.set_total(
        versions
            .iter()
            .map(|v| v.0.files.iter().filter(|f| f.primary).collect::<Vec<_>>().len() as u32)
            .sum(),
    );

    let mut mods = Vec::new();
    for (version, path, name) in versions {
        let mod_value = download_mod(progress.sender(), ctx.clone(), path, name, version).await?;
        mods.push(mod_value);
    }

    Ok(mods)
}

pub async fn download_mod(sender: Sender<Box<dyn Progress>>, ctx: Context, dir: PathBuf, name: String, version: Arc<Version>) -> anyhow::Result<Mod> {
    let mut set = DownloadSet::new();

    let mut downloaded_files = Vec::new();

    // We do not download any dependencies. Just the mod.
    for file in version.files.iter().filter(|f| f.primary) {
        if tokio::fs::read_to_string(dir.join(&file.filename))
            .await
            .is_ok_and(|s| calculate_sha1(s) == file.hashes.sha1)
        {
            let _ = sender.send(Box::new(UnitProgress));
            continue;
        }

        downloaded_files.push(ModFile {
            sha1: file.hashes.sha1.clone(),
            url: file.url.clone(),
            filename: file.filename.clone(),
        });

        let downloader = FileDownloader::new(file.url.clone(), dir.join(&file.filename))
            .with_sha1(file.hashes.sha1.clone())
            .into_retry();
        set.add(Box::new(downloader));
    }

    let sender = MappedSender::new_progress_mapper(Box::new(sender)).with_side_effect(move || ctx.request_repaint());

    Box::new(set).download(&sender).await;

    Ok(Mod {
        name,
        version_id: version.id.clone(),
        version_name: Some(version.name.clone()),
        version_number: Some(version.version_number.clone()),
        is_downloaded: true,
        files: downloaded_files,
        project_id: version.project_id.clone(),
    })
}

#[derive(Serialize, Deserialize)]
pub struct CurrentlyLoaded {
    id: usize,
}

impl CurrentlyLoaded {
    pub async fn write_with_comment(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let mut file = File::create(path.as_ref()).await?;

        file.write_all(b"# This file is automatically generated by Nomi.\n# It is not intended for manual editing.\n")
            .await?;
        file.write_all(toml::to_string_pretty(&self)?.as_bytes()).await?;

        file.flush().await?;

        Ok(())
    }
}

/// Load profile's mods by creating hard links.
pub async fn load_mods(id: InstanceProfileId) -> anyhow::Result<()> {
    async fn make_link(source: &Path, mods_dir: &Path, file_name: &OsStr) -> anyhow::Result<()> {
        let dst = mods_dir.join(file_name);
        tokio::fs::hard_link(source, dst).await.map_err(|e| e.into())
    }

    let instance_path = Instance::path_from_id(id.instance());
    let mods_stash = mods_stash_path_for_profile(id);
    let mods_dir = instance_path.join("mods");
    let loaded_lock_path = mods_dir.join(NOMI_LOADED_LOCK_FILE);

    if !loaded_lock_path.exists() {
        CurrentlyLoaded { id: id.profile() }.write_with_comment(&loaded_lock_path).await?
    }

    let mut loaded = read_toml_config::<CurrentlyLoaded>(&loaded_lock_path).await?;

    let target_dir = mods_dir
        .read_dir()?
        .filter_map(|r| r.ok())
        .map(|e| (e.file_name(), e.path()))
        .collect::<Vec<_>>();

    if loaded.id == id.profile() {
        let mut dir = tokio::fs::read_dir(mods_stash).await?;

        let mut mods_in_the_stash = Vec::new();

        while let Ok(Some(entry)) = dir.next_entry().await {
            mods_in_the_stash.push(entry.file_name());

            if target_dir.iter().any(|i| i.0 == entry.file_name()) {
                continue;
            }

            let source = entry.path();

            let Some(file_name) = source.file_name() else {
                continue;
            };

            make_link(&source, &mods_dir, file_name).await?;
        }

        for (file_name, path) in target_dir {
            if file_name == NOMI_LOADED_LOCK_FILE_NAME {
                continue;
            }

            if mods_in_the_stash.contains(&file_name) {
                continue;
            }

            tokio::fs::remove_file(path).await.report_error();
        }

        return Ok(());
    }

    let mut dir = tokio::fs::read_dir(&mods_dir).await?;
    while let Ok(Some(entry)) = dir.next_entry().await {
        if entry.file_name() == NOMI_LOADED_LOCK_FILE_NAME {
            continue;
        }

        tokio::fs::remove_file(entry.path()).await?;
    }

    let mut dir = tokio::fs::read_dir(mods_stash).await?;

    while let Ok(Some(entry)) = dir.next_entry().await {
        let source = entry.path();

        let Some(file_name) = source.file_name() else {
            continue;
        };

        make_link(&source, &mods_dir, file_name).await?;
    }

    loaded.id = id.profile();

    loaded.write_with_comment(loaded_lock_path).await?;

    Ok(())
}

pub fn mods_stash_path_for_profile(profile_id: InstanceProfileId) -> PathBuf {
    Instance::path_from_id(profile_id.instance())
        .join(DOT_NOMI_MODS_STASH_DIR)
        .join(format!("{}", profile_id.profile()))
}
