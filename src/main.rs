#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use arkdata::{Details, NameHashMapping, UpdateInfo, Version, BASE_URL};
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;
use std::io::Cursor;
use std::path::Path;

async fn download_asset(client: &Client, version: &String, name: String) -> Result<()> {
    let url = format!(
        "{BASE_URL}/assets/{version}/{}",
        name.replace(".ab", ".dat").replace('/', "_")
    );
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    zip_extract::extract(Cursor::new(response), Path::new("assets"), false)?;

    println!("[SUCCESS] {name}"); // debugging

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut details = Details::get("details.json");
    let /*mut*/ name_to_hash_mapping = NameHashMapping::get("hashes.json");
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

    let res = asset_info
        .into_iter()
        /*
        .filter_map(|entry| {
            name_to_hash_mapping
               .inner
                .get(&entry.name)
                .map_or(true, |hash| hash != &entry.md5)
                .then(|| {
                    name_to_hash_mapping
                        .inner
                        .insert(entry.name.clone(), entry.md5);
                    entry.name
                })
        })
        */
        .map(|entry| entry.name) // temporary replacement
        .map(|name| download_asset(&client, &details.version.resource, name))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<()>>>()
        .await
        .into_iter()
        .filter_map(std::result::Result::err);

    for error in res {
        println!("{error}"); // debugging
    }

    details.save();
    name_to_hash_mapping.save();
}
