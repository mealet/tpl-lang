use colored::Colorize;

pub struct GenError;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ErrorType {
    NotDefined,
    NotSupported,
    NotExpected,
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
