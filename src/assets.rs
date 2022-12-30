use crate::{extract, Cache, CONFIG};
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

/// # Errors
/// Returns Err if the HTTP response fetching fails in some way.
pub async fn download_asset(name: String, client: Client, version: String) -> Result<()> {
    let url = format!(
        "{}/assets/{version}/{}.dat",
        CONFIG.base_server_url,
        name.replace(".ab", "")
            .replace(".mp4", "")
            .replace('/', "_")
    );
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    spawn_blocking(move || {
        extract(Cursor::new(response), Path::new(&CONFIG.output_dir), false)
            .map_or_else(|err| println!("{err}"), |_| println!("[SUCCESS] {name}"));
    });

    Ok(())
}

#[derive(Deserialize)]
pub struct AssetData {
    pub name: String,
    pub md5: String,
    #[serde(rename = "pid")]
    pub pack_id: Option<String>,
}

#[derive(Deserialize)]
pub struct PackData {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub ab_infos: Vec<AssetData>,
    pub pack_infos: Vec<PackData>,
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
