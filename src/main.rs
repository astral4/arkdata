#![warn(clippy::all, clippy::pedantic)]

use arkdata::{Details, Version};
use reqwest::Client;

const BASE_URL: &str = "https://ark-us-static-online.yo-star.com/assetbundle/official/Android";

#[tokio::main]
async fn main() {
    let mut details = Details::get("details.json");
    let client = Client::new();

    let res = client
        .get(format!("{BASE_URL}/version"))
        .send()
        .await
        .expect("Failed to fetch latest version data")
        .error_for_status()
        .expect("The data server returned an error")
        .text()
        .await
        .expect("Failed to read response as text");

    let data_version: Version =
        serde_json::from_str(res.as_str()).expect("Failed to deserialize text as Version");

    details.version = data_version;

    details.save();
}
