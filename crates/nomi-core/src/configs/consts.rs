
// java consts
pub const PORTABLE_URL: &str = "https://download.oracle.com/java/17/latest/jdk-17_windows-x64_bin.zip";
pub const JDK_17_0_7_PORTABLE_SHA256: &str = "98385c1fd4db7ad3fd7ca2f33a1fadae0b15486cfde699138d47002d7068084a";

//launch consts
#[cfg(windows)]
pub const CLASSPATH_SEPARATOR: &str = ";";

#[cfg(not(windows))]
pub const CLASSPATH_SEPARATOR: &str = ":";

//loader const
pub const FABRIC_MAVEN: &str = "https://maven.fabricmc.net/";

//utils const
pub const LAUNCHER_MANIFEST: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";