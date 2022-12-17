#![warn(clippy::all, clippy::pedantic)]

use arkdata::{Details, NameHashMapping, UpdateInfo, Version, BASE_URL};
use reqwest::Client;

#[tokio::main]
async fn main() {
    let mut details = Details::get("details.json");
    let mut name_to_hash_mapping = NameHashMapping::get("hashes.json").map;
    let client = Client::new();

    let data_version = Version::fetch_latest(&client, format!("{BASE_URL}/version"))
        .await
        .expect("Failed to fetch version data");
    // if details.version == data_version {
    //     return;
    // }
    details.version = data_version;

    let asset_info = UpdateInfo::fetch_latest(
        &client,
        format!(
            "{BASE_URL}/assets/{}/hot_update_list.json",
            details.version.resource
        ),
    )
    .await
    .expect("Failed to fetch asset info list")
    .ab_infos;

    let mut new_assets = Vec::<String>::with_capacity(asset_info.len());

    for entry in asset_info {
        if name_to_hash_mapping
            .get(entry.name.as_ref())
            .map_or(true, |hash| hash != &entry.md5)
        {
            name_to_hash_mapping.insert(entry.name.to_string(), entry.md5);
            new_assets.push(entry.name.into_owned());
        }
    }

    details.save();
}
