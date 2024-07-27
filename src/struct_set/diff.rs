pub trait Diff<T, U> {
    fn diff(&self, vs: T) -> Vec<(DIffTag, U)>;
}

#[derive(Debug)]
pub enum DIffTag {
    Delete,
    Insert,
    Equal,
    Replace,
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_index_empty() {
//         diff_s()
//     }
// }
