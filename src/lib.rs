#![warn(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

mod assets;
mod details;
mod extract;
mod settings;
pub use assets::{download_asset, NameHashMapping, UpdateInfo};
pub use details::*;
pub use settings::CONFIG;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{any::type_name, fs::File, io::BufReader, marker::Sized};

pub trait Cache {
    #[must_use]
    fn get(path: &str) -> Self
    where
        for<'de> Self: Sized + Deserialize<'de>,
    {
        let file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {path}"));

        serde_json::from_reader(BufReader::new(file))
            .unwrap_or_else(|_| panic!("Failed to deserialize from {path}"))
    }

    fn save(&self, path: &str)
    where
        for<'a> Self: Serialize,
    {
        let file = File::create(path).unwrap_or_else(|_| panic!("Failed to open {path}"));
        serde_json::to_writer_pretty(file, &self)
            .unwrap_or_else(|_| panic!("Failed to serialize to {path}"));
    }
}

#[async_trait]
pub trait Fetch {
    async fn fetch(client: &Client, url: &str) -> Result<Self>
    where
        for<'de> Self: Sized + Deserialize<'de>,
    {
        let response = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let update_info: Self = serde_json::from_str(response.as_str())
            .unwrap_or_else(|_| panic!("Failed to read response as {}", type_name::<Self>()));

        Ok(update_info)
    }
}
