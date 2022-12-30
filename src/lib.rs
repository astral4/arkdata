#![warn(clippy::all, clippy::pedantic)]
#![forbid(unsafe_code)]

mod assets;
mod details;
mod extract;
mod settings;
pub use assets::*;
pub use details::*;
pub use extract::*;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader, marker::Sized};

pub static CONFIG: Lazy<settings::Settings> = Lazy::new(settings::Settings::get);

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
        serde_json::to_writer_pretty(file, &self)
            .unwrap_or_else(|_| panic!("Failed to serialize to {path}"));
    }
}
