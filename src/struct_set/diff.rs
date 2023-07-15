// use similar::{Algorithm, capture_diff_slices};

// use super::Blob;

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


// fn diff_s() {
//     let blob1 = Blob::new("/Users/noshishi/study/nss-workspace/nss_core/tests/test_repo/first.rs").unwrap();
//     let blob2 = Blob::new("/Users/noshishi/study/nss-workspace/nss_core/tests/test_repo/first_diff.rs").unwrap();

//     let ops = capture_diff_slices(Algorithm::Myers, &blob1.content, &blob2.content);
//     let changes: Vec<_> = ops.iter().flat_map(|x| x.iter_slices(&blob1.content, &blob2.content)).collect();

//     for c in changes{
//         println!("{}", String::from_utf8(Vec::from(c.1)).unwrap());
//     }
        
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_index_empty() {
//         diff_s()
//     }
// }

