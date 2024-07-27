//! Repository addresser

// Std
use std::fs;
use std::path::{Path, PathBuf};

// External
use dirs::home_dir;

// Internal
use super::config::Config;
use super::error::Error;
use crate::nss_io::file_system::{create_dir, read_content, write_content, ReadMode, WriteMode};
use crate::struct_set::error::Error as StructError;
use crate::struct_set::{BookMark, Commit, Hashable, Head, Index, IndexVesion1, Object};

pub trait Repository<T> {
    fn write(&self, item: T) -> Result<(), Error>;

    fn read(&self) -> Result<T, Error>;
}

pub trait PathRepository<T> {
    fn write(&self, item: T) -> Result<(), Error>;

    fn read<P: Into<String>>(&self, p: P) -> Result<T, Error>;
}

const REPO_NAME: &str = ".nss";
const OBJECT_NAME: &str = "objects";
const BOOKMARK_NAME: &str = "bookmarks";
// const LOCAL_NAME: &str = "local";
const CONFIG_NAME: &str = "config";
const HEAD_NAME: &str = "HEAD";
const INDEX_NAME: &str = "INDEX";

#[derive(Debug, Clone)]
pub struct HeadRepository {
    root: PathBuf,
}

impl Repository<Head> for HeadRepository {
    fn write(&self, item: Head) -> Result<(), Error> {
        let s = toml::to_string(&item)?;
        write_content(&self.root, s.as_bytes(), WriteMode::default())?;

        Ok(())
    }

    fn read(&self) -> Result<Head, Error> {
        let bytes = read_content(&self.root, ReadMode::default())?;
        let content = String::from_utf8(bytes)?;

        Ok(toml::from_str(&content)?)
    }
}

impl From<PathBuf> for HeadRepository {
    fn from(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigRepository {
    root: PathBuf,
}

impl Repository<Config> for ConfigRepository {
    fn write(&self, config: Config) -> Result<(), Error> {
        let content = toml::to_string(&config)?;
        write_content(&self.root, content.as_bytes(), WriteMode::default())?;

        Ok(())
    }

    fn read(&self) -> Result<Config, Error> {
        let bytes = read_content(&self.root, ReadMode::default())?;
        let content = String::from_utf8(bytes)?;

        Ok(toml::from_str(&content)?)
    }
}

impl From<PathBuf> for ConfigRepository {
    fn from(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Debug, Clone)]
pub struct IndexRepository {
    root: PathBuf,
}

impl Repository<Index> for IndexRepository {
    fn write(&self, index: Index) -> Result<(), Error> {
        write_content(&self.root, &index.as_bytes(), WriteMode::default())?;

        Ok(())
    }

    fn read(&self) -> Result<Index, Error> {
        let bytes = read_content(&self.root, ReadMode::default())?;

        Ok(Index::from_rawindex(bytes)?)
    }
}

impl From<PathBuf> for IndexRepository {
    fn from(root: PathBuf) -> Self {
        Self { root }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectRepository {
    root: PathBuf,
}

impl ObjectRepository {
    pub fn read_commit<P: Into<String>>(&self, p: P) -> Result<Commit, Error> {
        match self.read(p) {
            Ok(Object::Commit(c)) => Ok(c),
            _ => Err(Error::NssStruct(StructError::CannotSpecifyHash)),
        }
    }
}

impl PathRepository<Object> for ObjectRepository {
    fn write(&self, item: Object) -> Result<(), Error> {
        let hash = hex::encode(item.to_hash());
        let (d, f) = split_hash(&hash);
        let p = self.root.join(d).join(f);
        create_dir(self.root.join(d))?;
        write_content(p, &item.as_bytes(), WriteMode::CreateNewTrucate)?;

        Ok(())
    }

    fn read<P: Into<String>>(&self, p: P) -> Result<Object, Error> {
        let p = p.into();
        let (d, f) = split_hash(&p);
        let p = self.root.join(d).join(f);
        let content = read_content(p, ReadMode::default())?;

        Ok(Object::from_content(content)?)
    }
}

impl From<PathBuf> for ObjectRepository {
    fn from(root: PathBuf) -> Self {
        Self { root }
    }
}

fn object_path<T: Into<String>>(root: PathBuf, hash: T) -> (PathBuf, PathBuf) {
    let hash = hash.into();
    let (dir, file) = hash.split_at(2);

    let obj_dir = root.join(REPO_NAME).join(OBJECT_NAME).join(dir);

    let obj_file = obj_dir.join(file);

    (obj_dir, obj_file)
}

#[derive(Debug, Clone)]
pub struct LocalBookMarkRepository {
    root: PathBuf,
}

impl PathRepository<BookMark> for LocalBookMarkRepository {
    fn write(&self, item: BookMark) -> Result<(), Error> {
        let p = self.root.join(item.name);
        write_content(p, item.hash.as_bytes(), WriteMode::default())?;

        Ok(())
    }

    fn read<P: Into<String>>(&self, bookmarker: P) -> Result<BookMark, Error> {
        let p = self.root.join(bookmarker.into());
        let bytes = read_content(&p, ReadMode::default())?;
        let content = String::from_utf8(bytes)?;

        Ok(BookMark::new(p, content))
    }
}

impl From<PathBuf> for LocalBookMarkRepository {
    fn from(root: PathBuf) -> Self {
        Self { root }
    }
}

// Manager for repository absolute path
#[derive(Debug, Clone)]
pub struct NssRepository {
    pub root: PathBuf,
    pub config: ConfigRepository,
    pub index: IndexRepository,
    pub objects: ObjectRepository,
    pub head: HeadRepository,
    pub bookmark: LocalBookMarkRepository,
}

impl NssRepository {
    pub fn path(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn temp_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.root.clone().join(path)
    }

    pub fn new(root: PathBuf) -> Self {
        let config = root.join(REPO_NAME).join(CONFIG_NAME).into();
        let index = root.join(REPO_NAME).join(INDEX_NAME).into();
        let objects = root.join(REPO_NAME).join(OBJECT_NAME).into();
        let head = root.join(REPO_NAME).join(HEAD_NAME).into();
        let bookmark = root.join(REPO_NAME).join(BOOKMARK_NAME).into();

        Self {
            root,
            config,
            index,
            objects,
            head,
            bookmark,
        }
    }

    pub fn config(&self) -> &ConfigRepository {
        &self.config
    }

    pub fn index(&self) -> &IndexRepository {
        &self.index
    }

    pub fn objects(&self) -> &ObjectRepository {
        &self.objects
    }

    pub fn head(&self) -> &HeadRepository {
        &self.head
    }

    pub fn bookmark(&self) -> &LocalBookMarkRepository {
        &self.bookmark
    }

    // pub fn create(root: PathBuf) -> Self {

    //     Self { root, config, index, objects, head, bookmark }
    // }
}

// utility
pub fn split_hash(hash: &str) -> (&str, &str) {
    hash.split_at(2)
}

pub fn try_get_objects_path<T: Into<String>>(root: PathBuf, hash: T) -> Result<PathBuf, Error> {
    let hash = hash.into();

    if hash.len() < 6 {
        return Err(Error::NssStruct(StructError::LessObjectHash));
    }

    let (_, file) = hash.split_at(2);
    let (object_dir, _) = object_path(root, &hash);

    let mut paths: Vec<PathBuf> = vec![];
    ext_paths(&object_dir, &mut paths)?;

    let mut target_files: Vec<PathBuf> = vec![];
    for path in paths {
        if path.as_os_str().to_string_lossy().contains(file) {
            target_files.push(path)
        }
    }

    if target_files.len() > 2 {
        return Err(Error::NssStruct(StructError::CannotSpecifyHash));
    } else if target_files.is_empty() {
        return Err(Error::NssStruct(StructError::NotFoundObject));
    }

    Ok(object_dir.join(&target_files[0]))
}

fn ext_paths<P: AsRef<Path>>(target: P, paths: &mut Vec<PathBuf>) -> Result<(), Error> {
    // Print all files in target directory
    let files = target.as_ref().read_dir()?;

    for dir_entry in files {
        let path = dir_entry.unwrap().path();
        paths.push(path);
    }
    paths.sort();

    Ok(())
}

pub fn ext_paths_ignore<P: AsRef<Path>>(root: PathBuf, target: P, paths: &mut Vec<PathBuf>) {
    // Print all files in target directory
    let files = target.as_ref().read_dir().unwrap();

    let mut ignore_paths: Vec<PathBuf> = vec![];

    // Check .nssignore file
    if let Ok(content) = fs::read_to_string(".nssignore") {
        let lines = content.lines();
        ignore_paths.extend(
            lines
                .into_iter()
                .filter(|line| !line.contains('#') || line.is_empty())
                .map(|line| root.join(line)),
        );
    }

    // Program ignore folder
    ignore_paths.extend(vec![root.join(".git"), root.join(".nss")]);

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
            ext_paths_ignore(root.clone(), &path, paths);
            continue;
        }
        paths.push(path);
    }
    paths.sort();
}

#[allow(dead_code)]
pub fn get_all_paths(target: &PathBuf) -> Result<Vec<PathBuf>, Error> {
    let mut paths = vec![];
    ext_paths(target, paths.as_mut())?;

    Ok(paths)
}

pub fn get_all_paths_ignore<P: AsRef<Path>>(root: PathBuf, target: P) -> Vec<PathBuf> {
    let mut paths = vec![];
    ext_paths_ignore(root, target, paths.as_mut());

    paths
}

pub fn exists_repo<P: AsRef<Path>>(repo_dir: Option<P>) -> Result<NssRepository, Error> {
    let current_dir = match repo_dir {
        Some(p) => {
            if p.as_ref().to_path_buf() == home_dir().unwrap() {
                return Err(Error::NotFoundRepository);
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
            true => return Ok(NssRepository::new(current_dir)),
            false => continue,
        }
    }

    return exists_repo(current_dir.parent());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::config::*;

    use anyhow::Result;
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use testdir::testdir;

    #[test]
    fn test_write_config() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repository = NssRepository::new(temp_dir.clone());
        let config_path = repository.path().join(".nss").join(CONFIG_NAME);
        fs::create_dir_all(repository.path().join(".nss"))?;
        File::create(&config_path)?;

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

        repository.config().write(config)?;

        let mut file = File::open(config_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        assert_eq!(config_contet, content);

        fs::remove_dir_all(temp_dir)?;

        Ok(())
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
        fs::copy(&test_file, repository.path().join(".nss").join("config")).unwrap();

        let result = repository.config().read();
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
    fn test_exists_repo() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = testdir!();
        println!("Test Directory: {}", temp_dir.display());

        let nss_dir = temp_dir.join(".nss");
        fs::create_dir(&nss_dir).unwrap();

        let result = exists_repo(Some(temp_dir.clone()));
        assert!(result.is_ok());

        let result = exists_repo::<PathBuf>(None);
        assert!(result.is_err());

        // Clean up: Remove the temporary directory
        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }
}
