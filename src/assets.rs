use crate::{extract, Cache, BASE_URL, TARGET_PATH};
use ahash::HashMap;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{io::Cursor, path::Path};
use tokio::task::spawn_blocking;

#[derive(Serialize, Deserialize)]
pub struct NameHashMapping {
    #[serde(flatten)]
    pub inner: HashMap<String, String>,
}

impl Cache for NameHashMapping {}

#[derive(Deserialize)]
pub struct AssetData {
    pub name: String,
    pub md5: String,
    #[serde(rename = "pid")]
    pub pack_id: Option<String>,
}

#[derive(Deserialize)]
pub struct PackData {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub ab_infos: Vec<AssetData>,
    pub pack_infos: Vec<PackData>,
}

impl AssetData {
    /// # Errors
    /// Returns Err if the HTTP response fetching fails in some way.
    pub async fn download(self, client: Client, version: String) -> Result<()> {
        let url = format!(
            "{BASE_URL}/assets/{version}/{}",
            self.name
                .replace(".ab", ".dat")
                .replace(".mp4", ".dat")
                .replace('/', "_")
        );
        let response = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        spawn_blocking(|| {
            if let Err(e) = extract(Cursor::new(response), Path::new(TARGET_PATH), false) {
                println!("{e}");
            }
        });

        println!("[SUCCESS] {}", self.name);

        Ok(())
    }
}

impl PackData {
    /// # Errors
    /// Returns Err if the HTTP response fetching fails in some way.
    pub async fn download(self, client: Client, version: String) -> Result<()> {
        let url = format!("{BASE_URL}/assets/{version}/{}.dat", self.name);
        let response = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;

        spawn_blocking(|| {
            if let Err(e) = extract(Cursor::new(response), Path::new(TARGET_PATH), false) {
                println!("{e}");
            }
        });

        println!("[SUCCESS] {}", self.name);

        Ok(())
    }
}

impl UpdateInfo {
    /// # Errors
    /// Returns Err if the HTTP response fetching fails in some way.
    pub async fn fetch_latest(client: &Client, url: String) -> Result<Self> {
        let response = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        let update_info: Self =
            serde_json::from_str(response.as_str()).expect("Failed to read response as UpdateInfo");
        Ok(update_info)
    }
}
