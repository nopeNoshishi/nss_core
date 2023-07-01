// Std
use std::ffi::OsString;
use std::path::Path;

// External
use anyhow::Result;
use byteorder::{BigEndian, ByteOrder};
use chrono::prelude::Local;
use chrono::TimeZone;

// TODO use serde::{Deserialize, Serialize};

// Internal
use super::{Blob, Hashable};
use crate::repository::NssRepository;

#[derive(Debug, Clone)]
pub struct FileMeta {
    pub ctime: u32,
    pub ctime_nsec: u32,
    pub mtime: u32,
    pub mtime_nsec: u32,
    pub dev: u32,
    pub ino: u32,
    pub mode: u32,
    pub uid: u32,
    pub gid: u32,
    pub filesize: u32,
    pub hash: Vec<u8>,
    pub filename_size: u16,
    pub filename: OsString,
}

impl FileMeta {
    pub fn new<P: AsRef<Path>>(repository: &NssRepository, path: P) -> Result<Self> {
        // NOTE: Only unix metadata
        use std::os::unix::prelude::MetadataExt;

        let path = path.as_ref();
        // Exstract metadata on file
        let metadata = path.metadata().unwrap();
        let ctime = metadata.ctime() as u32;
        let ctime_nsec = metadata.ctime_nsec() as u32;
        let mtime = metadata.mtime() as u32;
        let mtime_nsec = metadata.mtime_nsec() as u32;
        let dev = metadata.dev() as u32;
        let ino = metadata.ino() as u32;
        let mode = metadata.mode();
        let uid = metadata.uid();
        let gid = metadata.gid();
        let filesize = metadata.size() as u32;

        let object = Blob::new(path)?;
        let hash = object.to_hash();

        // absolute path -> relative path (from repo path)
        let filename = path
            .strip_prefix(repository.path())
            .unwrap()
            .as_os_str()
            .to_os_string();
        let filename_size = filename.len() as u16;

        Ok(Self {
            ctime,
            ctime_nsec,
            mtime,
            mtime_nsec,
            dev,
            ino,
            mode,
            uid,
            gid,
            filesize,
            hash,
            filename_size,
            filename,
        })
    }

    pub fn new_temp<P: AsRef<Path>>(temp_path: P, temp_prefix: P) -> Result<Self> {
        // NOTE: Only unix metadata
        use std::os::unix::prelude::MetadataExt;

        let path = temp_path.as_ref();
        // Exstract metadata on file
        let metadata = path.metadata().unwrap();
        let ctime = metadata.ctime() as u32;
        let ctime_nsec = metadata.ctime_nsec() as u32;
        let mtime = metadata.mtime() as u32;
        let mtime_nsec = metadata.mtime_nsec() as u32;
        let dev = metadata.dev() as u32;
        let ino = metadata.ino() as u32;
        let mode = metadata.mode();
        let uid = metadata.uid();
        let gid = metadata.gid();
        let filesize = metadata.size() as u32;

        let object = Blob::new(path)?;
        let hash = object.to_hash();

        // absolute path -> relative path (from temp path)
        let filename = path
            .strip_prefix(&temp_prefix)
            .unwrap()
            .as_os_str()
            .to_os_string();
        let filename_size = filename.len() as u16;

        Ok(Self {
            ctime,
            ctime_nsec,
            mtime,
            mtime_nsec,
            dev,
            ino,
            mode,
            uid,
            gid,
            filesize,
            hash,
            filename_size,
            filename,
        })
    }

    pub fn from_rawindex(buf: &[u8]) -> Self {
        let ctime = BigEndian::read_u32(&buf[0..4]);
        let ctime_nsec = BigEndian::read_u32(&buf[4..8]);
        let mtime = BigEndian::read_u32(&buf[8..12]);
        let mtime_nsec = BigEndian::read_u32(&buf[12..16]);
        let dev = BigEndian::read_u32(&buf[16..20]);
        let ino = BigEndian::read_u32(&buf[20..24]);
        let mode = BigEndian::read_u32(&buf[24..28]);
        let uid = BigEndian::read_u32(&buf[28..32]);
        let gid = BigEndian::read_u32(&buf[32..36]);
        let filesize = BigEndian::read_u32(&buf[36..40]);
        let hash = Vec::from(&buf[40..60]);
        let filename_size = BigEndian::read_u16(&buf[60..62]);
        let filename = OsString::from(
            String::from_utf8(Vec::from(&buf[62..(62 + (filename_size as usize))])).unwrap(),
        );
        Self {
            ctime,
            ctime_nsec,
            mtime,
            mtime_nsec,
            dev,
            ino,
            mode,
            uid,
            gid,
            filesize,
            hash,
            filename_size,
            filename,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let entry_meta = [
            self.ctime.to_be_bytes(),
            self.ctime_nsec.to_be_bytes(),
            self.mtime.to_be_bytes(),
            self.mtime_nsec.to_be_bytes(),
            self.dev.to_be_bytes(),
            self.ino.to_be_bytes(),
            self.mode.to_be_bytes(),
            self.uid.to_be_bytes(),
            self.gid.to_be_bytes(),
            self.filesize.to_be_bytes(),
        ]
        .concat();

        let filemeta_vec = [
            entry_meta,
            self.hash.clone(),
            Vec::from(self.filename_size.to_be_bytes()),
            self.filename.to_str().unwrap().as_bytes().to_vec(),
        ]
        .concat();

        filemeta_vec
    }
}

impl PartialEq for FileMeta {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl std::fmt::Display for FileMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ctime = Local
            .timestamp_opt(self.ctime as i64, self.ctime_nsec)
            .unwrap();
        let mtime = Local
            .timestamp_opt(self.mtime as i64, self.mtime_nsec)
            .unwrap();

        let ctime = format!("Created Time: {}", ctime);
        let mtime = format!("Modified Time: {}", mtime);
        let device = format!("Device Id: {}", self.dev);
        let inode = format!("Inode Number: {}", self.ino);
        let mode = format!("File Mode: {:0>6o}", self.mode);
        let uid = format!("User Id: {}", self.uid);
        let gid = format!("Group Id: {}", self.gid);
        let file = format!(
            "Name {} / Size {} / Hash {}",
            self.filename.to_str().unwrap(),
            self.filesize,
            hex::encode(self.hash.clone())
        );

        write!(
            f,
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            ctime, mtime, device, inode, mode, uid, gid, file
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env;
    use std::fs;
    use std::path::PathBuf;

    use testdir::testdir;

    #[test]
    fn test_filemeta_new() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        let repository = NssRepository::new(temp_dir.clone());
        fs::copy(test_file_root, repository.path().join("first.rs")).unwrap();

        // Test file
        let result = FileMeta::new(&repository, repository.path().join("first.rs"));
        assert!(result.is_ok());

        // Test directory
        let result = FileMeta::new(&repository, repository.path());
        assert!(result.is_err());

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_filemeta_new_temp() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        fs::copy(test_file_root, temp_dir.join("first.rs")).unwrap();

        // Test file
        let result = FileMeta::new_temp(&temp_dir.join("first.rs"), &temp_dir);
        assert!(result.is_ok());

        // Test directory
        let result = FileMeta::new_temp(&temp_dir, &temp_dir);
        assert!(result.is_err());

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_filemeta_from_rawindex() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        let repository = NssRepository::new(temp_dir.clone());
        fs::copy(test_file_root, repository.path().join("first.rs")).unwrap();

        // Rqw content
        let test_filemeta = FileMeta::new(&repository, repository.path().join("first.rs")).unwrap();
        let content: Vec<u8> = test_filemeta.as_bytes();

        let filemeta = FileMeta::from_rawindex(&content);

        assert_eq!(filemeta, test_filemeta);
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_filemeta_partialeq() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");
        let test_file_root2 = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("second.rs");

        let repository = NssRepository::new(temp_dir.clone());
        fs::copy(test_file_root, repository.path().join("first.rs")).unwrap();
        fs::copy(test_file_root2, repository.path().join("second.rs")).unwrap();

        // Rqw content
        let test_filemeta1 =
            FileMeta::new(&repository, repository.path().join("first.rs")).unwrap();
        let test_filemeta2 =
            FileMeta::new(&repository, repository.path().join("second.rs")).unwrap();
        assert!(test_filemeta1.eq(&test_filemeta1));
        assert!(test_filemeta1.ne(&test_filemeta2));

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_as_bytes() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        let repository = NssRepository::new(temp_dir.clone());
        fs::copy(test_file_root, repository.path().join("first.rs")).unwrap();

        // Rqw content
        let test_filemeta = FileMeta::new(&repository, repository.path().join("first.rs")).unwrap();

        let test_entry = [
            test_filemeta.ctime.to_be_bytes(),
            test_filemeta.ctime_nsec.to_be_bytes(),
            test_filemeta.mtime.to_be_bytes(),
            test_filemeta.mtime_nsec.to_be_bytes(),
            test_filemeta.dev.to_be_bytes(),
            test_filemeta.ino.to_be_bytes(),
            test_filemeta.mode.to_be_bytes(),
            test_filemeta.uid.to_be_bytes(),
            test_filemeta.gid.to_be_bytes(),
            250_u32.to_be_bytes(),
        ]
        .concat();

        let test_content = [
            test_entry,
            hex::decode("5c73008ba75573c20d6a8a6e557d0556d4a84133").unwrap(),
            8_u16.to_be_bytes().to_vec(),
            b"first.rs".to_vec(),
        ]
        .concat();

        assert_eq!(test_filemeta.as_bytes(), test_content);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_display() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        let repository = NssRepository::new(temp_dir.clone());
        fs::copy(test_file_root, repository.path().join("first.rs")).unwrap();

        let test_filemeta = FileMeta::new(&repository, repository.path().join("first.rs")).unwrap();
        let display = format!("{}", test_filemeta);

        let test_ctime = Local
            .timestamp_opt(test_filemeta.ctime as i64, test_filemeta.ctime_nsec)
            .unwrap();
        let test_mtime = Local
            .timestamp_opt(test_filemeta.mtime as i64, test_filemeta.mtime_nsec)
            .unwrap();

        let test_display = format!(
            "Created Time: {}
Modified Time: {}
Device Id: {}
Inode Number: {}
File Mode: {:0>6o}
User Id: {}
Group Id: {}
Name first.rs / Size 250 / Hash 5c73008ba75573c20d6a8a6e557d0556d4a84133",
            test_ctime,
            test_mtime,
            test_filemeta.dev,
            test_filemeta.ino,
            test_filemeta.mode,
            test_filemeta.uid,
            test_filemeta.gid
        );

        assert_eq!(display, test_display);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_filemeta_debug() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        let repository = NssRepository::new(temp_dir.clone());
        fs::copy(test_file_root, repository.path().join("first.rs")).unwrap();

        let filemeta = FileMeta::new(&repository, repository.path().join("first.rs")).unwrap();

        let debug = format!("{:?}", filemeta);

        let test_debug = format!("FileMeta {{ ctime: {}, ctime_nsec: {}, mtime: {}, mtime_nsec: {}, dev: {}, ino: {}, mode: {}, uid: {}, gid: {}, filesize: 250, hash: [92, 115, 0, 139, 167, 85, 115, 194, 13, 106, 138, 110, 85, 125, 5, 86, 212, 168, 65, 51], filename_size: 8, filename: \"first.rs\" }}",
            filemeta.ctime,
            filemeta.ctime_nsec,
            filemeta.mtime,
            filemeta.mtime_nsec,
            filemeta.dev,
            filemeta.ino,
            filemeta.mode,
            filemeta.uid,
            filemeta.gid,
        );

        assert_eq!(debug, test_debug);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_filemeta_clone() {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let test_file_root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_repo")
            .join("first.rs");

        let repository = NssRepository::new(temp_dir.clone());
        fs::copy(test_file_root, repository.path().join("first.rs")).unwrap();

        let filemeta = FileMeta::new(&repository, repository.path().join("first.rs")).unwrap();

        assert_eq!(filemeta, filemeta.clone());

        fs::remove_dir_all(temp_dir).unwrap();
    }
}
