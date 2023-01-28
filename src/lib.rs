#![warn(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

mod assets;
mod details;
mod imageproc;
mod settings;
pub use assets::{fetch_all, AssetBundle, NameHashMapping, UpdateInfo};
pub use details::{Version, VERSION};
pub use imageproc::{combine_textures, process_portraits};
pub use settings::CONFIG;

use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, marker::Sized};

pub trait Cache {
    #[must_use]
    fn load(path: &str) -> Self
    where
        for<'de> Self: Sized + Deserialize<'de>,
    {
        let file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {path}"));

        serde_json::from_reader(BufReader::new(file))
            .unwrap_or_else(|_| panic!("Failed to deserialize from {path}"))
    }

    fn save(&self, path: &str)
    where
        Self: Serialize,
    {
        let file = File::create(path).unwrap_or_else(|_| panic!("Failed to open {path}"));

        serde_json::to_writer_pretty(file, &self)
            .unwrap_or_else(|_| panic!("Failed to serialize to {path}"));
    }
}
