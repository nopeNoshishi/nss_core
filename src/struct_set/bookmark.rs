// Std
use std::path::PathBuf;

use super::Commit;

#[derive(Debug)]
pub struct BookMark {
    pub name: PathBuf,
    pub commit: Commit,
}

impl BookMark {
    pub fn new(name: PathBuf, commit: Commit) -> Self {
        BookMark { name, commit }
    }
}
