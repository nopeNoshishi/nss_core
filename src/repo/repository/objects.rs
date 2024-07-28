use std::path::PathBuf;

use super::super::error::Error;
use super::{split_hash, Repository, RepositoryPathAccess};
use crate::nss_io::file_system::{create_dir, read_content, write_content, ReadMode, WriteMode};
use crate::struct_set::{Hashable, Object};

const ROOT_NAME: &str = "objects";

impl Repository<Object> {
    pub fn create(repo_path: PathBuf) -> Result<Self, Error> {
        let root = repo_path.join(ROOT_NAME);
        create_dir(&root)?;

        Ok(Self::new(root))
    }
}

impl RepositoryPathAccess<Object> for Repository<Object> {
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
