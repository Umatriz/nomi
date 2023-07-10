use std::path::PathBuf;

pub struct GetPath;

impl GetPath {
    pub fn config() -> PathBuf {
        std::env::current_dir().unwrap().join("config.json")
    }

    pub fn game() -> PathBuf {
        std::env::current_dir().unwrap().join("minecraft")
    }

    pub fn java_bin() -> Option<PathBuf> {
        let _path = std::env::var("Path").unwrap();
        let path_vec = _path.split(';').collect::<Vec<&str>>();
        let mut java_bin: Option<PathBuf> = None;
        for i in path_vec.iter() {
            if i.contains("java") {
                let pb = PathBuf::from(i).join("java.exe");
                match pb.exists() {
                    true => java_bin = Some(pb),
                    false => java_bin = None,
                }
            }
        }
        java_bin
    }
}
