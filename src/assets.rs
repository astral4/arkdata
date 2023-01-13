use crate::{extract::extract, Cache, CONFIG, VERSION};
use ahash::HashMap;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use tokio::task::spawn_blocking;

#[derive(Serialize, Deserialize)]
pub struct NameHashMapping {
    #[serde(flatten)]
    pub inner: HashMap<String, String>,
}

impl Cache for NameHashMapping {}

/// # Errors
/// Returns Err if the HTTP response fetching fails in some way.
pub async fn download_asset(name: String, client: Client) -> Result<()> {
    let url = format!(
        "{}/assets/{}/{}.dat",
        CONFIG.server_url.base,
        VERSION.resource,
        name.replace(".ab", "")
            .replace(".mp4", "")
            .replace('/', "_")
            .replace('#', "__")
    );

    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    spawn_blocking(move || {
        extract(Cursor::new(response), &CONFIG.output_path, false)
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
    /// Returns Err if the HTTP response fails in some way, or the response cannot be deserialized as `UpdateInfo`.
    pub async fn fetch(client: &Client, url: &str) -> Result<Self> {
        Ok(client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}
