use anyhow::Result;
use std::{
    fs,
    io::{copy, Read, Seek},
    path::Path,
};
use zip::ZipArchive;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use zip::read::ZipFile;

/// # Errors
/// Returns Err if filesystem manipuation, I/O, or unzipping fails.
pub fn extract<S: Read + Seek>(source: S, target_dir: &Path) -> Result<()> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    let mut archive = ZipArchive::new(source)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let relative_path = file.mangled_name();

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
        #[cfg(unix)]
        set_unix_mode(&file, &outpath)?;
    }

    Ok(())
}

#[cfg(unix)]
fn set_unix_mode(file: &ZipFile, outpath: &Path) -> Result<()> {
    if let Some(m) = file.unix_mode() {
        fs::set_permissions(outpath, PermissionsExt::from_mode(m))?;
    }
    Ok(())
}
