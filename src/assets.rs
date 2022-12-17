use ahash::HashMap;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize)]
struct NameHashMapping {
    #[serde(flatten)]
    map: HashMap<String, String>,
}

impl NameHashMapping {
    fn get(path: &str) -> Self {
        let file = File::open(path).expect("Failed to open name-to-hash file");
        serde_json::from_reader(BufReader::new(file))
            .expect("Failed to deserialize name-to-hash data")
    }
}

impl From<Vec<AssetData>> for NameHashMapping {
    fn from(value: Vec<AssetData>) -> Self {
        todo!()
    }
}

#[derive(Deserialize)]
// #[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
pub struct AssetData {
    name: String,
    // hash: String,
    md5: String,
    // pid: Option<String>,
    // #[serde(rename = "type")]
    // asset_type: Option<String>,
    // total_size: u32,
    // ab_size: u32,
    // cid: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
// #[serde(deny_unknown_fields)]
pub struct UpdateInfo {
    // full_pack: FullPack,
    // version_id: String,
    pub ab_infos: Vec<AssetData>,
    // count_of_typed_res: u32,
    // pack_infos: Vec<AssetData>,
}

pub fn bar() {
    let baz = HashMap::<String, String>::default();
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
