// Std
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

// Intenal
use super::error::Error;
use super::zlib::{read_decoder, write_encoder};

#[derive(Default)]
#[allow(dead_code)]
pub(crate) enum WriteMode {
    #[default]
    Trucate,
    TrucateWithZlib,
    CreateTrucate,
    CreateTrucateWithZlib,
    CreateNewTrucate,
    CreateNewTrucateWithZlib,
}

pub(crate) fn write_content<P: AsRef<Path>>(
    p: P,
    content: &[u8],
    mode: WriteMode,
) -> Result<(), Error> {
    let mut option = OpenOptions::new();
    option.write(true);

    let mut writer: Box<dyn Write> = match mode {
        WriteMode::Trucate => Box::new(option.truncate(true).open(p)?),
        WriteMode::TrucateWithZlib => {
            let file = option.truncate(true).open(p)?;
            Box::new(write_encoder(file))
        }
        WriteMode::CreateTrucate => Box::new(option.truncate(true).create(true).open(p)?),
        WriteMode::CreateTrucateWithZlib => {
            let file = option.truncate(true).create(true).open(p)?;
            Box::new(write_encoder(file))
        }
        WriteMode::CreateNewTrucate => Box::new(option.truncate(true).create_new(true).open(p)?),
        WriteMode::CreateNewTrucateWithZlib => {
            let file = option.truncate(true).create_new(true).open(p)?;
            Box::new(write_encoder(file))
        }
    };

    writer.write_all(content)?;
    writer.flush()?;

    Ok(())
}

#[derive(Default)]
#[allow(dead_code)]
pub(crate) enum ReadMode {
    #[default]
    None,
    WithZlib,
}

pub(crate) fn read_content<P: AsRef<Path>>(p: P, mode: ReadMode) -> Result<Vec<u8>, Error> {
    let mut option = OpenOptions::new();
    option.read(true);

    let mut reader: Box<dyn Read> = match mode {
        ReadMode::None => Box::new(option.open(p)?),
        ReadMode::WithZlib => {
            let file = option.open(p)?;
            Box::new(read_decoder(file))
        }
    };

    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;

    Ok(bytes)
}

pub(crate) fn create_dir<P: AsRef<Path>>(p: P) -> Result<(), Error> {
    fs::create_dir_all(p)?;

    Ok(())
}

#[allow(dead_code)]
pub(crate) fn remove_file<P: AsRef<Path>>(p: P) -> Result<(), Error> {
    fs::remove_file(p)?;

    Ok(())
}

pub(crate) fn remove_dir_all<P: AsRef<Path>>(p: P) -> Result<(), Error> {
    fs::remove_dir_all(p)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Result;
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use testdir::testdir;

    #[test]
    fn test_read_contet() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        fs::copy(test_file_root, temp_dir.join("first.rs")).unwrap();

        let content = read_content(temp_dir.join("first.rs"), ReadMode::default())?;

        let test_content = b"#[allow(dead_code)]
fn commit(message: &str) -> std::io::Result<()> {
    let tree_hash = write_tree()?;
    match commit_tree(&tree_hash, message)? {
        Some(c) => update_ref(&c)?,
        _ => println!(\"Nothing to commit\")
    };

    Ok(())
}";
        assert_eq!(&content, test_content);

        // No existed
        let result = read_content(temp_dir.join("second.rs"), ReadMode::default());
        assert!(result.is_err());

        // Directory
        let result = read_content(&temp_dir, ReadMode::default());
        assert!(result.is_err());

        // Clean up: Remove the test dir
        fs::remove_dir_all(temp_dir).unwrap();

        Ok(())
    }

    #[test]
    fn test_create_dir() {
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());
        let dir_path = temp_dir.join("test_dir").join("test_sub_dir");

        assert!(create_dir(&dir_path).is_ok());
        assert!(temp_dir.join("test_dir").is_dir());
        assert!(temp_dir.join("test_dir").join("test_sub_dir").is_dir());

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_remove_file() {
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

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_remove_dir_all() {
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

        let result = remove_dir_all(&temp_dir);
        assert!(result.is_ok());

        let oepn_result = File::open(temp_dir.join("first.rs"));
        assert!(oepn_result.is_err());

        let result = remove_file(temp_dir);
        assert!(result.is_err());
    }
}
