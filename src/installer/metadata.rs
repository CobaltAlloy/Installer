//! Module related to metadata left behind for future updating

use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use crate::installer::windows::exit_or_windows;
use super::{alloy::SAVED_DIFF_NAME, INSTALLER_FOLDER};

const METADATA_FILENAME: &str = "install_metadata.ron";

/// Data left for future versions to update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallMetadata {
    /// Version of alloy which was last installed
    pub alloy_version: String,
    /// Version of the installer binary which performed the install
    pub installer_version: String,
    /// The filename of the diff file used, must be in the same folder as this metadata file
    pub diff_file: String, 
}

impl InstallMetadata {
    /// Creates install metadata valid for this version of the installer.
    pub fn new() -> InstallMetadata {
        let installer_version = env!("CARGO_PKG_VERSION").to_string();
        Self { alloy_version: "0.0.3".to_string(), installer_version, diff_file: SAVED_DIFF_NAME.to_string() }
    }
}

/// Writes the metadata into the installer folder
pub fn write_metadata(base_path: PathBuf) {
    let meta = InstallMetadata::new();

    let as_string = ron::to_string(&meta).expect("Failed to encode metadata, this should not happen");

    if let Err(e) = std::fs::write(base_path.join(INSTALLER_FOLDER).join(METADATA_FILENAME), as_string) {
        println!("Failed to write install metadata: {}", e);
        exit_or_windows(100);
    }
}

/// Reads the metadata from the installer folder
pub fn read_metadata(base_path: PathBuf) -> Option<InstallMetadata> {
    
    let path = base_path.join(INSTALLER_FOLDER).join(METADATA_FILENAME);
    
    let as_string = std::fs::read_to_string(path).ok()?;

    ron::from_str(&as_string).expect("Failed to decode install metadata, this should not happen")
}
