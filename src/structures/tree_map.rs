// External
use anyhow::Result;

// Internal
use crate::repository::NssRepository;
use crate::struct_set::Index;

// Std
use std::collections::HashMap;
use std::path::PathBuf;

pub type TreeMap = Vec<(PathBuf, Vec<PathBuf>)>;

pub fn index_2_tree_map(repository: &NssRepository, index: Index) -> Result<TreeMap> {
    let mut file_paths = Vec::new();
    let mut dir_paths = Vec::new();

    let repo_path = repository.path();

    for filemeta in index.filemetas {
        let file_path = repo_path.join(filemeta.filename);
        let mut dir_name = file_path.parent().unwrap().to_path_buf();

        file_paths.push(repo_path.join(file_path));
        dir_paths.push(repo_path.clone());

        while dir_name != repo_path {
            dir_paths.push(repo_path.join(dir_name.clone()));

            dir_name = dir_name.parent().unwrap().to_path_buf();
        }
    }
    dir_paths.sort();
    dir_paths.dedup();

    let mut temp_map: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    for dir in &dir_paths {
        temp_map.insert(dir.to_path_buf(), vec![]);

        for file in &file_paths {
            if dir == &file.parent().unwrap().to_path_buf() {
                temp_map
                    .get_mut(&dir.to_path_buf())
                    .unwrap()
                    .push(file.to_path_buf())
            }
        }

        for sub_dir in &dir_paths {
            if dir == &sub_dir.parent().unwrap().to_path_buf() {
                temp_map
                    .get_mut(&dir.to_path_buf())
                    .unwrap()
                    .push(sub_dir.to_path_buf())
            }
        }
    }

    let mut tmp: Vec<(&PathBuf, &Vec<PathBuf>)> = temp_map.iter().collect();
    tmp.sort_by(|b, a| {
        let comp_a: Vec<&std::ffi::OsStr> = a.0.iter().collect();
        let comp_b: Vec<&std::ffi::OsStr> = b.0.iter().collect();
        if comp_a.len() >= comp_b.len() {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });

    let mut tree_dir: Vec<(PathBuf, Vec<PathBuf>)> = vec![];
    for t in tmp {
        tree_dir.push((t.0.to_owned(), t.1.to_vec()))
    }

    Ok(tree_dir)
}
