//! This doesn't work for the time being.

/*use std::{path::PathBuf, str::Bytes};

pub use serde::{Deserialize, Serialize};

const MIMETYPE_FOLDER: &str = "application/vnd.google-apps.folder";
const DAISYMOON_FOLDER_ID: &str = "10Tw1c530qnA5l3P6u1jRyzI9fa3sEWz5";

/// You need to include this file when compiling, or you'll get problems
///
/// This translates to the repo root /google-drive-api-key.txt
const API_KEY: &str = include_str!("../../google-drive-api-key.txt");

/// Time to wait in between requests so we don't get rate limited by Google
const REQUEST_WAIT_MS: u128 = 101;

/// How large is this???
const DOWNLOAD_REQUEST_WAIT_MS: u128 = 2001;

// The hardest part of writing this installer is the f&#king google drive api
//
// If you're in a browser - one click -> downloads as a zip
// If you're an app - manually crawl the folder and all its subfolders

/// Downloads the daisymoon folder into base_path/daisyMoon
pub async fn get_daisymoon_folder(base_path: PathBuf) {
    let root = index_folder(DAISYMOON_FOLDER_ID.to_string()).await.unwrap();

    let mut files = Vec::new();
    let mut folders = Vec::new();

    // See which ones are folder
    for maybe_file in root.files {
        if maybe_file.is_folder() {
            folders.push(("daisyMoon".to_string(), maybe_file));
        } else {
            files.push(("daisyMoon".to_string(), maybe_file));
        }
    }

    // Index each folder for further files
    let mut last_request = std::time::Instant::now();
    while folders.len() != 0 {
        let (folder_path, folder) = folders.remove(0);

        let true_folder_path = folder_path.to_string() + "/" + &folder.name;

        // Wait a bit to not get rate limited
        while last_request.elapsed().as_millis() <= REQUEST_WAIT_MS {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let folder_index = index_folder(folder.id).await.unwrap();
        last_request = std::time::Instant::now();

        for maybe_file in folder_index.files {
            if maybe_file.is_folder() {
                folders.push((true_folder_path.clone(), maybe_file));
            } else {
                files.push((true_folder_path.clone(), maybe_file));
            }
        }
    }

    drop(folders);

    println!("Fetched list of files ({})", files.len());
    println!("Downloading each file.. (this may take a bit)");

    while files.len() != 0 {
        let (file_path, file) = files.remove(0);

        let file_path = base_path.clone().join(file_path).join(file.name.clone());

        // Wait a bit to not get rate limited
        while last_request.elapsed().as_millis() <= DOWNLOAD_REQUEST_WAIT_MS {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let file_data = download_file(file.id).await.unwrap();
        last_request = std::time::Instant::now();

        std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        std::fs::write(file_path, file_data).unwrap();

        println!("Downloaded {}, {} left", file.name, files.len());
    }
}

/// Indexes a google drive folder
pub async fn index_folder(id: String) -> Result<DriveListResponse, reqwest::Error> {
    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q='{}'%20in%20parents&key={}",
        id, API_KEY
    );

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client.get(url).send().await?;

    let is_error = response.status().is_client_error();

    let json_body = response.text().await?;

    if is_error {
        println!("An error occurred: {}", json_body);
    }

    let deserialized_response: DriveListResponse = serde_json::from_str(&json_body).unwrap();

    Ok(deserialized_response)
}

/// Downloads a file by id and returns its buffer
pub async fn download_file(id: String) -> Result<Vec<u8>, reqwest::Error> {
    let url = format!(
        "https://www.googleapis.com/drive/v3/files/{}?acknowledgeAbuse=true&alt=media&key={}",
        id, API_KEY
    );

    let reqwest_client = reqwest::Client::new();
    let response = reqwest_client.get(url).send().await?;

    if response.status().is_client_error() {
        println!("An error occurred: {}", response.text().await.unwrap());
        panic!("");
    }

    let bytes = response.bytes().await.unwrap().to_vec();

    Ok(bytes)
}

#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DriveListResponse {
    files: Vec<DriveFile>,
}

#[derive(Clone, PartialEq, Eq, Default, Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct DriveFile {
    pub kind: String,
    pub mimeType: String,
    pub id: String,
    pub name: String,
}

impl DriveFile {
    /// Returns whether or not this file is a folder
    fn is_folder(&self) -> bool {
        self.mimeType.eq(MIMETYPE_FOLDER)
    }
}*/
