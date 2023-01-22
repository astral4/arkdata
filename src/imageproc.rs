use crate::CONFIG;
use glob::glob;
use image::open;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{fs::remove_file, iter::zip, path::PathBuf};

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
            if let Ok(rgb) = open(&paths.0) {
                if let Ok(alpha) = open(&paths.1) {
                    let mut rgb_image = rgb.into_rgba8();
                    let alpha_image = alpha.into_luma8();

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
