// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

use colored::Colorize;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub struct ParseError {
    filename: String,
    description: String,

    line: String,
    line_number: usize,
    position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseErrorHandler {
    data: Vec<ParseError>,
}

#[allow(unused)]
impl Default for ParseErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseErrorHandler {
    pub fn new() -> Self {
        ParseErrorHandler { data: Vec::new() }
    }

    pub fn attach(&mut self, parse_error: ParseError) {
        self.data.push(parse_error);
    }

    pub fn format_all(&self) -> String {
        let output = self
            .data
            .clone()
            .iter()
            .map(|err| err.format_error())
            .collect();

        output
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn informate(&self) -> String {
        let message = format!("parsing-analyzer found {} errors!", self.data.len());

        let formatted_errors = self.format_all();

        format!("---- {} ----\n{}", message, formatted_errors,)
    }
}

#[allow(unused)]
impl ParseError {
    pub fn new(
        filename: String,
        description: String,
        line: String,
        line_number: usize,
        position: usize,
    ) -> Self {
        ParseError {
            filename,
            description,
            line,
            line_number,
            position,
        }
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    pub fn error_description(&self) -> String {
        format!("{} {}", "[ParseError]:".red(), self.description.clone())
    }

    pub fn format_error(&self) -> String {
        let line_number_length = self.line_number.to_string().len();
        let filename_fmt = format!("--> {}", self.filename).cyan();
        let lines_fmt = format!(
            "{}{}\n {} {} {}\n{}{}",
            // first line
            " ".repeat(line_number_length + 2),
            "|".cyan(),
            // number + line data
            self.line_number,
            "|".cyan(),
            self.line,
            // last line
            " ".repeat(line_number_length + 2),
            "|".cyan(),
        );

        format!(
            "{} {}\n{}\n{}\n",
            "[ParseError]:".red(),
            self.description.clone(),
            // filename
            filename_fmt,
            // lines
            lines_fmt
        )
    }

    pub fn debug_message(&self) -> String {
        format!(
            "Description: {:?}
            Line: {:?}
            Position: {:?}",
            self.description.clone(),
            self.line.clone(),
            self.position.clone(),
        )
    }
}
