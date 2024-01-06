use crate::{Cache, CONFIG, VERSION};
use again::RetryPolicy;
use ahash::HashMap;
use anyhow::Result;
use futures::{future::join_all, Future};
use once_cell::sync::Lazy;
use pyo3::{types::PyBytes, Python};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    io::{Cursor, Read},
    sync::Arc,
    time::Duration,
};
use tap::Pipe;
use tokio::task::spawn_blocking;
use zip::ZipArchive;

fn is_in_whitelist(test: &str) -> bool {
    CONFIG
        .path_whitelist
        .as_ref()
        .map_or(true, |list| list.iter().any(|p| test.contains(p)))
}

#[derive(Serialize, Deserialize)]
pub struct NameHashMapping {
    #[serde(flatten)]
    inner: HashMap<Arc<str>, Arc<str>>,
}

impl Cache for NameHashMapping {}

impl NameHashMapping {
    pub fn set(&mut self, data: &UpdateInfo) {
        self.inner = data
            .ab_infos
            .iter()
            .filter(|asset| is_in_whitelist(&asset.name))
            .map(|asset| (asset.name.clone(), asset.md5.clone()))
            .collect();
    }
}

static RETRY_POLICY: Lazy<RetryPolicy> = Lazy::new(|| {
    RetryPolicy::exponential(Duration::from_secs(3))
        .with_max_retries(5)
        .with_jitter(true)
        .with_max_delay(Duration::from_secs(20))
});

#[derive(Deserialize)]
struct AssetData {
    name: Arc<str>,
    md5: Arc<str>,
    #[serde(rename = "pid")]
    pack_id: Option<String>,
}

#[derive(Deserialize)]
struct PackData {
    name: Arc<str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    ab_infos: Vec<AssetData>,
    pack_infos: Vec<PackData>,
}

impl UpdateInfo {
    /// # Errors
    /// Returns Err if the HTTP response fails in some way, or the response cannot be deserialized as `UpdateInfo`.
    pub async fn fetch(client: &Client, url: &str) -> Result<Self> {
        Ok(client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

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

async fn download_asset(name: Arc<str>, client: Client) -> Result<()> {
    let url = format!(
        "{}/assets/{}/{}.dat",
        CONFIG.server_url.base,
        VERSION.resource,
        name.replace(".ab", "")
            .replace(".mp4", "")
            .replace('/', "_")
            .replace('#', "__")
    );

    let response = RETRY_POLICY
        .retry(|| async { client.get(&url).send().await?.error_for_status() })
        .await?
        .bytes()
        .await?;

    spawn_blocking(move || {
        let mut archive = ZipArchive::new(Cursor::new(response))
            .unwrap_or_else(|_| panic!("Failed to create zip archive from response at {name}"));

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap_or_else(|_| {
                panic!("Failed to read zip file at index {i} in archive at {name}")
            });

            let mut buffer = Vec::with_capacity(
                file.size()
                    .try_into()
                    .expect("File size as u64 could not be truncated to usize"),
            );

            file.read_to_end(&mut buffer).unwrap();

            Python::with_gil(|py| {
                let extract = py.import("kawapack").unwrap().getattr("extract").unwrap();
                let data = PyBytes::new_with(py, buffer.len(), |bytes| {
                    bytes.copy_from_slice(&buffer);
                    Ok(())
                })
                .unwrap();

                extract
                    .call1((
                        data,
                        file.mangled_name().parent().unwrap().to_path_buf(),
                        &CONFIG.output_dir,
                    ))
                    .unwrap();
            });
        }
    });

    Ok(())
}

pub async fn fetch_all(hashes: &NameHashMapping, asset_info: &UpdateInfo, client: &Client) {
    if hashes.inner.is_empty() && CONFIG.path_whitelist.is_none() {
        // No assets have been downloaded before
        // Download asset packs
        asset_info
            .pack_infos
            .iter()
            .map(|pack| download_asset(pack.name.clone(), client.clone()))
            .pipe(join_parallel)
            .await
            .pipe(log_errors);

        // Some assets do not have a pack ID, so they need to be fetched separately
        asset_info
            .ab_infos
            .iter()
            .filter(|entry| entry.pack_id.is_none() && is_in_whitelist(&entry.name))
            .map(|entry| download_asset(entry.name.clone(), client.clone()))
            .pipe(join_parallel)
            .await
            .pipe(log_errors);
    } else {
        // Update collection of existing assets
        asset_info
            .ab_infos
            .iter()
            .filter(|entry| {
                is_in_whitelist(&entry.name)
                    && hashes
                        .inner
                        .get(&entry.name)
                        .map_or(true, |hash| hash != &entry.md5)
            })
            .map(|entry| download_asset(entry.name.clone(), client.clone()))
            .pipe(join_parallel)
            .await
            .pipe(log_errors);
    }
}
