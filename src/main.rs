#![warn(clippy::all, clippy::pedantic)]

// TODO: Find a better way to satisfy the borrow checker than cloning Strings and Clients.

use anyhow::Result;
use arkdata::{Details, NameHashMapping, UpdateInfo, Version, BASE_URL, TARGET_PATH};
use futures::Future;
use reqwest::Client;
use std::{fs, path::Path};

pub async fn join_parallel<T: Send + 'static>(
    futs: impl IntoIterator<Item = impl Future<Output = T> + Send + 'static>,
) -> Vec<T> {
    let tasks: Vec<_> = futs.into_iter().map(tokio::spawn).collect();
    // unwrap the Result because it is introduced by tokio::spawn()
    // and isn't something our caller can handle
    futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect()
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
        join_parallel(
            asset_info
                .pack_infos
                .into_iter()
                .map(|pack| pack.download(client.clone(), details.version.resource.clone())),
        )
        .await
        .into_iter()
        .filter_map(std::result::Result::err)
        .for_each(|err| println!("{err}"));

        // Some assets do not have a pack ID, so they need to be fetched separately
        join_parallel(asset_info.ab_infos.into_iter().filter_map(|entry| {
            name_to_hash_mapping
                .inner
                .insert(entry.name.clone(), entry.md5.clone());
            match entry.pack_id {
                Some(_) => None,
                None => Some(entry.download(client.clone(), details.version.resource.clone())),
            }
        }))
        .await
        .into_iter()
        .filter_map(std::result::Result::err)
        .for_each(|err| println!("{err}"));
    } else {
        // Update collection of existing assets
        join_parallel(
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
                                .insert(entry.name.clone(), entry.md5.clone());
                            entry
                        })
                })
                .map(|entry| entry.download(client.clone(), details.version.resource.clone())),
        )
        .await
        .into_iter()
        .filter_map(std::result::Result::err)
        .for_each(|err| println!("{err}"));
    }

    details.save();
    name_to_hash_mapping.save();
}
