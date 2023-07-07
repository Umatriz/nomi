use reqwest::{blocking, Client};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::task::spawn_blocking;

#[derive(Serialize, Deserialize, Debug)]
pub struct Assets {
    // pub objects: Vec<HashMap<String, AssetInformation>>
    pub objects: HashMap<String, AssetInformation>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetInformation {
    pub hash: String,
    pub size: i64,
}

impl AssetInformation {
    pub fn get_asset_dir_name(&self) -> &str {
        &self.hash[0..3]
    }
}

#[derive(Debug)]
pub struct AssetsDownload {
    assets: Assets,
    id: String,
    url: String,
}

impl AssetsDownload {
    pub async fn new(url: String, id: String) -> Self {
        Self {
            assets: Self::init(url.clone()).await.unwrap(),
            id,
            url,
        }
    }

    async fn init(url: String) -> Result<Assets, reqwest::Error> {
        let data: Assets = Client::new().get(url).send().await?.json().await?;

        Ok(data)
    }

    fn create_dir(&self, main_dir: &str, asset_dir_name: &str) -> PathBuf {
        let path = Path::new(main_dir)
            .join("assets")
            .join("objects")
            .join(asset_dir_name);

        let _ = std::fs::create_dir_all(&path);

        // TODO: remove this after debug
        println!("Dir {} created successfully", path.to_str().unwrap());

        path
    }

    pub async fn get_assets_json(&self, assets_dir: &String) -> Result<(), reqwest::Error> {
        let filen = format!("{}.json", self.id);
        let path = Path::new(&assets_dir)
            .join("assets")
            .join("indexes")
            .join(filen);

        let _ = std::fs::create_dir_all(path.parent().unwrap());

        let body = Client::new().get(&self.url).send().await?.text().await?;

        match std::fs::write(&path, body) {
            Ok(_) => println!("Downloaded successfully {}", path.to_str().unwrap()),
            Err(e) => println!("Error: {}", e),
        }

        Ok(())
    }

    pub async fn download_assets(&self, dir: &str) {
        for (_k, v) in self.assets.objects.iter() {
            let path = self.create_dir(dir, &v.hash[0..2]);

            println!("{:?}, {}", path.join(&v.hash), &v.hash[0..2]);

            let mut file = std::fs::File::create(path.join(&v.hash)).unwrap();

            let url = format!(
                "https://resources.download.minecraft.net/{}/{}",
                &v.hash[0..2],
                v.hash
            );
            let _response =
                spawn_blocking(move || blocking::get(url).unwrap().copy_to(&mut file).unwrap())
                    .await;
        }
    }
}
