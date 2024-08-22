// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

use colored::Colorize;

// IR Error

pub struct GenError;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ErrorType {
    NotDefined,
    NotSupported,
    NotExpected,
    VerificationFailure,
    ImportError,
    TypeError,
    MemoryError,
    BuildError,
}

impl GenError {
    pub fn throw<T: std::fmt::Display>(description: T, error_type: ErrorType, line: usize) {
        let stringified_type = format!("{:?}", error_type);
        let fmt = Self::format(description, stringified_type, line);

        eprintln!("{}", fmt);
    }

    pub fn format<T: std::fmt::Display>(description: T, error_type: String, line: usize) -> String {
        format!(
            "{} {}\n{} line: {}",
            format!("[CodeGen][{}]", error_type).red(),
            description,
            "-->".red(),
            line + 1
        )
    }
}

// Import Error

#[derive(Debug, Clone)]
pub struct ImportError;

#[derive(Debug, Clone)]
pub enum ImportErrorType {
    PathError,
    FormatError,
    ReadFailure,
}

impl ImportError {
    pub fn throw<T: std::fmt::Display>(description: T, error_type: ImportErrorType) {
        let stringified_type = format!("{:?}", error_type);
        let fmt = Self::format(description, stringified_type);

        eprintln!("{}", fmt);
    }

    pub fn format<T: std::fmt::Display>(description: T, error_type: String) -> String {
        format!(
            "{} {}",
            format!("[ImportError][{}]", error_type).red(),
            description
        )
    }
}
