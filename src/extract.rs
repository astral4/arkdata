use anyhow::Result;
use std::{
    fs,
    io::{copy, Read, Seek},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

/// # Errors
/// Returns Err if filesystem manipuation, I/O, or unzipping fails.
pub fn extract<S: Read + Seek>(source: S, target_dir: &Path, strip_toplevel: bool) -> Result<()> {
    if !target_dir.exists() {
        fs::create_dir(target_dir)?;
    }

    let mut archive = ZipArchive::new(source)?;

    let do_strip_toplevel = strip_toplevel && has_toplevel(&mut archive)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let mut relative_path = file.mangled_name();

        if do_strip_toplevel {
            let base = relative_path
                .components()
                .take(1)
                .fold(PathBuf::new(), |mut p, c| {
                    p.push(c);
                    p
                });
            relative_path = relative_path.strip_prefix(&base)?.to_path_buf();
        }

        if relative_path.to_string_lossy().is_empty() {
            // Top-level directory
            continue;
        }

        let mut outpath = target_dir.to_path_buf();
        outpath.push(relative_path);

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

fn has_toplevel<S: Read + Seek>(archive: &mut ZipArchive<S>) -> Result<bool> {
    let mut toplevel_dir: Option<PathBuf> = None;
    if archive.len() < 2 {
        return Ok(false);
    }

    for i in 0..archive.len() {
        let file = archive.by_index(i)?.mangled_name();
        if let Some(toplevel_dir) = &toplevel_dir {
            if !file.starts_with(toplevel_dir) {
                return Ok(false);
            }
        } else {
            // First iteration
            let comp: PathBuf = file.components().take(1).collect();
            toplevel_dir = Some(comp);
        }
    }
    Ok(true)
}
