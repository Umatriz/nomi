use std::path::Path;

pub fn path_to_string(p: impl AsRef<Path>) -> String {
    p.as_ref().to_string_lossy().to_string()
}
