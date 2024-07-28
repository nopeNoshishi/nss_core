use std::path::PathBuf;

use super::super::error::Error;
use super::{Repository, RepositoryAccess};
use crate::nss_io::file_system::{create_dir, read_content, write_content, ReadMode, WriteMode};
use crate::struct_set::Head;

const ROOT_NAME: &str = "HEAD";

impl Repository<Head> {
    pub fn create(repo_path: PathBuf) -> Result<Self, Error> {
        let root = repo_path.join(ROOT_NAME);
        let init_head = Head::from_path(root.join("local").join("voyage"));

        let repo = Self::new(root);
        repo.write(init_head)?;

        Ok(repo)
    }
}

impl RepositoryAccess<Head> for Repository<Head> {
    fn write(&self, item: Head) -> Result<(), Error> {
        let s = toml::to_string(&item)?;
        write_content(&self.root, s.as_bytes(), WriteMode::CreateTrucate)?;

        Ok(())
    }

    fn read(&self) -> Result<Head, Error> {
        let bytes = read_content(&self.root, ReadMode::default())?;
        let content = String::from_utf8(bytes)?;

        Ok(toml::from_str(&content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nss_io::file_system::create_dir;
    use crate::repo::config::*;

    use anyhow::Result;
    use std::env;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use testdir::testdir;

    #[test]
    fn test_write_head() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repo_path = temp_dir.clone().join(".nss");
        create_dir(&repo_path)?;
        let repo = Repository::<Head>::create(repo_path)?;

        // New config
        let head = Head::from_path(PathBuf::from("feature-abc"));

        repo.write(head)?;

        let head = Head::from_path(PathBuf::from("feature-abc"));

        repo.write(head)?;


        Ok(())
    }

    #[test]
    fn test_read_config() -> Result<()> {


        Ok(())
    }
}
