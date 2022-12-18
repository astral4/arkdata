#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use arkdata::{AssetType, Details, NameHashMapping, UpdateInfo, Version, BASE_URL};
use futures::{stream::FuturesUnordered, StreamExt};
use reqwest::Client;
use std::{fs, io::Cursor, path::Path};
use tokio::task::spawn_blocking;

const TARGET_PATH: &str = "assets";

async fn download_asset(
    client: &Client,
    version: &String,
    asset_type: AssetType,
    name: String,
) -> Result<()> {
    let url = match asset_type {
        AssetType::Asset => format!(
            "{BASE_URL}/assets/{version}/{}",
            name.replace(".ab", ".dat").replace('/', "_")
        ),
        AssetType::Pack => format!("{BASE_URL}/assets/{version}/{name}.dat"),
    };
    let response = client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;

    spawn_blocking(|| zip_extract::extract(Cursor::new(response), Path::new(TARGET_PATH), false));

    println!("[SUCCESS] {name}");

    Ok(())
}

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
    .expect("Failed to fetch asset info list");

    let target_path = Path::new(TARGET_PATH);

    if !target_path.is_dir() {
        fs::create_dir(target_path).expect("Failed to create missing target directory");
    }

    if fs::read_dir(target_path)
        .expect("Failed to read contents of target directory")
        .peekable()
        .peek()
        .is_none()
    {
        // No assets have been downloaded before
        // Download asset packs
        asset_info
            .pack_infos
            .into_iter()
            .map(|pack| {
                download_asset(
                    &client,
                    &details.version.resource,
                    AssetType::Pack,
                    pack.name,
                )
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<Result<()>>>()
            .await
            .into_iter()
            .filter_map(std::result::Result::err)
            .for_each(|err| println!("{err}"));

        // Some assets do not have a pack ID, so they need to be fetched separately
        asset_info
            .ab_infos
            .into_iter()
            .filter_map(|entry| {
                name_to_hash_mapping
                    .inner
                    .insert(entry.name.clone(), entry.md5);
                match entry.pack_id {
                    Some(_) => None,
                    None => Some(download_asset(
                        &client,
                        &details.version.resource,
                        AssetType::Asset,
                        entry.name,
                    )),
                }
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<Result<()>>>()
            .await
            .into_iter()
            .filter_map(std::result::Result::err)
            .for_each(|err| println!("{err}"));
    } else {
        // Update collection of existing assets
        asset_info
            .ab_infos
            .into_iter()
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
            .map(|name| download_asset(&client, &details.version.resource, AssetType::Asset, name))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<Result<()>>>()
            .await
            .into_iter()
            .filter_map(std::result::Result::err)
            .for_each(|err| println!("{err}"));
    }

    details.save();
    name_to_hash_mapping.save();
}
