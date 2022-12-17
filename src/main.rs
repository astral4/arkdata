#![warn(clippy::all, clippy::pedantic)]

use arkdata::{Details, Version};
use reqwest::Client;

const BASE_URL: &str = "https://ark-us-static-online.yo-star.com/assetbundle/official/Android";

#[tokio::main]
async fn main() {
    let mut details = Details::get("details.json");
    let client = Client::new();

    let data_version = Version::fetch_latest(client, format!("{BASE_URL}/version"))
        .await
        .expect("Failed to fetch version data");
    if details.version == data_version {
        return;
    }
    details.version = data_version;

    details.save();
}
