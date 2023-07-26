pub mod toml;

pub(crate) trait Config {
    fn write() -> anyhow::Result<()>;
    fn read() -> anyhow::Result<Self>
    where
        Self: std::marker::Sized;
}
