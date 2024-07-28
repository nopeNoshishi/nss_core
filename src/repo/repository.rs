//! Repository addresser

mod bookmark;
mod config;
mod head;
mod index;
mod objects;

// Std
use std::fs;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

// External
use dirs::home_dir;

// Internal
use super::config::Config;
use super::error::Error;
use crate::struct_set::error::Error as StructError;
use crate::struct_set::{BookMark, Head, Index, Object};

const REPO_NAME: &str = ".nss";

#[derive(Debug, Clone)]
pub struct Repository<T> {
    root: PathBuf,
    _maker: PhantomData<T>,
}

impl<T> Repository<T> {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            _maker: PhantomData,
        }
    }
}

pub trait RepositoryAccess<T> {
    fn write(&self, item: T) -> Result<(), Error>;

    fn read(&self) -> Result<T, Error>;
}

pub trait RepositoryPathAccess<T> {
    fn write(&self, item: T) -> Result<(), Error>;

    fn read<P: Into<String>>(&self, p: P) -> Result<T, Error>;
}

// Manager for repository absolute path
#[derive(Debug)]
pub struct NssRepository {
    pub root: PathBuf,
    pub config: Repository<Config>,
    pub index: Repository<Index>,
    pub head: Repository<Head>,
    pub objects: Repository<Object>,
    pub bookmark: Repository<BookMark>,
}

impl NssRepository {
    pub fn path(&self) -> PathBuf {
        self.root.clone()
    }

    pub fn temp_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.root.clone().join(path)
    }

    pub fn new(root: PathBuf) -> Self {
        let config = Repository::new(root.join(REPO_NAME).join("config"));
        let index = Repository::new(root.join(REPO_NAME).join("INDEX"));
        let objects = Repository::new(root.join(REPO_NAME).join("objects"));
        let head = Repository::new(root.join(REPO_NAME).join("HEAD"));
        let bookmark = Repository::new(root.join(REPO_NAME).join("bookmarkers"));

        Self {
            root,
            config,
            index,
            objects,
            head,
            bookmark,
        }
    }

    pub fn create(root: PathBuf) -> Result<Self, Error> {
        let config = Repository::<Config>::new(root.join(REPO_NAME));
        let index = Repository::new(root.join(REPO_NAME));
        let objects = Repository::new(root.join(REPO_NAME));
        let head = Repository::new(root.join(REPO_NAME));
        let bookmark = Repository::new(root.join(REPO_NAME));

        Ok(Self {
            root,
            config,
            index,
            objects,
            head,
            bookmark,
        })
    }

    pub fn config(&self) -> &Repository<Config> {
        &self.config
    }

    pub fn index(&self) -> &Repository<Index> {
        &self.index
    }

    pub fn objects(&self) -> &Repository<Object> {
        &self.objects
    }

    pub fn head(&self) -> &Repository<Head> {
        &self.head
    }

    pub fn bookmark(&self) -> &Repository<BookMark> {
        &self.bookmark
    }
}

// utility
pub(crate) fn split_hash(hash: &str) -> (&str, &str) {
    hash.split_at(2)
}

pub(crate) fn object_path<T: Into<String>>(root: PathBuf, hash: T) -> (PathBuf, PathBuf) {
    let hash = hash.into();
    let (dir, file) = hash.split_at(2);

    let obj_dir = root.join(REPO_NAME).join("objects").join(dir);

    let obj_file = obj_dir.join(file);

    (obj_dir, obj_file)
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

    use anyhow::Result;
    use std::fs;
    use testdir::testdir;

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
