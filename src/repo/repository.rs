//! Repository addresser
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

// External
use anyhow::{bail, Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;

use crate::config::Config;
use crate::error::*;
use crate::nss_io::file_system;
use crate::struct_set::{Hashable, Index, IndexVesion1, Object, Commit};

type BookMarker = String;
type CommitHash = String;


// Manager for repository absolute path
#[derive(Debug, Clone)]
pub struct NssRepository {
    root: PathBuf,
}

impl NssRepository {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn path(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn local_path(&self) -> PathBuf {
        self.root.clone().join(".nss")
    }

    pub fn temp_path<T: Into<String>>(&self, hash: T) -> PathBuf {
        self.root.clone().join(".nss").join(hash.into())
    }

    pub fn objects_path<T: Into<String>>(&self, hash: T) -> PathBuf {
        let hash = hash.into();
        let (dir, file) = hash.split_at(2);

        self.root
            .clone()
            .join(".nss")
            .join("objects")
            .join(dir)
            .join(file)
    }

    pub fn bookmarks_path<S: AsRef<str>>(&self, bookmarker: S) -> PathBuf {
        self.root
            .clone()
            .join(".nss")
            .join("bookmarks")
            .join("local")
            .join(bookmarker.as_ref())
    }

    pub fn config_path(&self) -> PathBuf {
        self.root.clone().join(".nss").join("config")
    }

    pub fn write_config(&self, config: Config) -> Result<()> {
        let content = toml::to_string(&config)?;
        file_system::open_file_trucate(self.config_path(), content.as_bytes())?;

        Ok(())
    }

    pub fn read_config(&self) -> Result<Config> {
        let content = file_system::read_contet_with_string(self.config_path())?;

        Ok(toml::from_str(&content)?)
    }

    pub fn head_path(&self) -> PathBuf {
        self.root.clone().join(".nss").join("HEAD")
    }

    pub fn write_head<S: AsRef<str>>(&self, hash_or_bookmark: S) -> Result<()> {

        let hash_or_bookmark = match self.read_bookmark(&hash_or_bookmark) {
            Ok(_) => format!("bookmarks/local/{}", hash_or_bookmark.as_ref()),
            Err(_) => hash_or_bookmark.as_ref().to_string()
        };

        let content = format!("bookmarker: {}", hash_or_bookmark);
        file_system::open_file_trucate(self.head_path(), content.as_bytes())?;

        Ok(())
    }

    pub fn read_head_base(&self) -> Result<BookMarker> {
        let content = file_system::read_contet_with_string(self.head_path())?;
        let prefix_path = content.split(' ').collect::<Vec<&str>>();

        if prefix_path[1].contains('/') {
            let bookmarker = prefix_path[1].split('/').collect::<Vec<&str>>()[2];

            Ok(bookmarker.to_string())
        } else {
            bail!(RepositoryError::DetachHead)
        }
    }

    pub fn read_head(&self) -> Result<CommitHash> {
        let content = file_system::read_contet_with_string(self.head_path())?;
        let prefix_path = content.split(' ').collect::<Vec<&str>>();

        if prefix_path[1].contains('/') {
            let bookmarker = prefix_path[1].split('/').collect::<Vec<&str>>()[2];
            let hash = file_system::read_contet_with_string(self.bookmarks_path(bookmarker))?;

            return Ok(hash)
        }

        Ok(prefix_path[1].to_string())
    }

    pub fn read_head_with_commit(&self) -> Result<Commit> {
        let content = file_system::read_contet_with_string(self.head_path())?;
        let prefix_path = content.split(' ').collect::<Vec<&str>>();

        if prefix_path[1].contains('/') {
            let bookmarker = prefix_path[1].split('/').collect::<Vec<&str>>()[2];
            let hash = file_system::read_contet_with_string(self.bookmarks_path(bookmarker))?;

            match self.read_object(hash)? {
                Object::Commit(c) => return Ok(c),
                _ => bail!(ObjectError::DontMatchType(prefix_path[1].to_string(), "commit".to_string()))
            }
        }

        match self.read_object(prefix_path[1])? {
            Object::Commit(c) => Ok(c),
            _ => bail!(ObjectError::DontMatchType(prefix_path[1].to_string(), "commit".to_string()))
        }
    }

    pub fn write_bookmark<S: AsRef<str>>(&self, bookmarker: S, hash: CommitHash) -> Result<()> {
        file_system::open_file_trucate(self.bookmarks_path(bookmarker), hash.as_bytes())?;

        Ok(())
    }

    pub fn read_bookmark<S: AsRef<str>>(&self, bookmarker: S) -> Result<CommitHash> {
        let content = file_system::read_contet_with_string(self.bookmarks_path(bookmarker))?;

        Ok(content)
    }

    pub fn index_path(&self) -> PathBuf {
        self.root.clone().join(".nss").join("INDEX")
    }

    pub fn write_index(&self, index: Index) -> Result<()> {
        file_system::open_file_trucate(self.index_path(), &index.as_bytes())?;

        Ok(())
    }

    pub fn read_index(&self) -> Result<Index> {
        let bytes = file_system::read_contet_with_bytes(self.index_path())?;

        Index::from_rawindex(bytes)
    }

    pub fn write_object<H: Hashable>(&self, object: H) -> Result<()> {
        let object_path = self.objects_path(hex::encode(object.to_hash()));
        file_system::create_dir(object_path.parent().unwrap())?;

        // Encode
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&object.as_bytes())?;
        let compressed = encoder.finish()?;

        file_system::create_new_with_buffer(object_path, &compressed)?;

        Ok(())
    }

    pub fn read_object<S: AsRef<str>>(&self, hash: S) -> Result<Object> {
        let hash_path = self.try_get_objects_path(hash.as_ref())?;

        let bytes = file_system::read_contet_with_bytes(hash_path)?;

        // Decode
        let mut decoder = ZlibDecoder::new(&bytes[..]);
        let mut object_content: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut object_content)?;

        Object::from_content(object_content)
    }

    pub fn read_commit<S: AsRef<str>>(&self, hash: S) -> Result<Commit> {
        let hash_path = self.try_get_objects_path(hash.as_ref())?;

        let bytes = file_system::read_contet_with_bytes(&hash_path)?;

        // Decode
        let mut decoder = ZlibDecoder::new(&bytes[..]);
        let mut object_content: Vec<u8> = Vec::new();
        decoder.read_to_end(&mut object_content)?;

        match Object::from_content(object_content)? {
            Object::Commit(c) => Ok(c),
            _ => bail!(ObjectError::DontMatchType(hash_path.to_string_lossy().to_string(), "commit".to_string()))
        }
    }

    /// Return your object database **absolutely** path
    pub fn try_get_objects_path<T: Into<String>>(&self, hash: T) -> Result<PathBuf> {
        let hash = hash.into();

        if hash.len() < 6 {
            bail!(ObjectError::LessObjectHash)
        }

        let (dir, file) = hash.split_at(2);
        let object_dir = &self.path().join(".nss").join("objects").join(dir);

        let mut paths: Vec<PathBuf> = vec![];
        self.ext_paths(object_dir, &mut paths)?;

        let mut target_files: Vec<PathBuf> = vec![];
        for path in paths {
            if path.as_os_str().to_string_lossy().contains(file) {
                target_files.push(path)
            }
        }

        if target_files.len() > 2 {
            bail!(ObjectError::CannotSpecifyHash)
        } else if target_files.is_empty() {
            bail!(ObjectError::NotFoundObject)
        }

        Ok(object_dir.join(&target_files[0]))
    }

    pub fn ext_paths<P: AsRef<Path>>(&self, target: P, paths: &mut Vec<PathBuf>) -> Result<()> {
        // Print all files in target directory
        let files = target.as_ref().read_dir().with_context(|| {
            format!(
                "{} object database has no objects",
                target.as_ref().display()
            )
        })?;

        for dir_entry in files {
            let path = dir_entry.unwrap().path();
            paths.push(path);
        }
        paths.sort();

        Ok(())
    }

    pub fn ext_paths_ignore<P: AsRef<Path>>(&self, target: P, paths: &mut Vec<PathBuf>) {
        // Print all files in target directory
        let files = target.as_ref().read_dir().unwrap();

        let mut ignore_paths: Vec<PathBuf> = vec![];

        // Check .nssignore file
        match fs::read_to_string(".nssignore") {
            Ok(content) => {
                let lines = content.lines();
                ignore_paths.extend(
                    lines
                        .into_iter()
                        .filter(|line| !line.contains('#') || line.is_empty())
                        .map(|line| self.path().join(line)),
                );
            }
            Err(..) => (),
        }

        // Program ignore folder
        ignore_paths.extend(vec![self.path().join(".git"), self.path().join(".nss")]);

        for dir_entry in files {
            let path = dir_entry.unwrap().path();

            let mut do_ignore = false;
            for ignore_path in ignore_paths.clone() {
                if path == ignore_path {
                    do_ignore = true
                }
            }

            if do_ignore {
                continue;
            }

            if path.is_dir() {
                self.ext_paths_ignore(&path, paths);
                continue;
            }
            paths.push(path);
        }
        paths.sort();
    }

    #[allow(dead_code)]
    pub fn get_all_paths(&self, target: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut paths = vec![];
        self.ext_paths(target, paths.as_mut())?;

        Ok(paths)
    }

    pub fn get_all_paths_ignore<P: AsRef<Path>>(&self, target: P) -> Vec<PathBuf> {
        let mut paths = vec![];
        self.ext_paths_ignore(target, paths.as_mut());

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::config::*;

    use std::env;
    use std::fs;
    use std::fs::File;
    use testdir::testdir;

    #[test]
    fn test_nss_repository() {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repository = NssRepository::new(temp_dir.clone());

        assert_eq!(repository.path(), temp_dir);
        assert_eq!(repository.local_path(), temp_dir.join(".nss"));
        assert_eq!(
            repository.objects_path("3e46eb7dbc405630832193193cf17385f29cb243"),
            temp_dir
                .join(".nss")
                .join("objects")
                .join("3e")
                .join("46eb7dbc405630832193193cf17385f29cb243")
        );
        assert_eq!(
            repository.bookmarks_path("test"),
            temp_dir
                .join(".nss")
                .join("bookmarks")
                .join("local")
                .join("test")
        );
        assert_eq!(
            repository.config_path(),
            temp_dir.join(".nss").join("config")
        );
        assert_eq!(repository.head_path(), temp_dir.join(".nss").join("HEAD"));
        assert_eq!(repository.index_path(), temp_dir.join(".nss").join("INDEX"));
    }

    #[test]
    fn test_write_config() {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repository = NssRepository::new(temp_dir.clone());
        fs::create_dir_all(repository.path().join(".nss")).unwrap();
        File::create(repository.config_path()).unwrap();

        // New config
        let user = User::new(
            "noshishi".to_string(),
            Some("nopenoshishi@gmail.com".to_string()),
        );
        let config = Config::new(user);
        let config_contet = "[user]
name = \"noshishi\"
email = \"nopenoshishi@gmail.com\"
";

        let result = repository.write_config(config);
        assert!(result.is_ok());

        let config_path = repository.config_path();
        let mut file = File::open(config_path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        assert_eq!(config_contet, content);

        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_read_config() {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repository = NssRepository::new(temp_dir.clone());
        fs::create_dir_all(repository.path().join(".nss")).unwrap();

        let test_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("tests")
            .join("test_config")
            .join("config");
        fs::copy(&test_file, &repository.path().join(".nss").join("config")).unwrap();

        let result = repository.read_config();
        assert!(result.is_ok());

        let test_user = User::new(
            "noshishi".to_string(),
            Some("nopenoshishi@gmail.com".to_string()),
        );
        let test_config = Config::new(test_user);

        assert_eq!(result.unwrap(), test_config);
    }

    #[test]
    fn test_write_head() {}

    #[test]
    fn test_read_head() {}

    #[test]
    fn test_write_index() {}

    #[test]
    fn test_read_index() {}

    #[test]
    fn test_write_object() {}

    #[test]
    fn test_read_object() {}

    #[test]
    fn test_try_get_objects_path() {}

    #[test]
    fn test_ext_paths() {}

    #[test]
    fn test_ext_paths_ignore() {}

    #[test]
    fn test_get_all_paths() {}

    #[test]
    fn test_get_all_paths_ignore() {}

    #[test]
    fn test_repository_debug() {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repository = NssRepository::new(temp_dir.clone());

        let debug = format!("{:?}", repository);

        let test_debug = format!("NssRepository {{ root: \"{}\" }}", temp_dir.display());

        assert_eq!(debug, test_debug);
    }

    #[test]
    fn test_repository_clone() {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repository = NssRepository::new(temp_dir.clone());

        assert_eq!(repository.root, temp_dir);
    }
}
