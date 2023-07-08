use anyhow::Result;
use std::path::PathBuf;

pub struct GetPath;

impl GetPath {
  pub fn config() -> Result<PathBuf> {
    // TODO: Remove this .join()
    return Ok(
      std::env::current_dir()?.join("config.json")
    );
  }

  pub fn game() -> Result<PathBuf> {
    return Ok(
      std::env::current_dir()?.join("minecraft")
    )
  }
  
  // TODO: retern Err if cant find
  pub fn java_bin() -> Result<Option<PathBuf>> {
    let _path = std::env::var("Path")?;
    let path_vec = _path.split(';').collect::<Vec<&str>>();
    let mut java_bin: Option<PathBuf> = None;
    for i in path_vec.iter() {
      if i.contains("java") {
        let pb = PathBuf::from(i).join("java.exe");
        match pb.exists() {
          true => {
            java_bin = Some(pb)
          },
          false => {
            java_bin = None
          }
        }
      }
    }
    return Ok(java_bin);
  }
}
