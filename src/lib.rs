#![warn(clippy::all, clippy::pedantic)]

mod assets;
mod details;
pub use assets::*;
pub use details::*;

pub const BASE_URL: &str = "https://ark-us-static-online.yo-star.com/assetbundle/official/Android";
pub const TARGET_PATH: &str = "assets";
