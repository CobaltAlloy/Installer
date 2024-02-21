use std::{io::ErrorKind, path::PathBuf};

use inquire::{
    autocompletion::Replacement,
    validator::{StringValidator, Validation},
    Autocomplete, CustomUserError,
};

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
            return Ok(Validation::Invalid(
                "That path doesn't contain cobalt.exe.".into(),
            ));
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

        return Ok(Validation::Invalid("That path already exists".into()));
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

#[derive(Clone, Default)]
/// Provides autocomplete for files
///
/// Stolen from
/// <https://github.com/mikaelmello/inquire/blob/main/inquire/examples/complex_autocompletion.rs>
pub struct FilePathCompleter {
    input: String,
    paths: Vec<String>,
    lcp: String,
}

impl FilePathCompleter {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        if input == self.input {
            return Ok(());
        }

        self.input = input.to_owned();
        self.paths.clear();

        let input_path = std::path::PathBuf::from(input);

        let fallback_parent = input_path
            .parent()
            .map(|p| {
                if p.to_string_lossy() == "" {
                    std::path::PathBuf::from(".")
                } else {
                    p.to_owned()
                }
            })
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let scan_dir = if input.ends_with('/') {
            input_path
        } else {
            fallback_parent.clone()
        };

        let entries = match std::fs::read_dir(scan_dir) {
            Ok(read_dir) => Ok(read_dir),
            Err(err) if err.kind() == ErrorKind::NotFound => std::fs::read_dir(fallback_parent),
            Err(err) => Err(err),
        }?
        .collect::<Result<Vec<_>, _>>()?;

        let mut idx = 0;
        let limit = 15;

        while idx < entries.len() && self.paths.len() < limit {
            let entry = entries.get(idx).unwrap();

            let path = entry.path();
            let path_str = if path.is_dir() {
                format!("{}/", path.to_string_lossy())
            } else {
                path.to_string_lossy().to_string()
            };

            if path_str.starts_with(&self.input) && path_str.len() != self.input.len() {
                self.paths.push(path_str);
            }

            idx = idx.saturating_add(1);
        }

        self.lcp = self.longest_common_prefix();

        Ok(())
    }

    fn longest_common_prefix(&self) -> String {
        let mut ret: String = String::new();

        let mut sorted = self.paths.clone();
        sorted.sort();
        if sorted.is_empty() {
            return ret;
        }

        let mut first_word = sorted.first().unwrap().chars();
        let mut last_word = sorted.last().unwrap().chars();

        loop {
            match (first_word.next(), last_word.next()) {
                (Some(c1), Some(c2)) if c1 == c2 => {
                    ret.push(c1);
                }
                _ => return ret,
            }
        }
    }
}

impl Autocomplete for FilePathCompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        self.update_input(input)?;

        Ok(self.paths.clone())
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        self.update_input(input)?;

        Ok(match highlighted_suggestion {
            Some(suggestion) => Replacement::Some(suggestion),
            None => match self.lcp.is_empty() {
                true => Replacement::None,
                false => Replacement::Some(self.lcp.clone()),
            },
        })
    }
}
