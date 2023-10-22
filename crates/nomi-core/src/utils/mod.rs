use std::{cell::OnceCell, path::PathBuf};

use serde::de::DeserializeOwned;

pub mod state;

pub async fn get<T: DeserializeOwned>(url: impl Into<String>) -> anyhow::Result<T> {
    Ok(reqwest::get(url.into()).await?.json::<T>().await?)
}
