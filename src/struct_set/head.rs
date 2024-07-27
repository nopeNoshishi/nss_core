// Std
use std::path::PathBuf;

// External
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Head {
    Bookmarker(PathBuf),
    ObjectHash(String),
}
