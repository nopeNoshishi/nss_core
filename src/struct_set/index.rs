// Std
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// External
use anyhow::{bail, Result};
use byteorder::{BigEndian, ByteOrder};
// TODO use serde::{Deserialize, Serialize};

// Internal
use super::{Blob, FileMeta, Hashable, Object, Tree};
use crate::nss_io::file_system;
use crate::repo::repository::NssRepository;
use crate::repo::error::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub version: u32,
    pub filemetas: Vec<FileMeta>,
}

impl Index {
    pub fn empty() -> Self {
        Self {
            version: 1,
            filemetas: vec![],
        }
    }

    pub fn new_all(repository: &NssRepository) -> Result<Self> {
        let mut all_paths = repository.get_all_paths_ignore(repository.path());
        all_paths.sort();

        let filemetas = all_paths
            .iter()
            .map(|path| FileMeta::new(repository, path).unwrap())
            .collect::<Vec<_>>();

        Ok(Self {
            version: 1,
            filemetas,
        })
    }

    pub fn add<P: AsRef<Path>>(
        &mut self,
        repository: &NssRepository,
        file_path: P,
        temp_prefix: Option<P>,
    ) -> Result<()> {
        let add_filemeta = match temp_prefix {
            Some(p) => FileMeta::new_temp(file_path, p)?,
            None => FileMeta::new(repository, file_path)?,
        };

        let mut new_filemetas: Vec<FileMeta> = vec![];
        for filemeta in self.filemetas.clone() {
            if filemeta == add_filemeta {
                continue;
            } else {
                new_filemetas.push(filemeta);
            }
        }
        new_filemetas.push(add_filemeta);
        new_filemetas.sort_by(|a, b| b.filename.cmp(&a.filename));
        self.filemetas = new_filemetas;

        Ok(())
    }

    pub fn try_from_tree(repository: &NssRepository, tree: Tree) -> Result<Self> {
        let mut index = Index::empty();
        let mut path_blob: HashMap<PathBuf, Blob> = HashMap::new();

        let temp_dir = repository.temp_path(hex::encode(tree.to_hash()));
        file_system::create_dir(&temp_dir)?;

        push_paths(repository, &mut path_blob, tree, &temp_dir)?;

        // Tempolary create file -> filemeta
        for (path, blob) in path_blob {
            file_system::create_dir(path.parent().unwrap())?;
            file_system::create_file_with_buffer(&path, &blob.content)?;

            index.add(repository, path, Some(temp_dir.clone()))?;
        }

        file_system::remove_dir_all(temp_dir)?;

        Ok(index)
    }
}

fn padding(size: usize) -> usize {
    // calclate padding size
    let floor = (size - 2) / 8;
    let target = (floor + 1) * 8 + 2;

    target - size
}

fn push_paths(
    repository: &NssRepository,
    path_blob: &mut HashMap<PathBuf, Blob>,
    tree: Tree,
    base_path: &Path,
) -> Result<()> {
    for entry in tree.entries {
        let path = base_path.join(&entry.name);

        if entry.as_type() == "blob" {
            let blob = match repository.read_object(hex::encode(&entry.hash))? {
                Object::Blob(b) => b,
                _ => bail!(ObjectError::DontMatchType("Blob".to_string(), hex::encode(entry.hash))),
            };
            path_blob.insert(path, blob);
        } else {
            let hash = hex::encode(entry.hash);
            let sub_tree = match repository.read_object(&hash)? {
                Object::Tree(t) => t,
                _ => bail!(ObjectError::DontMatchType("Tree".to_string(), hash)),
            };

            push_paths(repository, path_blob, sub_tree, &path)?
        }
    }

    Ok(())
}

pub trait IndexVesion1 {
    fn as_bytes(&self) -> Vec<u8>;
    fn from_rawindex(buf: Vec<u8>) -> Result<Self>
    where
        Self: Sized;
}

impl IndexVesion1 for Index {
    fn as_bytes(&self) -> Vec<u8> {
        let index_header = b"DIRC";
        let index_version = self.version;
        let entry_num = self.filemetas.len() as u32;
        let header = [
            *index_header,
            index_version.to_be_bytes(),
            entry_num.to_be_bytes(),
        ]
        .concat();

        let mut filemetas_vec: Vec<Vec<u8>> = vec![];
        for filemeta in &self.filemetas {
            let len = 62 + filemeta.filename_size as usize;
            let padding = (0..(8 - len % 8)).map(|_| b'\0').collect::<Vec<u8>>();
            let filemeta_vec = [filemeta.as_bytes(), padding].concat();

            filemetas_vec.push(filemeta_vec)
        }

        [header, filemetas_vec.concat()].concat()
    }

    fn from_rawindex(buf: Vec<u8>) -> Result<Self> {
        if buf == Vec::<u8>::new() {
            bail!("First index");
        }

        let entry_num = BigEndian::read_u32(&buf[8..12]) as usize;
        let mut start_size = 12_usize;
        let mut filemetas: Vec<FileMeta> = vec![];
        for _ in 0..entry_num {
            let name_size =
                BigEndian::read_u16(&buf[(start_size + 60)..(start_size + 62)]) as usize;
            filemetas.push(FileMeta::from_rawindex(
                &buf[(start_size)..(start_size + 62 + name_size)],
            ));

            let padding_size = padding(name_size);
            start_size = start_size + 62 + name_size + padding_size;
        }

        Ok(Self {
            version: 1,
            filemetas,
        })
    }
}

// TEST FEATURE！
// pub trait IndexVesion2 {
//     fn as_bytes(&self) -> bincode::Result<Vec<u8>>;
//     fn from_rawindex(buf: Vec<u8>) -> bincode::Result<Self>
//     where
//         Self: Sized;
// }

// impl IndexVesion2 for Index {
//     fn as_bytes(&self) -> bincode::Result<Vec<u8>>
//     where
//         Self: Serialize,
//     {
//         bincode::serialize(self)
//     }

//     fn from_rawindex(buf: Vec<u8>) -> bincode::Result<Self> {
//         bincode::deserialize(&buf)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_empty() {
        let empty_index = Index::empty();
        let test_index = Index {
            version: 1,
            filemetas: vec![],
        };

        assert_eq!(empty_index, test_index);
    }

    #[test]
    fn test_index_new_all() {}

    #[test]
    fn test_index_from_rawindex() {}

    #[test]
    fn test_index_add() {}

    #[test]
    fn test_as_bytes() {}

    #[test]
    fn test_padding() {}

    #[test]
    fn test_index_try_from_tree() {}

    #[test]
    fn test_push_paths() {}

    #[test]
    fn test_to_tree() {}
}
