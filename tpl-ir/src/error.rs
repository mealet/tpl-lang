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
    pub fn throw<T: std::fmt::Display>(
        description: T,
        error_type: ErrorType,
        module_name: String,
        source: String,
        line: usize,
    ) {
        let stringified_type = format!("{:?}", error_type);
        let fmt = Self::format(description, stringified_type, module_name, source, line);

        eprintln!("{}", fmt);
    }

    pub fn format<T: std::fmt::Display>(
        description: T,
        error_type: String,
        module_name: String,
        source: String,
        line: usize,
    ) -> String {
        let line_number_len = line.to_string().len();
        let fetched_line = source.lines().collect::<Vec<&str>>()[line];

        format!(
            "{} {}\n{}",
            format!("[CodeGen][{}][{}]:", error_type, module_name).red(),
            description,
            format!(
                "{}{}\n {} {} {}\n{}{}",
                " ".repeat(line_number_len + 2),
                "|".cyan(),
                line + 1,
                "|".cyan(),
                fetched_line,
                " ".repeat(line_number_len + 2),
                "|".cyan()
            )
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
            format!("[ImportError][{}]:", error_type).red(),
            description
        )
    }
}
