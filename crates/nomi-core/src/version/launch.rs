pub trait VersionLaunch {
    fn launch() -> anyhow::Result<()>;
}
