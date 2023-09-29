use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use super::{Config, ConfigSetter, ConfigState};

#[async_trait::async_trait(?Send)]
pub trait Toml {
    async fn write(&self) -> anyhow::Result<()>;
    async fn read(&mut self) -> anyhow::Result<()>;
}

#[async_trait::async_trait(?Send)]
impl<State> Toml for Config<State>
where
    State: ConfigState + ConfigSetter,
{
    async fn write(&self) -> anyhow::Result<()> {
        if let Some(path) = &self.path.parent() {
            tokio::fs::create_dir_all(path).await?;
        }

        let data = toml::to_string_pretty(self.data.convert())?;

        let mut file = tokio::fs::File::create(&self.path).await?;
        file.write_all(data.as_bytes()).await?;

        Ok(())
    }

    async fn read(&mut self) -> anyhow::Result<()> {
        let data = tokio::fs::read_to_string(&self.path).await?;

        let deserialized = toml::from_str(&data)?;
        self.data.set(deserialized);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct Mock {
        name: String,
        vec: Vec<i32>,
    }

    #[tokio::test]
    async fn borrowed_write_test() {
        let data = Mock {
            name: "".to_string(),
            vec: vec![],
        };
        let cfg = Config::borrowed(&data, "./borrowed_write_test.toml");

        cfg.read().await.unwrap();
    }

    #[tokio::test]
    async fn borrowed_read_test() {
        let cfg = Config::owned(
            Mock {
                name: "unread".to_string(),
                vec: vec![],
            },
            "./borrowed_write_test.toml",
        );
        cfg.read().await.unwrap();
    }
}
