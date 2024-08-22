// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

use crate::error::{ImportError, ImportErrorType};
use std::convert::From;
use std::path::PathBuf;

const COMMENTS_START: &'static str = "//";

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct ImportObject {
    pub name: String,
    pub path: PathBuf,
    pub source: String,
}

#[allow(unused)]
impl ImportObject {
    pub fn new(name: String, path: PathBuf, source: String) -> Self {
        Self { name, path, source }
    }
}

impl From<String> for ImportObject {
    fn from(value: String) -> Self {
        // init variables

        let path = PathBuf::from(value);
        let path_clone = path.clone();
        let name = path_clone
            .file_name()
            .unwrap_or_else(|| {
                ImportError::throw(
                    format!("Error with formatting path: {:?}", path),
                    ImportErrorType::FormatError,
                );
                std::process::exit(1);
            })
            .to_str()
            .unwrap_or_else(|| {
                ImportError::throw(
                    format!("Error with formatting path: {:?}", path),
                    ImportErrorType::FormatError,
                );
                std::process::exit(1);
            });

        // test if path is really exists

        if !path.exists() {
            ImportError::throw(
                format!("Module `{}` does not exists!", name),
                ImportErrorType::PathError,
            );
            std::process::exit(1);
        }

        // reading source code
        let source = std::fs::read_to_string(path.clone()).unwrap_or_else(|_| {
            ImportError::throw(
                format!("Cannot read `{}` module!", name),
                ImportErrorType::ReadFailure,
            );
            std::process::exit(1);
        });

        // deleting comments

        let formatted_source = source
            .lines()
            .map(|line| {
                if let Some(index) = line.find(COMMENTS_START) {
                    &line[..index]
                } else {
                    line
                }
            })
            .collect::<Vec<&str>>()
            .join("\n");

        Self {
            path,
            name: name.to_string(),
            source: formatted_source,
        }
    }
}
