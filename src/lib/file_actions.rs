/*
- The goal is to make all file_actions sequential first, and then implement concurrency into all
- To mitigate the total # of functions, going to make all functions to be batch
*/
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use axum::body::Bytes;
use futures::future::join_all;
use tokio::fs;
use uuid::Uuid;

pub fn file_path(buckets_home_path: &str, bucket_id: Uuid, file_id: Uuid) -> String {
    format!("{}/{}/{}", buckets_home_path, bucket_id, file_id)
}

// Takes in vector of file structs
// Outputs the vector of file structs with newly updated paths
// This function overwrites or creates the file at the given path
pub async fn create_or_update_files<'a>(
    file_map: &'a HashMap<String, Bytes>,
) -> (
    HashMap<&'a String, &'a Bytes>,
    HashMap<&'a String, &'a Bytes>,
) {
    let mut succeeded_file_map: HashMap<&'a String, &'a Bytes> = HashMap::new();
    let mut failed_file_map: HashMap<&'a String, &'a Bytes> = HashMap::new();

    async fn create_helper<'a>(path: &'a String, bytes: &'a Bytes) -> bool {
        if let Some(parent) = Path::new(path).parent() {
            if fs::create_dir_all(parent).await.is_err() {
                return false;
            }
        }

        fs::write(path, bytes).await.is_ok()
    }

    let write_futures: Vec<_> = file_map
        .iter()
        .map(|(path, bytes)| create_helper(path, bytes))
        .collect();

    let results = join_all(write_futures).await;

    for ((path, bytes), succeeded) in file_map.iter().zip(results) {
        if succeeded {
            succeeded_file_map.insert(path, bytes);
        } else {
            failed_file_map.insert(path, bytes);
        }
    }

    (succeeded_file_map, failed_file_map)
}

// Used for when we want to move a file from one directory to another (specifically when moving a file to another bucket)
pub async fn move_file(old_path: &str, new_path: &str) -> bool {
    if let Some(parent) = Path::new(new_path).parent() {
        if fs::create_dir_all(parent).await.is_err() {
            return false;
        }
    }

    match fs::copy(old_path, new_path).await {
        Ok(_) => {
            match fs::remove_file(old_path).await {
                Ok(_) => {},
                Err(_) => {}
            } // Even if the deletion is a fail, we copied the old file so it is fine, just a tradeoff

            return true;
        }
        Err(_) => {
            return false;
        }
    }
}

// Takes in map of file_id -> path
// Returns vectors of file_ids that were deleted and that failed to delete
pub async fn delete_files(file_paths: HashMap<String, String>) -> (Vec<String>, Vec<String>) {
    let mut deleted_files = Vec::new();
    let mut failed_files = Vec::new();

    async fn delete_helper(path: &String) -> bool {
        fs::remove_file(&path).await.is_ok()
    }

    let delete_futures: Vec<_> = file_paths
        .iter()
        .map(|(_file_id, path)| delete_helper(path))
        .collect();

    let results = join_all(delete_futures).await;

    for ((file_id, _path), result) in file_paths.iter().zip(results){
        if result {
            deleted_files.push(file_id.clone());
        } else {
            failed_files.push(file_id.clone());
        }
    }

    (deleted_files, failed_files)
}

pub async fn read_files(file_map: &HashMap<Uuid, String>) -> (HashMap<Uuid, Bytes>, Vec<Uuid>) {
    let mut found_files: HashMap<Uuid, Bytes> = HashMap::new();
    let mut not_found_files: Vec<Uuid> = vec![];

    for (file_id, file_path) in file_map {
        match fs::read(file_path).await {
            Ok(bytes) => {
                found_files.insert(*file_id, Bytes::from(bytes));
            }
            Err(_) => {
                not_found_files.push(*file_id);
            }
        }
    }

    return (found_files, not_found_files);
}

pub async fn delete_files_in_bucket(base_dir: &str, bucket_id: Uuid) -> Vec<String> {
    let bucket_dir = format!("{}/{}", base_dir, bucket_id);
    let mut deleted_files = Vec::new();

    let mut read_dir = match fs::read_dir(&bucket_dir).await {
        Ok(dir) => dir,
        Err(_) => return deleted_files,
    };

    let mut file_paths: Vec<(String, PathBuf)> = Vec::new();
    while let Ok(Some(entry)) = read_dir.next_entry().await {
        file_paths.push((
            entry.file_name().to_string_lossy().into_owned(),
            entry.path(),
        ));
    }

    async fn delete_helper(path: PathBuf) -> bool {
        fs::remove_file(&path).await.is_ok()
    }

    let delete_futures: Vec<_> = file_paths
        .iter()
        .map(|(_file_name, path)| delete_helper(path.clone()))
        .collect();

    let results = join_all(delete_futures).await;

    for ((file_name, _path), result) in file_paths.iter().zip(results) {
        if result {
            deleted_files.push(file_name.clone());
        }
    }

    let _ = fs::remove_dir(&bucket_dir).await;

    deleted_files
}