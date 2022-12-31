use crate::Cache;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct Version {
    #[serde(rename = "resVersion")]
    pub resource: String,
    #[serde(rename = "clientVersion")]
    client: String,
}

#[derive(Serialize, Deserialize)]
pub struct Details {
    #[serde(flatten)]
    pub version: Version,
}

impl Version {
    /// # Errors
    /// Returns Err if the HTTP response fetching fails in some way.
    pub async fn fetch_latest(client: &Client, url: &str) -> Result<Self> {
        let response = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let version: Self =
            serde_json::from_str(response.as_str()).expect("Failed to read response as Version");
        Ok(version)
    }
}

impl Cache for Details {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CONFIG;
    use serde_json::json;
    use std::{fs::File, panic::catch_unwind};
    use uuid::Uuid;

    fn generate_path() -> String {
        format!("{}{}", Uuid::new_v4(), ".json")
    }

    #[tokio::test]
    async fn get_version() {
        let client = Client::new();
        Version::fetch_latest(&client, &CONFIG.server_url.version)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn reject_invalid_url() {
        let client = Client::new();
        let version = Version::fetch_latest(&client, "").await;
        assert!(version.is_err());
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn panic_on_nonexistent_file() {
        Details::get(generate_path().as_str());
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn panic_on_invalid_deserializing() {
        let path = generate_path();
        let res = catch_unwind(|| {
            if let Ok(file) = File::create(&path) {
                if serde_json::to_writer(file, &json!("{}")).is_ok() {
                    Details::get(path.as_str());
                }
            }
        });
        std::fs::remove_file(path);
        res.unwrap();
    }
}
