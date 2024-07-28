// Std
use std::path::PathBuf;

// External
use serde::{Deserialize, Serialize};

use crate::repository::split_hash;

#[derive(Debug, Deserialize, Serialize)]
pub enum Head {
    Bookmarker(PathBuf),
    ObjectHash(String),
}

impl Head {
    pub fn from_path(path: PathBuf) -> Self {
        Self::Bookmarker(path)
    }

    pub fn from_hash(hash: String) -> Self {
        Self::ObjectHash(hash)
    }

    pub fn is_hash(&self) -> bool {
        matches!(self, Head::ObjectHash(_))
    }

    pub fn get_path(&self) -> PathBuf {
        match self {
            Head::Bookmarker(b) => b.clone(),
            Head::ObjectHash(h) => {
                let (d, f) = split_hash(h);
                PathBuf::new().join(d).join(f)
            }
        }
    }
}
