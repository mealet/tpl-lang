// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

use colored::Colorize;

// handler

#[derive(Debug, Clone)]
pub struct LexerErrorHandler {
    data: Vec<Box<LexerError>>,
}

// error type

#[allow(unused)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct LexerError {
    filename: String,
    description: String,

    line: String,
    line_number: usize,
    position: usize,
    char: char,
}

// implementations

#[allow(unused)]
impl LexerErrorHandler {
    pub fn new() -> Self {
        LexerErrorHandler { data: Vec::new() }
    }

    pub fn attach(&mut self, lexer_error: LexerError) {
        self.data.push(Box::new(lexer_error));
    }

    pub fn format_all(&self) -> String {
        let output = self
            .data
            .clone()
            .iter()
            .map(|err| err.format_error())
            .collect();

        return output;
    }

    pub fn is_empty(&self) -> bool {
        return self.data.is_empty();
    }

    pub fn informate(&self) -> String {
        let message = format!("lexing-analyzer found {} errors!", self.data.len());

        let formatted_errors = self.format_all();

        format!("---- {} ----\n{}", message, formatted_errors,)
    }
}

#[allow(unused)]
impl LexerError {
    pub fn new(
        filename: String,
        description: String,
        line: String,
        line_number: usize,
        position: usize,
        char: char,
    ) -> Self {
        LexerError {
            filename,
            description,
            line,
            line_number,
            position,
            char,
        }
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    pub fn error_description(&self) -> String {
        format!("{} {}", "[LexerError]:".red(), self.description.clone())
    }

    pub fn format_error(&self) -> String {
        let line_number_length = self.line_number.to_string().len();

        format!(
            "{} {}\n{}\n{}\n",
            "[LexerError]:".red(),
            self.description.clone(),
            // filename
            format!("--> {}", self.filename).cyan(),
            // lines
            format!(
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
            )
        )
    }

    pub fn debug_message(&self) -> String {
        format!(
            "Description: {:?}
            Line: {:?}
            Position: {:?}
            Char: {:?}",
            self.description.clone(),
            self.line.clone(),
            self.position.clone(),
            self.char.clone(),
        )
    }
}
