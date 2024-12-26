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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_calculate_directory_size_empty() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let size = calculate_directory_size(&dir_path).unwrap();
        assert_eq!(size, 0, "Empty directory should have size 0");
    }

    #[test]
    fn test_calculate_directory_size_with_files() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let file_path1 = dir_path.join("file1.txt");
        {
            let mut file = std::fs::File::create(&file_path1).unwrap();
            file.write_all(&vec![0u8; 1024]).unwrap();
        }

        let file_path2 = dir_path.join("file2.txt");
        {
            let mut file = std::fs::File::create(&file_path2).unwrap();
            file.write_all(&vec![0u8; 2048]).unwrap();
        }

        let sub_dir_path = dir_path.join("subdir");
        std::fs::create_dir_all(&sub_dir_path).unwrap();
        let sub_file_path = sub_dir_path.join("file3.txt");
        {
            let mut file = std::fs::File::create(&sub_file_path).unwrap();
            file.write_all(&vec![0u8; 1024]).unwrap();
        }

        let size = calculate_directory_size(&dir_path).unwrap();
        assert_eq!(size, 1024 + 2048 + 1024, "Total size should be 4KB");
    }

    #[test]
    fn test_check_storage_sufficient_space() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        assert!(
            check_storage(&dir_path),
            "Empty directory should have enough space"
        );
    }

    #[test]
    fn test_check_storage_insufficient_space() {
        let dir = tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();

        let big_file_path = dir_path.join("bigfile.bin");
        {
            let mut file = std::fs::File::create(&big_file_path).unwrap();
            let large_data = vec![0u8; (45 * 1024 * 1024) as usize];
            file.write_all(&large_data).unwrap();
        }

        assert!(
            !check_storage(&dir_path),
            "Directory with large file should not have enough space"
        );
    }
}
