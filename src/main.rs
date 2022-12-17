#![warn(clippy::all, clippy::pedantic)]

use arkdata::{Details, NameHashMapping, UpdateInfo, Version, BASE_URL};
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;

async fn do_something(_: String) {}

#[tokio::main]
async fn main() {
    let mut details = Details::get("details.json");
    let mut name_to_hash_mapping = NameHashMapping::get("hashes.json");
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

    asset_info
        .into_iter()
        .filter_map(|entry| {
            name_to_hash_mapping
                .map
                .get(&entry.name)
                .map_or(true, |hash| hash != &entry.md5)
                .then(|| {
                    name_to_hash_mapping
                        .map
                        .insert(entry.name.clone(), entry.md5);
                    entry.name
                })
        })
        .map(do_something)
        .collect::<FuturesUnordered<_>>()
        .collect::<()>()
        .await;

    details.save();
    name_to_hash_mapping.save();
}
