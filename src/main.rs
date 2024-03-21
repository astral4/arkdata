#![warn(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

use arkdata::{
    combine_textures, fetch_all, process_portraits, Cache, NameHashMapping, UpdateInfo, Version,
    CONFIG, VERSION,
};
use reqwest::Client;
use std::fs::create_dir_all;

#[tokio::main]
async fn main() {
    let version = Version::load(&CONFIG.versions_path);
    let mut name_to_hash_mapping = NameHashMapping::load(&CONFIG.hashes_path);
    let client = Client::builder()
        .http2_prior_knowledge()
        .https_only(true)
        .use_rustls_tls()
        .build()
        .expect("Failed to build reqwest Client");

    if version.get() == &*VERSION {
        return;
    }

    let asset_info = {
        UpdateInfo::fetch(
            &client,
            &format!(
                "{}/assets/{}/hot_update_list.json",
                CONFIG.server_url.base, VERSION.resource
            ),
        )
        .await
        .expect("Failed to fetch asset info list")
    };

    if !CONFIG.output_dir.is_dir() {
        create_dir_all(&CONFIG.output_dir).expect("Failed to create output directory");
    }

    fetch_all(&name_to_hash_mapping, &asset_info, &client).await;

    if CONFIG.update_cache {
        let mut version = version;
        version.set(VERSION.clone());
        version.save(&CONFIG.versions_path);

        name_to_hash_mapping.set(&asset_info);
        name_to_hash_mapping.save(&CONFIG.hashes_path);
    }

    combine_textures();
    process_portraits();
}
