/*
- The goal is to make all file_actions sequential first, and then implement concurrency into all
- To mitigate the total # of functions, going to make all functions to be batch
*/
use std::collections::HashMap;
use axum::body::Bytes;
use tokio::fs;


// Takes in vector of file structs
// Outputs the vector of file structs with newly updated paths

pub async fn create_files(file_map: &HashMap<String, Bytes>) -> (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>
) {
    let mut succeeded_file_map: HashMap<&String, &Bytes> = HashMap::new();
    let mut failed_file_map: HashMap<&String, &Bytes> = HashMap::new();
    
    for (path, bytes) in file_map {
        match fs::write(path, bytes).await {
            Ok(_) => {succeeded_file_map.insert(path, bytes);},
            Err(_) => {failed_file_map.insert(path, bytes);},
        }
    }

    // All or nothing, if any files fail to be added, delete the successful files
    if failed_file_map.len() > 0 && succeeded_file_map.len() > 0 {
        for (path, _) in &succeeded_file_map {
            match fs::remove_file(path).await {
                Ok(_) => {} 
                Err(_) => {} // Do nothing for now....
            }
        }
    }

    return (succeeded_file_map, failed_file_map);
}

// Takes in vector of file ids
// Returns vector of file_ids that were deleted
/*pub async fn delete_files(Vec<file_id){

}*/

// Takes in vector of file ids
// Returns vector of file_structs
pub async fn read_files(){

}

// Takes in file_id, model of updated fields
// Returns new file as a file_struct
pub async fn update_file(){

}

