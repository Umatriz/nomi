use std::{path::{Path, PathBuf}, collections::HashMap};
use reqwest::{blocking, get, Client};
use serde::{Serialize, Deserialize};
use tokio::task::spawn_blocking;

#[derive(Serialize, Deserialize, Debug)]
pub struct Assets {
  // pub objects: Vec<HashMap<String, AssetInformation>>
  pub objects: HashMap<String, AssetInformation>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AssetInformation {
  pub hash: String,
  pub size: i64,
}

impl AssetInformation {
  pub fn get_asset_dir_name(&self) -> &str {
    return &self.hash[0..3];
  }
}

#[derive(Debug)]
pub struct AssetsDownload {
  assets: Assets
}

impl AssetsDownload {
  pub async fn new(url: String) -> Self {
    Self {
      assets: Self::init(url)
        .await
        .unwrap(),
    }
  }

  async fn init(url: String) -> Result<Assets, reqwest::Error> {
    let data: Assets = Client::new()
      .get(url)
      .send()
      .await?
      .json()
      .await?;
    
    return Ok(data);
  }

  fn create_dir (&self, main_dir: &str, asset_dir_name: &str) -> PathBuf {
    let path = Path::new(main_dir)
      .join("assets")
      .join("objects")
      .join(asset_dir_name);

    let _ = std::fs::create_dir_all(&path);

    // TODO: remove this after debug
    println!("Dir {} created successfully", path.to_str().unwrap().to_string());

    return path;
  }

  pub async fn download_assets(&self, dir: &str) {
    for (_k, v) in self.assets.objects.iter() {
      let path = self.create_dir(dir, &v.hash[0..2]);

      println!("{:?}, {}", path.join(&v.hash), &v.hash[0..2]);
  
      let mut file = std::fs::File::create(path.join(&v.hash)).unwrap();
  
      let url = format!("https://resources.download.minecraft.net/{}/{}", v.hash[0..2].to_string(), v.hash);
      let _response = spawn_blocking(move || {
        blocking::get(url)
          .unwrap()
          .copy_to(&mut file)
          .unwrap()
      }).await;
    }
  }
}