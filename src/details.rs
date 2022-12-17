use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct Version {
    #[serde(rename = "resVersion")]
    pub resource: String,
    #[serde(rename = "clientVersion")]
    client: String,
}

#[derive(Serialize, Deserialize)]
pub struct Details<'a> {
    #[serde(flatten)]
    pub version: Version,
    #[serde(skip)]
    path: &'a str,
}

impl Version {
    /// # Errors
    /// Returns Err if the HTTP response fetching fails in some way.
    pub async fn fetch_latest(client: &Client, url: String) -> Result<Self> {
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

impl Details<'_> {
    #[must_use]
    pub fn get(path: &str) -> Details {
        let file = File::open(path).expect("Failed to open details file");
        let mut details: Details =
            serde_json::from_reader(BufReader::new(file)).expect("Failed to deserialize details");
        details.path = path;
        details
    }

    pub fn save(self) {
        let file = File::create(self.path).expect("Failed to open details file");
        serde_json::to_writer(file, &self).expect("Failed to serialize details");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BASE_URL;
    use serde_json::json;
    use std::panic::catch_unwind;
    use uuid::Uuid;

    fn generate_path() -> String {
        format!("{}{}", Uuid::new_v4(), ".json")
    }

    #[tokio::test]
    async fn get_version() {
        let client = Client::new();
        Version::fetch_latest(&client, format!("{BASE_URL}/version"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn reject_invalid_url() {
        let client = Client::new();
        let version = Version::fetch_latest(&client, String::default()).await;
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
