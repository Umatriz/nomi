pub const DOT_NOMI_DIR: &str = "./.nomi";
pub const DOT_NOMI_TEMP_DIR: &str = "./.nomi/temp";
pub const DOT_NOMI_CONFIGS_DIR: &str = "./.nomi/configs";
pub const DOT_NOMI_SETTINGS_CONFIG: &str = "./.nomi/configs/Settings.toml";
pub const DOT_NOMI_LOGS_DIR: &str = "./.nomi/logs";
pub const DOT_NOMI_JAVA_DIR: &str = "./.nomi/java";
pub const DOT_NOMI_JAVA_EXECUTABLE: &str = "./.nomi/java/jdk-22.0.1/bin/java";
pub const DOT_NOMI_DATA_PACKS_DIR: &str = "./.nomi/datapacks";

pub const LIBRARIES_DIR: &str = "./libraries";
pub const ASSETS_DIR: &str = "./assets";

pub const INSTANCES_DIR: &str = "./instances";
/// Path to instance's config file with respect to instance's directory.
///
/// # Example
///
/// ```rust
/// # use std::path::Path;
/// Path::new("./instances/example").join(INSTANCE_CONFIG)
/// ```
pub const INSTANCE_CONFIG: &str = ".nomi/Instance.toml";

pub const NOMI_VERSION: &str = "0.2.0";
pub const NOMI_NAME: &str = "Nomi";
