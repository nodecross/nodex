use std::fs;
use std::path::Path;

const BASE_LIMIT: u64 = 50 * 1024 * 1024;
const MIN_FREE_SPACE: u64 = 30 * 1024 * 1024;

pub fn check_storage(directory: impl AsRef<Path>) -> bool {
    let dir_path = directory.as_ref().to_path_buf();
    let total_size = calculate_directory_size(&dir_path).unwrap_or(0);
    let free_space = BASE_LIMIT.saturating_sub(total_size);
    free_space >= MIN_FREE_SPACE
}

fn calculate_directory_size(dir: impl AsRef<Path>) -> Result<u64, std::io::Error> {
    let mut total_size = 0;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            total_size += calculate_directory_size(entry.path())?;
        }
    }

    Ok(total_size)
}
