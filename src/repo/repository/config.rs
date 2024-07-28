use std::path::PathBuf;

use super::super::error::Error;
use super::{Repository, RepositoryAccess};
use crate::config::{Config, User};
use crate::nss_io::file_system::{read_content, write_content, ReadMode, WriteMode};

const ROOT_NAME: &str = "config";

impl Repository<Config> {
    pub fn create(repo_path: PathBuf) -> Result<Self, Error> {
        let root = repo_path.join(ROOT_NAME);
        let init_config = Config::new(User::new(whoami::username(), None));

        let repo = Self::new(root);
        repo.write(init_config)?;

        Ok(repo)
    }
}

impl RepositoryAccess<Config> for Repository<Config> {
    fn write(&self, config: Config) -> Result<(), Error> {
        let content = toml::to_string(&config)?;
        write_content(&self.root, content.as_bytes(), WriteMode::CreateTrucate)?;

        Ok(())
    }

    fn read(&self) -> Result<Config, Error> {
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
    fn test_write_config() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repo_path = temp_dir.clone().join(".nss");
        create_dir(&repo_path)?;
        let repo = Repository::<Config>::create(repo_path)?;

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

        repo.write(config)?;

        let mut file = File::open(repo.root)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        assert_eq!(config_contet, content);

        fs::remove_dir_all(temp_dir)?;

        Ok(())
    }

    #[test]
    fn test_read_config() -> Result<()> {
        // Create a temporary directory for testing
        let temp_dir = testdir! {};
        println!("Test Directory: {}", temp_dir.display());

        let repo_path = temp_dir.clone().join(".nss");
        create_dir(&repo_path)?;
        let repo = Repository::<Config>::create(repo_path)?;
        println!("{repo:?}");

        let test_file = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?)
            .join("tests")
            .join("test_config")
            .join("config");
        fs::copy(&test_file, &repo.root)?;

        let conig = repo.read()?;

        let test_user = User::new(
            "noshishi".to_string(),
            Some("nopenoshishi@gmail.com".to_string()),
        );
        let test_config = Config::new(test_user);

        assert_eq!(conig, test_config);

        Ok(())
    }
}
