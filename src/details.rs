use crate::{settings::Server, Cache, CONFIG};
use foldhash::HashMap;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{cmp::max, sync::LazyLock, thread::sleep, time::Duration};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct VersionInner {
    #[serde(alias = "resVersion")]
    pub resource: String,
    #[serde(alias = "clientVersion")]
    client: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Version {
    #[serde(flatten)]
    version: HashMap<Server, VersionInner>,
}

impl Cache for Version {}

impl Version {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn get(&self) -> &VersionInner {
        self.version
            .get(&CONFIG.server)
            .expect("Failed to get version data for server")
    }

    pub fn set(&mut self, version: VersionInner) {
        self.version.insert(CONFIG.server, version);
    }
}

pub static VERSION: LazyLock<VersionInner> = LazyLock::new(|| {
    let client = Client::builder()
        .https_only(true)
        .timeout(Duration::from_secs(10))
        .use_rustls_tls()
        .build()
        .expect("Failed to build reqwest Client");

    for idx in 0..5 {
        match (|| {
            client
                .get(&CONFIG.server_url.version)
                .send()?
                .error_for_status()?
                .json()
        })() {
            Ok(version) => return version,
            Err(e) => println!("{e}"),
        }
        sleep(Duration::from_secs(max(3u64.pow(idx + 1), 20)));
    }
    panic!("Failed to fetch resource version from server");
});

#[cfg(test)]
#[allow(clippy::should_panic_without_expect)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        fs::{remove_file, File},
        panic::catch_unwind,
    };
    use uuid::Uuid;

    fn generate_path() -> String {
        format!("{}{}", Uuid::new_v4(), ".json")
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn panic_on_nonexistent_file() {
        Version::load(generate_path().as_str());
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn panic_on_invalid_deserializing() {
        let path = generate_path();
        let res = catch_unwind(|| {
            if let Ok(file) = File::create(&path) {
                if serde_json::to_writer(file, &json!("{}")).is_ok() {
                    Version::load(path.as_str());
                }
            }
        });
        remove_file(path);
        res.unwrap();
    }
}
