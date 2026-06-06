/*
- The goal is to make all file_actions sequential first, and then implement concurrency into all
- To mitigate the total # of functions, going to make all functions to be batch
*/
use std::collections::HashMap;
use axum::body::Bytes;
use tokio::fs;
use uuid::Uuid;



// Takes in vector of file structs
// Outputs the vector of file structs with newly updated paths
// This function overwrites or creates the file at the given path
pub async fn create_or_update_files(file_map: &HashMap<String, Bytes>) -> (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>
) {
    let mut succeeded_file_map: HashMap<&String, &Bytes> = HashMap::new();
    let mut failed_file_map: HashMap<&String, &Bytes> = HashMap::new();
    
    for (path, bytes) in file_map {
        match fs::write(path, bytes).await {
            Ok(_) => {succeeded_file_map.insert(path, bytes);},
            Err(_) => {failed_file_map.insert(path, bytes);},
        }
    }

    return (succeeded_file_map, failed_file_map);
}


// Used for when we want to move a file from one directory to another (specifically when moving a file to another bucket)
pub async fn move_file(old_path: &String, new_path: &String) -> bool {
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

    for (file_id, path) in file_paths {
        match fs::remove_file(&path).await {
            Ok(_) => deleted_files.push(file_id),
            Err(_) => failed_files.push(file_id),
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

