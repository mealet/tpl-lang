// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

pub mod error;
mod macros;
pub mod token;
pub mod token_type;

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
                macros::std_symbol!('&', TokenType::Ampersand),
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
                macros::std_function!("type"),
                macros::std_function!("to_str"),
                macros::std_function!("to_int8"),
                macros::std_function!("to_int16"),
                macros::std_function!("to_int32"),
                macros::std_function!("to_int64"),
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
                macros::std_keyword!("int8"),
                macros::std_keyword!("int16"),
                macros::std_keyword!("int32"),
                macros::std_keyword!("int64"),
                macros::std_keyword!("int128"),
                macros::std_keyword!("auto"),
                macros::std_keyword!("fn"),
                macros::std_keyword!("void"),
                macros::std_keyword!("str"),
                macros::std_keyword!("bool"),
                // Boolean Values
                macros::std_token!("true", TokenType::Boolean),
                macros::std_token!("false", TokenType::Boolean),
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
            self.position,
            self.char,
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
        self.char == '\0'
    }

    // helpful functions

    fn get_integer(&mut self) -> i64 {
        let mut value = String::new();
        // lexer will support numbers like 10_000_000 instead 10000000
        while self.char.is_ascii_digit() || self.char == '_' {
            if self.char != '_' {
                value.push(self.char);
            }
            self.getc();
        }

        value.parse().unwrap_or_else(|_| {
            self.error("Too big integer found! Max supported number is 64-bit integer: from −9,223,372,036,854,775,808 to 9,223,372,036,854,775,807");
            0
        })
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
                    if self.char.is_ascii_digit() {
                        let value = -self.get_integer();

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
                _ if self.char.is_ascii_digit() => {
                    let value = self.get_integer();

                    output.push(Token::new(TokenType::Number, value.to_string(), self.line));
                }
                _ if self.char.is_alphabetic() => {
                    let allowed_identifier_chars = ['_'];

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

                        // self.getc();
                        // This line was the main reason of failing ~30% parser tests 0_0
                    }
                }

                // undefined chars/symbols
                _ => {
                    self.error(format!("Undefined char found: {}", self.char));
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
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn std_symbols_lexing() {
        let input = String::from("+ - * / = ! < > . , ; ( ) [ ] { }");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                macros::std_symbol!('+', TokenType::Plus).1,
                macros::std_symbol!('-', TokenType::Minus).1,
                macros::std_symbol!('*', TokenType::Multiply).1,
                macros::std_symbol!('/', TokenType::Divide).1,
                macros::std_symbol!('=', TokenType::Equal).1,
                macros::std_symbol!('!', TokenType::Not).1,
                macros::std_symbol!('<', TokenType::Lt).1,
                macros::std_symbol!('>', TokenType::Bt).1,
                macros::std_symbol!('.', TokenType::Dot).1,
                macros::std_symbol!(',', TokenType::Comma).1,
                macros::std_symbol!(';', TokenType::Semicolon).1,
                macros::std_symbol!('(', TokenType::LParen).1,
                macros::std_symbol!(')', TokenType::RParen).1,
                macros::std_symbol!('[', TokenType::LBrack).1,
                macros::std_symbol!(']', TokenType::RBrack).1,
                macros::std_symbol!('{', TokenType::LBrace).1,
                macros::std_symbol!('}', TokenType::RBrace).1,
                Token::new(TokenType::EOF, "".to_string(), 0)
            ]
        );
    }

    #[test]
    fn strings_lexing() {
        let input = String::from(" \"This is an interesting string\" ");
        let expected = String::from("This is an interesting string");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(result[0].value, expected);
    }

    #[test]
    fn test_std_functions_lexing() {
        let input = String::from("print concat");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Function, String::from("print"), 0),
                Token::new(TokenType::Function, String::from("concat"), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_constructions() {
        let input = String::from("if else while for in break");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Keyword, String::from("if"), 0),
                Token::new(TokenType::Keyword, String::from("else"), 0),
                Token::new(TokenType::Keyword, String::from("while"), 0),
                Token::new(TokenType::Keyword, String::from("for"), 0),
                Token::new(TokenType::Keyword, String::from("in"), 0),
                Token::new(TokenType::Keyword, String::from("break"), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_functional_keywords() {
        let input = String::from("define return import");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Keyword, String::from("define"), 0),
                Token::new(TokenType::Keyword, String::from("return"), 0),
                Token::new(TokenType::Keyword, String::from("import"), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_datatypes() {
        let input = String::from("int8 int16 int32 int64 auto void bool str fn");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Keyword, String::from("int8"), 0),
                Token::new(TokenType::Keyword, String::from("int16"), 0),
                Token::new(TokenType::Keyword, String::from("int32"), 0),
                Token::new(TokenType::Keyword, String::from("int64"), 0),
                Token::new(TokenType::Keyword, String::from("auto"), 0),
                Token::new(TokenType::Keyword, String::from("void"), 0),
                Token::new(TokenType::Keyword, String::from("bool"), 0),
                Token::new(TokenType::Keyword, String::from("str"), 0),
                Token::new(TokenType::Keyword, String::from("fn"), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let input = String::from("id1 id2 a b c abc camel_case");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Identifier, String::from("id1"), 0),
                Token::new(TokenType::Identifier, String::from("id2"), 0),
                Token::new(TokenType::Identifier, String::from("a"), 0),
                Token::new(TokenType::Identifier, String::from("b"), 0),
                Token::new(TokenType::Identifier, String::from("c"), 0),
                Token::new(TokenType::Identifier, String::from("abc"), 0),
                Token::new(TokenType::Identifier, String::from("camel_case"), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let input = String::from("1 2 3 1000 1_000_000");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Number, String::from("1"), 0),
                Token::new(TokenType::Number, String::from("2"), 0),
                Token::new(TokenType::Number, String::from("3"), 0),
                Token::new(TokenType::Number, String::from("1000"), 0),
                Token::new(TokenType::Number, String::from("1000000"), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_negative_numbers() {
        let input = String::from("-1 -2 -3 -1000 -1_000_000");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Number, String::from("-1"), 0),
                Token::new(TokenType::Number, String::from("-2"), 0),
                Token::new(TokenType::Number, String::from("-3"), 0),
                Token::new(TokenType::Number, String::from("-1000"), 0),
                Token::new(TokenType::Number, String::from("-1000000"), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_lines() {
        let input = String::from("line0 \n line1 \n line2");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Identifier, String::from("line0"), 0),
                Token::new(TokenType::Identifier, String::from("line1"), 1),
                Token::new(TokenType::Identifier, String::from("line2"), 2),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_booleans() {
        let input = String::from("true false");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Boolean, String::from("true"), 0),
                Token::new(TokenType::Boolean, String::from("false"), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_compare_operators() {
        let input = String::from("> < == !=");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize().unwrap();

        assert_eq!(
            result,
            vec![
                Token::new(TokenType::Bt, String::from(">"), 0),
                Token::new(TokenType::Lt, String::from("<"), 0),
                Token::new(TokenType::Eq, String::from("=="), 0),
                Token::new(TokenType::Ne, String::from("!="), 0),
                Token::new(TokenType::EOF, String::from(""), 0),
            ]
        );
    }

    #[test]
    fn test_undefined_char() {
        let input = String::from("😃");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let result = lexer.tokenize();

        assert!(result.is_err());
    }

    #[test]
    fn get_integer_test() {
        let input = String::from("50");
        let mut lexer = Lexer::new(input, "tests".to_string());

        let num = lexer.get_integer();

        assert_eq!(num, 50i64);
    }

    #[test]
    fn is_eof_test() {
        let input = String::from("\0");
        let lexer = Lexer::new(input, "tests".to_string());

        let is_eof = lexer.is_eof();

        assert!(is_eof);
    }

    #[test]
    fn getc_test() {
        let input = String::from("50");
        let mut lexer = Lexer::new(input, "tests".to_string());

        assert_eq!(lexer.char, '5');

        lexer.getc();

        assert_eq!(lexer.char, '0');
    }
}
