/*
- The goal is to make all file_actions sequential first, and then implement concurrency into all
- To mitigate the total # of functions, going to make all functions to be batch
*/
use std::collections::HashMap;
use axum::body::Bytes;
use tokio::fs;


// Takes in vector of file structs
// Outputs the vector of file structs with newly updated paths

pub async fn create_files(file_map: HashMap<&String, &Bytes>) -> (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>) {
    let succeeded_file_map: HashMap<&String, &Bytes> = HashMap::new();
    let failed_file_map: HashMap<&String, &Bytes> = HashMap::new();
    
    for (path, bytes) in file_map {
        match fs::write(path, bytes).await {
            Ok(_) => {succeeded_file_map.insert(path, bytes);},
            Err(_) => {failed_file_map.insert(path, bytes);},
        }
    }

    // All or nothing, if any files fail to be added, delete the successful files
    if (failed_file_map.len() > 0 && succeeded_file_map.len() > 0){
        for (path, bytes) in succeeded_file_map {
            fs::remove_file(path).await?;
        }
    }

    return (succeeded_file_map, failed_file_map);
}

// Takes in vector of file_path
// Returns vector of files and that failed to parse
pub async fn delete_files(file_paths: HashMap<String, String>) -> (Vec<String>, Vec<String>){
    deleted_files = Vec::new();
    failed_files = Vec::new();

    for (file_id, path) in &file_paths {
        match fs::remove_file(path).await {
            Ok(_) => deleted_files.insert(file_id);
            Err(_) => failed_files.insert(file_id);
        }
    }

    return (deleted_files, failed_files);
}

// Takes in vector of file ids
// Returns vector of file_structs
pub async fn read_files(){

}

// Takes in file_id, model of updated fields
// Returns new file as a file_struct
pub async fn update_file(){

}

