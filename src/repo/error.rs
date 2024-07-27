use crate::nss_io::error::Error as NssIoError;
use crate::struct_set::error::Error as NssStructError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Detached head")]
    DetachHead,

    #[error("Dismatch hash bookmark {0}")]
    DontMatchHashAtBookmarker(String),

    #[error("{0}")]
    NssFileSystem(#[from] NssIoError),

    #[error("{0}")]
    NssStruct(#[from] NssStructError),

    #[error("{0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("{0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("not string utf8: {0}")]
    NotUtf8String(#[from] std::string::FromUtf8Error),

    #[error("{0}")]
    IoError(#[from] std::io::Error),

    #[error("No nss repository (or any of the parent directories): .nss")]
    NotFoundRepository,
}
