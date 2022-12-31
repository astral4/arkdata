use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize)]
enum Server {
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
    server: Server,
    #[serde(skip)]
    pub server_url: ServerLink,
    pub details_path: String,
    pub hashes_path: String,
    pub force_fetch: bool,
    pub path_start_patterns: Option<Vec<String>>,
    pub output_dir: String,
    pub update_cache: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SettingsWrapper {
    fetch_settings: Settings,
}

impl Settings {
    pub fn get() -> Self {
        let config = Config::builder()
            .add_source(File::new("config.toml", FileFormat::Toml))
            .build()
            .expect("Failed to read configuration");
        let mut settings = config
            .try_deserialize::<SettingsWrapper>()
            .expect("Failed to deserialize configuration file")
            .fetch_settings;
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
    }
}
