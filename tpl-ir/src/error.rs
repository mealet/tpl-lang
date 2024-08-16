use colored::Colorize;

pub struct GenError;

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ErrorType {
    NotDefined,
    NotSupported,
    NotExpected,
    TypeError,
}

impl GenError {
    pub fn throw(description: String, error_type: ErrorType) {
        let stringified_type = format!("{:?}", error_type);
        let fmt = Self::format(description, stringified_type);

        eprintln!("{}", fmt);
    }

    pub fn format(description: String, error_type: String) -> String {
        format!(
            "{} {}",
            format!("[CodeGen][{}]", error_type).red(),
            description
        )
    }
}
