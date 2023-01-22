use crate::CONFIG;
use glob::glob;
use image::open;
use rayon::iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use serde::{de, Deserialize};
use std::{
    fs::{remove_file, File},
    io::BufReader,
    iter::zip,
    path::PathBuf,
};

const ALPHA_SUFFIXES: [&str; 3] = ["_alpha", "[alpha]", "a"];

fn get_rgb_path(path: PathBuf) -> Option<(PathBuf, PathBuf)> {
    if let Some(stem) = path.file_stem() {
        let stem_str = stem.to_string_lossy();
        for suffix in ALPHA_SUFFIXES {
            if stem_str.ends_with(suffix) {
                return Some((
                    path.with_file_name(format!("{}.png", stem_str.rsplit_once(suffix).unwrap().0)),
                    path,
                ));
            }
        }
    }
    None
}

/// # Panics
/// Panics if an image cannot be saved to or deleted from the filesystem.
pub fn combine_textures() {
    glob(&format!("{}/**/*.png", CONFIG.output_dir.to_string_lossy()))
        .expect("Failed to construct valid glob pattern")
        .par_bridge()
        .filter_map(Result::ok)
        .filter_map(get_rgb_path)
        .for_each(|paths| {
            if let Ok(rgb_image) = open(&paths.0) {
                if let Ok(alpha_image) = open(&paths.1) {
                    let mut rgb_image = rgb_image.into_rgba8();
                    let alpha_image = alpha_image.into_luma8();

                    zip(rgb_image.pixels_mut(), alpha_image.pixels())
                        .par_bridge()
                        .for_each(|(rgb_pixel, alpha_pixel)| {
                            rgb_pixel.0[3] = alpha_pixel.0[0];
                        });

                    rgb_image.save(&paths.0).unwrap_or_else(|_| {
                        panic!("Failed to save image to {}", paths.0.to_string_lossy())
                    });
                    remove_file(&paths.1).unwrap_or_else(|_| {
                        panic!("Failed to delete image at {}", paths.1.to_string_lossy())
                    });
                }
            }
        });
}

#[derive(Deserialize)]
struct SpriteRect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Deserialize)]
struct Sprite {
    name: String,
    rect: SpriteRect,
    #[serde(deserialize_with = "deserialize_bool")]
    rotate: bool,
}

#[derive(Deserialize)]
struct PortraitData {
    #[serde(rename(deserialize = "m_Name"))]
    name: String,
    #[serde(rename(deserialize = "_sprites"))]
    sprites: Vec<Sprite>,
}

fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: u8 = de::Deserialize::deserialize(deserializer)?;

    match s {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(de::Error::unknown_variant(&s.to_string(), &["0", "1"])),
    }
}

/// # Panics
/// Panics if an image cannot be saved to or deleted from the filesystem, or if portrait data cannot be deserialized.
pub fn process_portraits() {
    let portrait_dir = CONFIG
        .output_dir
        .join(["arts", "charportraits"].iter().collect::<PathBuf>());

    let data_dir = CONFIG.output_dir.join(
        [
            "torappu",
            "dynamicassets",
            "arts",
            "charportraits",
            "UIAtlasTextureRef",
        ]
        .iter()
        .collect::<PathBuf>(),
    );

    if !portrait_dir.is_dir() || !data_dir.is_dir() {
        return;
    }

    glob(&format!("{}/*.json", data_dir.to_string_lossy()))
        .expect("Failed to construct valid glob pattern")
        .par_bridge()
        .filter_map(Result::ok)
        .for_each(|data_path| {
            let file = File::open(&data_path)
                .unwrap_or_else(|_| panic!("Failed to open {}", data_path.to_string_lossy()));

            let data: PortraitData =
                serde_json::from_reader(BufReader::new(file)).unwrap_or_else(|_| {
                    panic!("Failed to deserialize from {}", data_path.to_string_lossy())
                });

            let portrait_path = portrait_dir.join(data.name).with_extension("png");

            if let Ok(portraits) = open(&portrait_path) {
                let height = portraits.height();
                data.sprites.par_iter().for_each(|sprite| {
                    let mut portrait = portraits.crop_imm(
                        sprite.rect.x,
                        height - sprite.rect.y - sprite.rect.h,
                        sprite.rect.w,
                        sprite.rect.h,
                    );

                    if sprite.rotate {
                        portrait = portrait.rotate90();
                    }

                    let target_path = portrait_dir.join(&sprite.name).with_extension("png");
                    portrait.save(&target_path).unwrap_or_else(|_| {
                        panic!("Failed to save image to {}", target_path.to_string_lossy())
                    });
                });

                remove_file(&portrait_path).unwrap_or_else(|_| {
                    panic!(
                        "Failed to delete image at {}",
                        portrait_path.to_string_lossy()
                    )
                });
            }
        });
}
