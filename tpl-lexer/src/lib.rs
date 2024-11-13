// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

pub mod error;
pub mod token;
pub mod token_type;
mod macros;

use std::collections::HashMap;
//
use error::LexerErrorHandler;
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
                macros::std_symbol!('+', TokenType::Plus),
                macros::std_symbol!('-', TokenType::Minus),
                macros::std_symbol!('*', TokenType::Multiply),
                macros::std_symbol!('/', TokenType::Divide),
                macros::std_symbol!('=', TokenType::Equal),
                macros::std_symbol!('!', TokenType::Not),

                macros::std_symbol!('<', TokenType::Lt),
                macros::std_symbol!('>', TokenType::Bt),

                macros::std_symbol!('.', TokenType::Dot),
                macros::std_symbol!(',', TokenType::Comma),
                macros::std_symbol!('"', TokenType::Quote),
                macros::std_symbol!(';', TokenType::Semicolon),

                macros::std_symbol!('(', TokenType::LParen),
                macros::std_symbol!(')', TokenType::RParen),
                macros::std_symbol!('[', TokenType::LBrack),
                macros::std_symbol!(']', TokenType::RBrack),
                macros::std_symbol!('{', TokenType::LBrace),
                macros::std_symbol!('}', TokenType::RBrace),
            ]),
            std_words: HashMap::from([
                // Built-In Functions
                macros::std_function!("print"),
                macros::std_function!("concat"),

                // Constructions
                macros::std_keyword!("if"),
                macros::std_keyword!("else"),
                macros::std_keyword!("while"),
                macros::std_keyword!("for"),
                macros::std_keyword!("in"),
                macros::std_keyword!("break"),

                // Functions and Imports
                macros::std_keyword!("define"),
                macros::std_keyword!("return"),
                macros::std_keyword!("import"),

                // Datatypes
                macros::std_keyword!("int"),
                macros::std_keyword!("str"),
                macros::std_keyword!("bool"),

                // Boolean Values 
                macros::std_keyword!("true"),
                macros::std_keyword!("false"),
            ]),
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

    fn error<T: std::fmt::Display>(&mut self, description: T) {
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

    fn get_number(&mut self) -> i64 {
        let mut value = 0;
        // lexer will support numbers like 10_000_000 instead 10000000
        while self.char.is_digit(10) || self.char == '_' {
            if self.char != '_' {
                value = value * 10 + self.char.to_digit(10).unwrap() as i64;
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

                        output.push(Token::new(token_type, token_value, self.line));

                        self.getc();
                    } else {
                        output.push(Token::new(TokenType::Minus, String::from("-"), 0));
                        self.getc();
                    }
                }
                _ if self.std_symbols.contains_key(&self.char) => {
                    let matched_token = self.std_symbols.get(&self.char).unwrap().clone();

                    match matched_token.token_type {
                        TokenType::Quote => {
                            self.getc();
                            let mut captured_string = String::new();

                            while self.char != '"' {
                                captured_string.push(self.char);
                                self.getc();
                            }

                            // pushing token
                            output.push(Token::new(TokenType::String, captured_string, self.line));
                            self.getc();
                        }
                        TokenType::Equal => {
                            // checking if next symbol is `equal`
                            self.getc();

                            if self.char == '=' {
                                output.push(Token::new(
                                    TokenType::Eq,
                                    String::from("=="),
                                    self.line,
                                ));
                                self.getc();
                            } else {
                                let mut formatted_token = matched_token;
                                formatted_token.line = self.line;

                                output.push(formatted_token);
                            }
                        }
                        TokenType::Not => {
                            // checking if next symbol is `equal`
                            self.getc();

                            if self.char == '=' {
                                output.push(Token::new(
                                    TokenType::Ne,
                                    String::from("!="),
                                    self.line,
                                ));
                                self.getc();
                            } else {
                                let mut formatted_token = matched_token;
                                formatted_token.line = self.line;

                                output.push(formatted_token);
                            }
                        }
                        _ => {
                            let mut formatted_token = matched_token;
                            formatted_token.line = self.line;

                            output.push(formatted_token);
                            self.getc();
                        }
                    }
                }
                _ if self.char.is_digit(10) => {
                    let value = self.get_number();

                    output.push(Token::new(TokenType::Number, value.to_string(), self.line));
                }
                _ if self.char.is_alphabetic() => {
                    let allowed_identifier_chars = ['!', '_', '.'];

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
                        output.push(Token::new(TokenType::Identifier, id, self.line));
                    }
                }

                // undefined chars/symbols
                _ => {
                    let _ = self.error(format!("Undefined char found: {}", self.char));
                    self.getc();
                }
            }
        }

        if !output.contains(&Token::new(TokenType::EOF, String::new(), 0)) {
            output.push(Token::new(TokenType::EOF, String::new(), 0));
        };

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        return Ok(output);
    }
}
