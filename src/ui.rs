use im::Vector;
use druid::{Data, Lens};

use crate::downloads::launcher_manifest::LauncherManifestVersion;

#[derive(Clone, Data, Lens)]
pub struct Launcher {
  pub versions: Vector<LauncherManifestVersion>,
  pub username: String,
}

