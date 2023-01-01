use crate::{Cache, CONFIG};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
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

impl Cache for Details {}

pub static VERSION: Lazy<Version> = Lazy::new(|| {
    let response = reqwest::blocking::get(&CONFIG.server_url.version)
        .expect("Failed to send request")
        .error_for_status()
        .expect("Failed to get a successful response from server")
        .text()
        .expect("Failed to get the response body");
    serde_json::from_str(&response).expect("Failed to deserialize response as Version")
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
