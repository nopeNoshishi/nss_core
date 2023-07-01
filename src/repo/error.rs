use thiserror::Error;

#[derive(Error, Debug)]
pub enum ObjectError {
    #[error("No existed path!")]
    NotFoundPath,
    #[error("No existed object!")]
    NotFoundObject,
    #[error("Need more hash value digit (less digit)")]
    LessObjectHash,
    #[error("Need more hash value digit (nearly hash value exists)")]
    CannotSpecifyHash,
    #[error("{0} is not {1} hash")]
    DontMatchType(String, String)
}

mod tests {
    use super::*;

    #[test]
    fn test_object_error() {
        let not_found_error = ObjectError::NotFoundPath;
        println!("{:?}", not_found_error);
        println!("{}", not_found_error);

        let not_found_object = ObjectError::NotFoundObject;
        println!("{:?}", not_found_object);
        println!("{}", not_found_object);

        let less_object_hash = ObjectError::LessObjectHash;
        println!("{:?}", less_object_hash);
        println!("{}", less_object_hash);

        let cant_specify_hash = ObjectError::CannotSpecifyHash;
        println!("{:?}", cant_specify_hash);
        println!("{}", cant_specify_hash);

        let dont_match_type = ObjectError::DontMatchType("Blob".to_string(), "fqf89q3fauqp3g23g32g".to_string());
        println!("{:?}", dont_match_type);
        println!("{}", dont_match_type);
    }
}