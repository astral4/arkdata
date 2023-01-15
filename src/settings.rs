use config::{Config, File, FileFormat};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{env, path::PathBuf, str::FromStr};

#[derive(Deserialize)]
enum Server {
    US,
    CN,
}

impl FromStr for Server {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "US" => Ok(Self::US),
            "CN" => Ok(Self::CN),
            _ => Err(()),
        }
    }
}

#[derive(Default)]
pub struct ServerLink {
    pub version: String,
    pub base: String,
}

#[derive(Deserialize)]
pub struct Settings {
    server: Option<Server>,
    #[serde(skip)]
    pub server_url: ServerLink,
    pub details_path: String,
    pub hashes_path: String,
    pub force_fetch: bool,
    output_dir: String,
    #[serde(skip)]
    pub output_path: PathBuf,
    pub update_cache: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SettingsWrapper {
    fetch_settings: Settings,
}

pub static CONFIG: Lazy<Settings> = Lazy::new(|| {
    let config = Config::builder()
        .add_source(File::new("config.toml", FileFormat::Toml))
        .build()
        .expect("Failed to read configuration");

    let mut settings = config
        .try_deserialize::<SettingsWrapper>()
        .expect("Failed to deserialize configuration file")
        .fetch_settings;

    // If a server is supplied as an environmental variable,
    // it takes precedence over the server specified in config.toml
    if let Some(server) = env::args().nth(1) {
        settings.server = Some(
            Server::from_str(&server).expect("Failed to parse server from environment variable"),
        )
    }

    if let Some(server) = &settings.server {
        settings.server_url = match server {
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
        }
    } else {
        panic!("No server was provided")
    }

    settings.output_path = PathBuf::from(&settings.output_dir);

    settings
});
