// Std
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

// External
use anyhow::{bail, Result};
use dirs::home_dir;

pub fn read_contet_with_string<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let mut file = OpenOptions::new().read(true).open(file_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    Ok(content)
}

pub fn read_contet_with_bytes<P: AsRef<Path>>(file_path: P) -> Result<Vec<u8>> {
    let mut file = OpenOptions::new().read(true).open(file_path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    Ok(bytes)
}

pub fn open_file_trucate<P: AsRef<Path>>(file_path: P, buffer: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;

    file.write_all(buffer)
}

pub fn create_file_with_buffer<P: AsRef<Path>>(file_path: P, buffer: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)?;

    file.write_all(buffer)
}

pub fn create_dir<P: AsRef<Path>>(dir_path: P) -> std::io::Result<()> {
    fs::create_dir_all(dir_path)
}

pub fn remove_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    fs::remove_file(path)
}

pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    fs::remove_dir_all(path)
}

pub fn exists_repo<P: AsRef<Path>>(repo_dir: Option<P>) -> Result<PathBuf> {
    let current_dir = match repo_dir {
        Some(p) => {
            if p.as_ref().to_path_buf() == home_dir().unwrap() {
                bail!("not a nss repository (or any of the parent directories): .nss")
            } else {
                p.as_ref().to_path_buf()
            }
        }
        _ => std::env::current_dir()?,
    };

    let repo_path = current_dir.join(PathBuf::from(".nss"));
    let read_dir = fs::read_dir(&current_dir)?;

    for entry in read_dir {
        match entry?.path() == repo_path {
            true => return Ok(current_dir),
            false => continue,
        }
    }

    return exists_repo(Some(current_dir.parent().unwrap()));
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use testdir::testdir;

    #[test]
    fn test_read_contet_with_string() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        fs::copy(test_file_root, temp_dir.join("first.rs")).unwrap();

        // Existed
        let result = read_contet_with_string(temp_dir.join("first.rs"));
        assert!(result.is_ok());

        let test_content = "#[allow(dead_code)]
fn commit(message: &str) -> std::io::Result<()> {
    let tree_hash = write_tree()?;
    match commit_tree(&tree_hash, message)? {
        Some(c) => update_ref(&c)?,
        _ => println!(\"Nothing to commit\")
    };

    Ok(())
}";

        assert_eq!(result.unwrap(), test_content);

        // No existed
        let result = read_contet_with_string(&temp_dir.join("second.rs"));
        assert!(result.is_err());

        // Directory
        let result = read_contet_with_string(&temp_dir);
        assert!(result.is_err());

        // Clean up: Remove the test dir
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_read_contet_with_bytes() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        fs::copy(test_file_root, temp_dir.join("first.rs")).unwrap();

        // Existed
        let result = read_contet_with_bytes(temp_dir.join("first.rs"));
        assert!(result.is_ok());

        let test_content = b"#[allow(dead_code)]
fn commit(message: &str) -> std::io::Result<()> {
    let tree_hash = write_tree()?;
    match commit_tree(&tree_hash, message)? {
        Some(c) => update_ref(&c)?,
        _ => println!(\"Nothing to commit\")
    };

    Ok(())
}";

        assert_eq!(result.unwrap(), test_content);

        // No existed
        let result = read_contet_with_bytes(temp_dir.join("second.rs"));
        assert!(result.is_err());

        // Directory
        let result = read_contet_with_bytes(&temp_dir);
        assert!(result.is_err());

        // Clean up: Remove the test dir
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_open_file_trucate() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_config")
            .join("config");

        fs::copy(test_file_root, temp_dir.join("config")).unwrap();

        let test_content = "[user]
name = \"nopipi\"
";

        // Existed
        let result = open_file_trucate(temp_dir.join("config"), test_content.as_bytes());
        assert!(result.is_ok());

        let mut file = OpenOptions::new()
            .read(true)
            .open(temp_dir.join("config"))
            .unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        assert_eq!(content, test_content);

        // No existed
        let result = open_file_trucate(temp_dir.join("config2"), test_content.as_bytes());
        assert!(result.is_err());

        // Clean up: Remove the test dir
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_create_file_with_buffer() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        // Target test file and buffer
        let file_path = temp_dir.join("test_file.txt");
        let buffer = b"Hello, world!";

        // Run the function under test
        assert!(create_file_with_buffer(&file_path, buffer).is_ok());

        // Verify that the file is created and its contents match the buffer
        let mut file = fs::File::open(&file_path).unwrap();
        let mut file_contents = Vec::new();
        file.read_to_end(&mut file_contents).unwrap();

        assert_eq!(file_contents, buffer);

        // Already existed file
        assert!(create_file_with_buffer(&file_path, buffer).is_err());

        // Clean up: Remove the test dir
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_create_dir() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        // Target test folder
        let dir_path = temp_dir.join("test_dir").join("test_sub_dir");

        // Run the function under test
        assert!(create_dir(&dir_path).is_ok());

        // Verify that the expected directory are created
        assert!(temp_dir.join("test_dir").is_dir());
        assert!(temp_dir.join("test_dir").join("test_sub_dir").is_dir());

        // Clean up: Remove the test dir
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_remove_file() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        fs::copy(test_file_root, temp_dir.join("first.rs")).unwrap();

        // Existed
        let result = remove_file(temp_dir.join("first.rs"));
        assert!(result.is_ok());

        let oepn_result = File::open(temp_dir.join("first.rs"));
        assert!(oepn_result.is_err());

        // No existed
        let result = remove_file(temp_dir.join("second.rs"));
        assert!(result.is_err());

        // Clean up: Remove the test dir
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_remove_dir_all() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");
        let test_file2_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("second.rs");

        fs::copy(test_file_root, temp_dir.join("first.rs")).unwrap();
        fs::copy(test_file2_root, temp_dir.join("second.rs")).unwrap();

        // Existed
        let result = remove_dir_all(&temp_dir);
        assert!(result.is_ok());

        let oepn_result = File::open(temp_dir.join("first.rs"));
        assert!(oepn_result.is_err());

        // No existed
        let result = remove_file(temp_dir);
        assert!(result.is_err());
    }

    #[test]
    fn test_exists_repo() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        // Create the .nss directory inside the temporary directory
        let nss_dir = temp_dir.join(".nss");
        fs::create_dir(&nss_dir).unwrap();

        // Run the function under test with the repo_dir argument as PathBuf
        let result = exists_repo(Some(temp_dir.clone()));

        // Verify that the function returns the correct result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir);

        // Run the function under test with the repo_dir argument as &Path
        let result = exists_repo(Some(temp_dir.as_path()));

        // Verify that the function returns the correct result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir);

        // Run the function under test without the repo_dir argument
        let result = exists_repo::<PathBuf>(None);

        // Verify that the function returns the correct result
        assert!(result.is_err());

        // Clean up: Remove the temporary directory
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
