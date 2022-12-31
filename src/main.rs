#![warn(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

use anyhow::Result;
use arkdata::{download_asset, Cache, Details, NameHashMapping, UpdateInfo, Version, CONFIG};
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
    let mut details = Details::get(&CONFIG.details_path);
    let mut name_to_hash_mapping = NameHashMapping::get(&CONFIG.hashes_path);
    let client = Client::new();

    let data_version = Version::fetch_latest(&client, CONFIG.server_url.version.as_str())
        .await
        .expect("Failed to fetch version data");
    if !CONFIG.force_fetch && details.version == data_version {
        return;
    }
    details.version = data_version;

    let asset_info = UpdateInfo::fetch_latest(
        &client,
        format!(
            "{}/assets/{}/hot_update_list.json",
            CONFIG.server_url.base, details.version.resource
        ),
    )
    .await
    .expect("Failed to fetch asset info list");

    let target_path = Path::new(&CONFIG.output_dir);

    if !target_path.is_dir() {
        fs::create_dir(target_path).expect("Failed to create missing target directory");
    }

    if fs::read_dir(target_path)
        .expect("Failed to read contents of target directory")
        .peekable()
        .peek()
        .is_none()
        && name_to_hash_mapping.inner.is_empty()
    {
        // No assets have been downloaded before
        // Download asset packs
        join_parallel(asset_info.pack_infos.into_iter().map(|pack| {
            download_asset(pack.name, client.clone(), details.version.resource.clone())
        }))
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
                None => Some(download_asset(
                    entry.name,
                    client.clone(),
                    details.version.resource.clone(),
                )),
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
                .filter(|entry| {
                    CONFIG.path_start_patterns.as_ref().map_or(true, |pats| {
                        pats.iter().any(|pat| entry.name.starts_with(pat))
                    })
                })
                .filter_map(|entry| {
                    name_to_hash_mapping
                        .inner
                        .get(&entry.name)
                        .map_or(true, |hash| CONFIG.force_fetch || hash != &entry.md5)
                        .then(|| {
                            name_to_hash_mapping
                                .inner
                                .insert(entry.name.clone(), entry.md5.clone());
                            entry
                        })
                })
                .map(|entry| {
                    download_asset(entry.name, client.clone(), details.version.resource.clone())
                }),
        )
        .await
        .into_iter()
        .filter_map(std::result::Result::err)
        .for_each(|err| println!("{err}"));
    }

    if CONFIG.update_cache {
        details.save(&CONFIG.details_path);
        name_to_hash_mapping.save(&CONFIG.hashes_path);
    }
}
