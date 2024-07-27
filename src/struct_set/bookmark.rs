// Std
use std::path::PathBuf;

#[derive(Debug)]
pub struct BookMark {
    pub name: PathBuf,
    pub hash: String,
}

impl BookMark {
    pub fn new(name: PathBuf, hash: String) -> Self {
        BookMark { name, hash }
    }
}
