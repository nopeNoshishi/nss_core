use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Nss IO error: {0}")]
    IOError(#[from] std::io::Error),
}
