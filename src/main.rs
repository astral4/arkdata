#![warn(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

use anyhow::Result;
use arkdata::{download_asset, Cache, NameHashMapping, UpdateInfo, Version, CONFIG, VERSION};
use futures::{future::join_all, Future};
use reqwest::Client;
use std::fs;
use tap::Pipe;

fn log_errors<T>(results: impl IntoIterator<Item = Result<T>>) {
    results
        .into_iter()
        .filter_map(Result::err)
        .for_each(|err| println!("{err}"));
}

async fn join_parallel<T: Send + 'static>(
    futs: impl IntoIterator<Item = impl Future<Output = T> + Send + 'static>,
) -> Vec<T> {
    let tasks: Vec<_> = futs.into_iter().map(tokio::spawn).collect();
    // unwrap the Result because it is introduced by tokio::spawn()
    // and isn't something our caller can handle
    join_all(tasks)
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect()
}

#[tokio::main]
async fn main() {
    let details = Version::load(&CONFIG.details_path);
    let name_to_hash_mapping = NameHashMapping::load(&CONFIG.hashes_path);
    let client = Client::builder()
        .https_only(true)
        .use_rustls_tls()
        .build()
        .expect("Failed to build reqwest Client");

    if !CONFIG.force_fetch && *details.get() == *VERSION {
        return;
    }

    let asset_info = {
        UpdateInfo::fetch(
            &client,
            format!(
                "{}/assets/{}/hot_update_list.json",
                CONFIG.server_url.base, VERSION.resource
            )
            .as_str(),
        )
        .await
        .expect("Failed to fetch asset info list")
    };

    if !CONFIG.output_dir.is_dir() {
        fs::create_dir(&CONFIG.output_dir).expect("Failed to create output directory");
    }

    if name_to_hash_mapping.inner.is_empty() {
        // No assets have been downloaded before
        // Download asset packs
        asset_info
            .pack_infos
            .into_iter()
            .map(|pack| download_asset(pack.name, client.clone()))
            .pipe(join_parallel)
            .await
            .pipe(log_errors);

        // Some assets do not have a pack ID, so they need to be fetched separately
        asset_info
            .ab_infos
            .iter()
            .filter_map(|entry| {
                entry
                    .pack_id
                    .is_none()
                    .then(|| download_asset(entry.name.clone(), client.clone()))
            })
            .pipe(join_parallel)
            .await
            .pipe(log_errors);
    } else {
        // Update collection of existing assets
        asset_info
            .ab_infos
            .iter()
            .filter_map(|entry| {
                name_to_hash_mapping
                    .inner
                    .get(&entry.name)
                    .map_or(true, |hash| CONFIG.force_fetch || hash != &entry.md5)
                    .then(|| download_asset(entry.name.clone(), client.clone()))
            })
            .pipe(join_parallel)
            .await
            .pipe(log_errors);
    }

    if CONFIG.update_cache {
        let mut details = details;
        details.set(VERSION.clone());
        details.save(&CONFIG.details_path);

        let mut name_to_hash_mapping = name_to_hash_mapping;

        name_to_hash_mapping
            .inner
            .extend(asset_info.ab_infos.into_iter().filter_map(|entry| {
                CONFIG
                    .output_dir
                    .join(&entry.name)
                    .is_file()
                    .then_some((entry.name, entry.md5))
            }));

        name_to_hash_mapping.save(&CONFIG.hashes_path);
    }
}
