#![warn(clippy::all, clippy::pedantic)]

mod assets;
mod details;
pub use assets::*;
pub use details::*;

pub const BASE_URL: &str = "https://ark-us-static-online.yo-star.com/assetbundle/official/Android";
pub const TARGET_PATH: &str = "assets";

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::marker::Sized;

pub trait Cache {
    #[must_use]
    fn get(path: &str) -> Self
    where
        for<'de> Self: Sized + Deserialize<'de>,
    {
        let file = File::open(path).unwrap_or_else(|_| panic!("Failed to open {path}"));

        serde_json::from_reader(BufReader::new(file))
            .unwrap_or_else(|_| panic!("Failed to deserialize to {path}"))
    }

    fn save(&self, path: &str)
    where
        for<'a> Self: Serialize,
    {
        let file = File::create(path).unwrap_or_else(|_| panic!("Failed to open {path}"));
        serde_json::to_writer(file, &self)
            .unwrap_or_else(|_| panic!("Failed to serialize to {path}"));
    }
}
