use std::path::PathBuf;

use inquire::{validator::{Validation, StringValidator}, CustomUserError};

pub const STEAM_RELATIVE_PATH: &str = "steamapps/common/Cobalt";

macro_rules! return_if_some {
    ( $x:expr ) => {{
        if let Some(some) = $x {
            return Some(some);
        }
    }};
}

/// Find the path to cobalt, if any
pub fn find_cobalt_path() -> Option<PathBuf> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            let username = whoami::username();
            return_if_some!(search_for_cobalt_in(format!("/home/{}/.steam", username).into()));
            return_if_some!(search_for_cobalt_in(format!("/home/{}/.steam/steam", username).into()));
            return_if_some!(search_for_cobalt_in(format!("/home/{}/.var/app/com.valvesoftware.Steam/data/Steam", username).into()));
            return_if_some!(search_for_cobalt_in(format!("/home/{}/.local/share/Steam", username).into()));
        }
    }
    cfg_if::cfg_if! {
     if #[cfg(target_os = "windows")] {
             return_if_some!(search_for_cobalt_in("C:/Program Files (x86)/Steam".into()));
             return_if_some!(search_for_cobalt_in("C:/Program Files/Steam".into()));
     }
    }
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            let username = whoami::username();
            return_if_some!(search_for_cobalt_in(format!("/Users/{}/Library/Application Support/Steam", username).into()));
        }
    }
    return None;
}

/// Searches for cobalt in a possible steam directory and returns the path if it exists.
fn search_for_cobalt_in(steam_dir: PathBuf) -> Option<PathBuf> {
    if !steam_dir.exists() {
        return None;
    }

    if steam_dir
        .join(STEAM_RELATIVE_PATH)
        .join("cobalt.exe")
        .exists()
    {
        return Some(steam_dir.join(STEAM_RELATIVE_PATH));
    }

    return None;
}

/// Enquire validator to ensure the path is a valid cobalt install
#[derive(Clone)]
pub struct InquireGamePathValidator {}

impl InquireGamePathValidator {
    fn validate_path(&self, path: String) -> Result<Validation, CustomUserError> {
            let pathbuf: PathBuf = path.into();

            if !pathbuf.exists() {
                return Ok(Validation::Invalid("That path doesn't exist.".into()));
            }

            let exe_path = pathbuf.join("cobalt.exe");
            if !exe_path.exists() {
                return Ok(Validation::Invalid("That path doesn't contain cobalt.exe.".into()));
            }

            return Ok(Validation::Valid);
    }
    
}

impl StringValidator for InquireGamePathValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
       self.validate_path(input.to_string()) 
    }
}

/// Enquire validator to ensure the path doesn't already exist
#[derive(Clone)]
pub struct InquirePathDoesntExistValidator {}

impl InquirePathDoesntExistValidator {
    fn validate_path(&self, path: String) -> Result<Validation, CustomUserError> {
            let pathbuf: PathBuf = path.into();

            if !pathbuf.exists() {
                return Ok(Validation::Valid);
            }

            return Ok(Validation::Invalid("That path already exists.".into()));
    }
    
}

impl StringValidator for InquirePathDoesntExistValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
       self.validate_path(input.to_string()) 
    }
}


/// Enquire validator to ensure the path exists
#[derive(Clone)]
pub struct InquirePathExistsValidator {}

impl InquirePathExistsValidator {
    fn validate_path(&self, path: String) -> Result<Validation, CustomUserError> {
            let pathbuf: PathBuf = path.into();

            if pathbuf.exists() {
                return Ok(Validation::Valid);
            }

            return Ok(Validation::Invalid("That path doesn't exist".into()));
    }
    
}

impl StringValidator for InquirePathExistsValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
       self.validate_path(input.to_string()) 
    }
}
