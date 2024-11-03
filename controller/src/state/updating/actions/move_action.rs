use std::fs;
use std::io::Result;
use std::io::{self};
use std::path::Path;

pub fn execute(src: &str, dest: &str) -> Result<()> {
    let src_path = Path::new(src);
    if !src_path.exists() || !src_path.is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "Source file not found or is not a file"));
    }

    let dest_path = Path::new(dest);
    if !dest_path.exists() {
        println!("Destination directory does not exist. Creating directory: {}", dest);
        fs::create_dir_all(dest_path)?;
    } else if !dest_path.is_dir() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Destination path is not a directory"));
    }

    let file_name = src_path.file_name().ok_or(io::Error::new(io::ErrorKind::InvalidInput, "Invalid source file name"))?;
    let dest_file_path = dest_path.join(file_name);

    println!("Moving file from {} to {}", src, dest_file_path.display());
    fs::rename(src_path, dest_file_path)?;

    Ok(())
}
