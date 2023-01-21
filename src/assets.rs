use crate::{Cache, CONFIG, VERSION};
use again::RetryPolicy;
use ahash::HashMap;
use anyhow::Result;
use bytes::Bytes;
use crossbeam_channel::Sender;
use once_cell::sync::Lazy;
use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use std::{
    io::{Cursor, Read},
    path::PathBuf,
    time::Duration,
};
use tokio::task::spawn_blocking;
use zip::ZipArchive;

#[derive(Serialize, Deserialize)]
pub struct NameHashMapping {
    #[serde(flatten)]
    pub inner: HashMap<String, String>,
}

impl Cache for NameHashMapping {}

static RETRY_POLICY: Lazy<RetryPolicy> = Lazy::new(|| {
    RetryPolicy::exponential(Duration::from_secs(3))
        .with_max_retries(5)
        .with_jitter(true)
        .with_max_delay(Duration::from_secs(20))
});

pub struct AssetBundle {
    pub path: PathBuf,
    pub data: Bytes,
}

/// # Errors
/// Returns Err if the HTTP response fetching fails in some way.
/// # Panics
/// Panics in the following situations:
/// - Asset data cannot be unzipped
/// - Size of a file in the zip archive cannot be truncated to usize
/// - Data cannot be sent across channel
pub async fn download_asset(
    name: String,
    client: Client,
    sender: Sender<AssetBundle>,
) -> Result<()> {
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
        .retry_if(
            || async { client.get(&url).send().await?.error_for_status() },
            Error::is_timeout,
        )
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

            sender
                .send(AssetBundle {
                    path: CONFIG
                        .output_dir
                        .join(file.mangled_name())
                        .parent()
                        .unwrap()
                        .to_path_buf(),
                    data: buffer.into(),
                })
                .unwrap_or_else(|_| {
                    panic!("Failed to send data across channel while processing {name}")
                });
        }
    });

    Ok(())
}

#[derive(Deserialize)]
pub struct AssetData {
    pub name: String,
    pub md5: String,
    #[serde(rename = "pid")]
    pub pack_id: Option<String>,
}

#[derive(Deserialize)]
pub struct PackData {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub ab_infos: Vec<AssetData>,
    pub pack_infos: Vec<PackData>,
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
