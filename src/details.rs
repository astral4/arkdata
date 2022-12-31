use crate::{Cache, Fetch};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq)]
pub struct Version {
    #[serde(rename = "resVersion")]
    pub resource: String,
    #[serde(rename = "clientVersion")]
    client: String,
}

impl Fetch for Version {}

#[derive(Serialize, Deserialize)]
pub struct Details {
    #[serde(flatten)]
    pub version: Version,
}

impl Cache for Details {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Fetch, CONFIG};
    use reqwest::Client;
    use serde_json::json;
    use std::{fs::File, panic::catch_unwind};
    use uuid::Uuid;

    fn generate_path() -> String {
        format!("{}{}", Uuid::new_v4(), ".json")
    }

    #[tokio::test]
    async fn get_version() {
        let client = Client::new();
        Version::fetch(&client, CONFIG.server_url.version.as_str())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn reject_invalid_url() {
        let client = Client::new();
        let version = Version::fetch(&client, "").await;
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
