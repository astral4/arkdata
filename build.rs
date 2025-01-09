use std::{
    env::var,
    error::Error,
    fs::{read_to_string, File},
    io::Write,
    path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut output_file = File::create(Path::new(&var("OUT_DIR")?).join("codegen.rs"))?;

    let py_src = read_to_string(
        Path::new(&var("CARGO_MANIFEST_DIR")?)
            .join("src")
            .join("extract.py"),
    )?;

    write!(
        &mut output_file,
        "const PY_FILE: &std::ffi::CStr = cr#\"{py_src}\"#;",
    )?;

    Ok(())
}
