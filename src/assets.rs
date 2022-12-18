use ahash::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize)]
pub struct NameHashMapping<'a> {
    #[serde(flatten)]
    pub inner: HashMap<String, String>,
    #[serde(skip)]
    path: &'a str,
}

impl NameHashMapping<'_> {
    #[must_use]
    pub fn get(path: &str) -> NameHashMapping {
        let file = File::open(path).expect("Failed to open name-to-hash file");
        let mut mapping: NameHashMapping = serde_json::from_reader(BufReader::new(file))
            .expect("Failed to deserialize name-hash mapping");
        mapping.path = path;
        mapping
    }

    pub fn save(self) {
        let file = File::create(self.path).expect("Failed to open name-to-hash file");
        serde_json::to_writer(file, &self).expect("Failed to serialize name-hash mapping");
    }
}

#[derive(Deserialize)]
pub struct AssetData {
    pub name: String,
    pub md5: String,
    #[serde(rename = "pid")]
    pub pack_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub ab_infos: Vec<AssetData>,
    pub pack_infos: Vec<AssetData>,
}

impl UpdateInfo {
    /// # Errors
    /// Returns Err if the HTTP response fetching fails in some way.
    pub async fn fetch_latest(client: &reqwest::Client, url: String) -> Result<Self> {
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

pub enum AssetType {
    Asset,
    Pack,
}
