use config::{Config, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize)]
enum Server {
    US,
    CN,
}

#[derive(Deserialize)]
pub struct Settings {
    server: Server,
    #[serde(skip)]
    pub base_server_url: String,
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
        settings.base_server_url = String::from(match settings.server {
            Server::US => "https://ark-us-static-online.yo-star.com/assetbundle/official/Android",
            Server::CN => todo!(),
        });
        settings
    }
}
