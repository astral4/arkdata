use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{fs::read, path::PathBuf};

#[derive(Deserialize, Serialize, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Server {
    US,
    CN,
}

#[derive(Default)]
pub struct ServerLink {
    pub version: String,
    pub base: String,
}

#[derive(Deserialize)]
pub struct Settings {
    pub server: Server,
    #[serde(skip)]
    pub server_url: ServerLink,
    pub versions_path: String,
    pub hashes_path: String,
    pub force_fetch: bool,
    pub output_dir: PathBuf,
    pub update_cache: bool,
}

pub static CONFIG: Lazy<Settings> = Lazy::new(|| {
    let mut settings: Settings =
        toml::from_slice(&read("config.toml").expect("Failed to read configuration file"))
            .expect("Failed to deserialize configuration file");

    settings.server_url = match settings.server {
        Server::US => ServerLink {
            version: String::from(
                "https://ark-us-static-online.yo-star.com/assetbundle/official/Android/version",
            ),
            base: String::from(
                "https://ark-us-static-online.yo-star.com/assetbundle/official/Android",
            ),
        },
        Server::CN => ServerLink {
            version: String::from(
                "https://ak-conf.hypergryph.com/config/prod/official/Android/version",
            ),
            base: String::from("https://ak.hycdn.cn/assetbundle/official/Android"),
        },
    };

    settings
});
