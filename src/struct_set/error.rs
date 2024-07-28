use thiserror::Error;

use crate::nss_io::error::Error as NssIoError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("No existed path!")]
    NotFoundPath,

    #[error("No existed object!")]
    NotFoundObject,

    #[error("Not blob object")]
    NotBlobObject,

    #[error("Not tree object")]
    NotTreeObject,

    #[error("Not commit object")]
    NotCommitObject,

    #[error("Already existed obkect!")]
    AlreadyExistsObject,

    #[error("Need more hash value digit (less digit)")]
    LessObjectHash,

    #[error("Need more hash value digit (nearly hash value exists)")]
    CannotSpecifyHash,

    #[error("{0} is not {1} hash")]
    DontMatchType(String, String),

    #[error("nss repository error: {0}")]
    NssIoError(#[from] NssIoError),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_object_error() {
        let not_found_error = Error::NotFoundPath;
        println!("{:?}", not_found_error);
        println!("{}", not_found_error);

        let not_found_object = Error::NotFoundObject;
        println!("{:?}", not_found_object);
        println!("{}", not_found_object);

        let less_object_hash = Error::LessObjectHash;
        println!("{:?}", less_object_hash);
        println!("{}", less_object_hash);

        let cant_specify_hash = Error::CannotSpecifyHash;
        println!("{:?}", cant_specify_hash);
        println!("{}", cant_specify_hash);

        let dont_match_type =
            Error::DontMatchType("Blob".to_string(), "fqf89q3fauqp3g23g32g".to_string());
        println!("{:?}", dont_match_type);
        println!("{}", dont_match_type);
    }
}
