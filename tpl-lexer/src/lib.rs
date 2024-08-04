// ------------------------
// Toy Programming Language
// ------------------------

pub mod error;
pub mod token;
pub mod token_type;

use colored::Colorize;
use std::collections::HashMap;
//
use error::{LexerError, LexerErrorHandler};
use token::Token;
use token_type::TokenType;

#[allow(unused)]
pub struct Lexer {
    source: String,
    filename: String,

    std_symbols: HashMap<char, Token>,
    std_words: HashMap<String, Token>,
    errors: LexerErrorHandler,

    input: Vec<char>,
    position: usize,
    line: usize,
    char: char,
}

#[allow(unused)]
impl Lexer {
    // constructor
    pub fn new(source: String, filename: String) -> Self {
        let mut lexer = Lexer {
            source: source.clone(),
            filename,

            std_symbols: HashMap::from([
                ('+', Token::new(TokenType::Plus, String::new())),
                ('-', Token::new(TokenType::Minus, String::new())),
                ('*', Token::new(TokenType::Multiply, String::new())),
                ('/', Token::new(TokenType::Divide, String::new())),
                ('.', Token::new(TokenType::Dot, String::new())),
                ('"', Token::new(TokenType::Quote, String::new())),
            ]),
            std_words: HashMap::from([(
                "print".to_string(),
                Token::new(TokenType::Function, String::from("print")),
            )]),
            errors: LexerErrorHandler::new(),

            input: source.chars().collect(),
            position: 0,
            line: 0,
            char: ' ',
        };

        lexer.getc();
        lexer
    }

    // fundamental functions

    fn error(&mut self, description: &str) {
        let source_clone = self.source.clone();
        let source_lines: Vec<&str> = source_clone.lines().collect();

        self.errors.attach(error::LexerError::new(
            self.filename.clone(),
            description.to_string(),
            source_lines[self.line].to_string(),
            self.line,
            self.position.clone(),
            self.char.clone(),
        ));
    }

    fn getc(&mut self) {
        if self.position < self.input.len() {
            self.char = self.input[self.position];
            self.position += 1;
        } else {
            self.char = '\0'
        }
    }

    // filters

    fn is_eof(&self) -> bool {
        return self.char == '\0';
    }

    // helpful functions

    fn get_number(&mut self) -> i32 {
        let mut value = 0;
        // lexer will support numbers like 10_000_000 instead 10000000
        while self.char.is_digit(10) || self.char == '_' {
            if self.char != '_' {
                value = value * 10 + self.char.to_digit(10).unwrap() as i32;
            }
            self.getc();
        }

        return value;
    }

    // main function

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerErrorHandler> {
        let mut output = Vec::new();

        while !self.is_eof() {
            match self.char {
                '\0' => self.getc(),
                '\n' => {
                    self.line += 1;
                    self.getc();
                }
                _ if self.char.is_whitespace() => self.getc(),
                '-' => {
                    // possibly negative number
                    self.getc();
                    if self.char.is_digit(10) {
                        let value = self.get_number() * -1;

                        // formatting value and matching stringify mode
                        let token_value = value.to_string();
                        let token_type = TokenType::Number;

                        // pushing token

                        output.push(Token::new(token_type, token_value));

                        self.getc();
                    } else {
                        output.push(Token::new(TokenType::Minus, String::new()));
                        self.getc();
                    }
                }
                _ if self.std_symbols.contains_key(&self.char) => {
                    let matched_token = self.std_symbols.get(&self.char).unwrap().clone();

                    if matched_token == Token::new(TokenType::Quote, String::new()) {
                        self.getc();
                        let mut captured_string = String::new();

                        while self.char != '"' {
                            captured_string.push(self.char);
                            self.getc();
                        }

                        // skipping quote
                        self.getc();

                        // pushing token
                        output.push(Token::new(TokenType::String, captured_string));
                        self.getc();
                    } else {
                        output.push(matched_token);
                        self.getc();
                    }
                }
                _ if self.char.is_digit(10) => {
                    let value = self.get_number();

                    output.push(Token::new(TokenType::Number, value.to_string()));
                    self.getc();
                }
                _ if self.char.is_alphabetic() => {
                    let allowed_identifier_chars = ['!', '_', '-', '.'];

                    let mut id = String::new();
                    while self.char.is_alphanumeric()
                        || allowed_identifier_chars.contains(&self.char)
                    {
                        id.push(self.char);
                        self.getc();
                    }

                    if self.std_words.contains_key(&id) {
                        let matched_token = self.std_words.get(&id).unwrap().clone();
                        output.push(matched_token);
                    } else {
                        output.push(Token::new(TokenType::Identifier, id))
                    }
                }

                // undefined chars/symbols
                _ => {
                    let _ = self.error(format!("Undefined char found: {}", self.char).as_str());
                    self.getc();
                }
            }
        }

        if !output.contains(&Token::new(TokenType::EOF, String::new())) {
            output.push(Token::new(TokenType::EOF, String::new()));
        };

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        return Ok(output);
    }
}
