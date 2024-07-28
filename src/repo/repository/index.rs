use super::super::error::Error;
use super::{Repository, RepositoryAccess};
use crate::nss_io::file_system::{read_content, write_content, ReadMode, WriteMode};
use crate::struct_set::{Index, IndexVesion1};

#[allow(dead_code)]
const ROOT_NAME: &str = "INDEX";

impl RepositoryAccess<Index> for Repository<Index> {
    fn write(&self, index: Index) -> Result<(), Error> {
        write_content(&self.root, &index.as_bytes(), WriteMode::default())?;

        Ok(())
    }

    fn read(&self) -> Result<Index, Error> {
        let bytes = read_content(&self.root, ReadMode::default())?;

        Ok(Index::from_rawindex(bytes)?)
    }
}
