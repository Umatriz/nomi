use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Variables {
    pub root: PathBuf,
}

impl Variables {
    pub fn is_current(&self) -> anyhow::Result<bool> {
        Ok(std::env::current_dir()? == self.root)
    }
}

#[cfg(test)]
mod tests {
    use crate::configs::read_config;
    use crate::configs::write_config;

    use super::*;

    #[tokio::test]
    async fn write_test() {
        let v = Variables {
            root: std::env::current_dir().unwrap(),
        };

        write_config(&v, "./configs/Variables.toml").await.unwrap();
    }

    #[tokio::test]
    async fn read_test() {
        let v = read_config::<Variables>("./configs/Variables.toml")
            .await
            .unwrap();

        assert_eq!(
            v.root,
            Variables {
                root: std::env::current_dir().unwrap(),
            }
            .root
        );
    }
}
