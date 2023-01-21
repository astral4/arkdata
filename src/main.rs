#![warn(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

use arkdata::{
    fetch_all, AssetBundle, Cache, NameHashMapping, UpdateInfo, Version, CONFIG, VERSION,
};
use crossbeam_channel::unbounded;
use pyo3::{types::PyBytes, Python};
use rayon::iter::{ParallelBridge, ParallelIterator};
use reqwest::Client;
use std::{fs, thread};

#[tokio::main]
async fn main() {
    let version = Version::load(&CONFIG.details_path);
    let mut name_to_hash_mapping = NameHashMapping::load(&CONFIG.hashes_path);
    let client = Client::builder()
        .https_only(true)
        .use_rustls_tls()
        .build()
        .expect("Failed to build reqwest Client");

    if !CONFIG.force_fetch && *version.get() == *VERSION {
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

    if CONFIG.update_cache {
        let mut version = version;
        version.set(VERSION.clone());
        version.save(&CONFIG.details_path);

        name_to_hash_mapping.update(&asset_info);
        name_to_hash_mapping.save(&CONFIG.hashes_path);
    }

    if !CONFIG.output_dir.is_dir() {
        fs::create_dir_all(&CONFIG.output_dir).expect("Failed to create output directory");
    }

    let (sender, receiver) = unbounded::<AssetBundle>();

    let thread_handle = thread::spawn(|| {
        receiver.into_iter().par_bridge().for_each(|bundle| {
            Python::with_gil(|py| {
                let extract = py.import("kawapack").unwrap().getattr("extract").unwrap();
                let data = PyBytes::new_with(py, bundle.data.len(), |bytes| {
                    bytes.copy_from_slice(bundle.data.as_ref());
                    Ok(())
                })
                .unwrap();

                extract
                    .call1((data, bundle.path, &CONFIG.output_dir))
                    .unwrap();
            });
        });
    });

    fetch_all(&name_to_hash_mapping, asset_info, client, sender).await;

    thread_handle.join().unwrap();
}
