#![warn(clippy::all, clippy::pedantic)]

use arkdata::{Details, Version, BASE_URL};
use reqwest::Client;

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
