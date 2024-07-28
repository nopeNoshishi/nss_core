use std::path::PathBuf;

use super::super::error::Error;
use super::{split_hash, Repository, RepositoryPathAccess};
use crate::nss_io::file_system::{create_dir, read_content, write_content, ReadMode, WriteMode};
use crate::struct_set::{BookMark, Commit, Hashable};

const ROOT_NAME: &str = "bookmarks";

impl Repository<BookMark> {
    pub fn create(repo_path: PathBuf) -> Result<Self, Error> {
        let root = repo_path.join(ROOT_NAME);
        create_dir(&root)?;

        Ok(Self::new(root))
    }
}

impl RepositoryPathAccess<BookMark> for Repository<BookMark> {
    fn write(&self, item: BookMark) -> Result<(), Error> {
        let p = self.root.join(item.name);
        write_content(p, &item.commit.to_hash(), WriteMode::default())?;

        Ok(())
    }

    fn read<P: Into<String>>(&self, bookmarker: P) -> Result<BookMark, Error> {
        let p = self.root.join(bookmarker.into());
        let bytes = read_content(&p, ReadMode::default())?;
        let hash = String::from_utf8(bytes)?;

        let (d, f) = split_hash(&hash);
        let content = read_content(p.join(d).join(f), ReadMode::default())?;
        let commit = Commit::from_rawobject(&content)?;

        Ok(BookMark::new(p, commit))
    }
}
