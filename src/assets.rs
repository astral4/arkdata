use ahash::HashMap;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize)]
pub struct NameHashMapping {
    #[serde(flatten)]
    pub map: HashMap<String, String>,
}

impl NameHashMapping {
    #[must_use]
    pub fn get(path: &str) -> Self {
        let file = File::open(path).expect("Failed to open name-to-hash file");
        serde_json::from_reader(BufReader::new(file))
            .expect("Failed to deserialize name-to-hash data")
    }
}

impl From<Vec<AssetData>> for NameHashMapping {
    fn from(_value: Vec<AssetData>) -> Self {
        todo!()
    }
}

#[derive(Deserialize)]
pub struct AssetData {
    pub name: String,
    pub md5: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub ab_infos: Vec<AssetData>,
}

impl UpdateInfo {
    /// # Errors
    /// Returns Err if the HTTP response fetching fails in some way.
    pub async fn fetch_latest(
        client: &reqwest::Client,
        url: String,
    ) -> Result<Self, reqwest::Error> {
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
