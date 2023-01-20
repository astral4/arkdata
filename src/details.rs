use crate::{settings::Server, Cache, CONFIG};
use ahash::HashMap;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Version {
    #[serde(rename = "resVersion")]
    pub resource: String,
    #[serde(rename = "clientVersion")]
    client: String,
}

#[derive(Serialize, Deserialize)]
pub struct Details {
    version: HashMap<Server, Version>,
}

impl Cache for Details {}

impl Details {
    #[must_use]
    pub fn get_version(&self) -> &Version {
        self.version
            .get(&CONFIG.server)
            .expect("Failed to get version data for server")
    }

    pub fn set_version(&mut self, version: Version) {
        self.version.insert(CONFIG.server, version);
    }
}

pub static VERSION: Lazy<Version> = Lazy::new(|| {
    let client = Client::builder()
        .https_only(true)
        .timeout(Duration::from_secs(10))
        .use_rustls_tls()
        .build()
        .expect("Failed to build reqwest Client");

    client
        .get(&CONFIG.server_url.version)
        .send()
        .expect("Failed to send request")
        .error_for_status()
        .expect("Failed to get a successful response from server")
        .json()
        .expect("Failed to get the response body")
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VERSION;
    use serde_json::json;
    use std::{fs::File, panic::catch_unwind};
    use uuid::Uuid;

    fn generate_path() -> String {
        format!("{}{}", Uuid::new_v4(), ".json")
    }

    #[tokio::test]
    #[allow(clippy::let_underscore_drop)]
    async fn get_version() {
        let _ = VERSION;
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
