mod error;
mod expressions;
mod statements;
mod value;

use error::{ParseError, ParseErrorHandler};
use tpl_lexer::{token::Token, token_type::TokenType};

use expressions::Expressions;
use statements::Statements;
use value::Value;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser {
    filename: String,
    source: String,

    tokens: Vec<Token>,
    position: usize,

    errors: ParseErrorHandler,
}

#[allow(unused)]
impl Parser {
    // constructor

    pub fn new(tokens: Vec<Token>, filename: String, source: String) -> Self {
        Parser {
            filename,
            source,
            tokens,
            position: 0,
            errors: ParseErrorHandler::new(),
        }
    }

    // error

    fn error(&mut self, description: &str) {
        let source_clone = self.source.clone();
        let source_lines: Vec<&str> = source_clone.lines().collect();

        let current_line = self.current().line;

        self.errors.attach(error::ParseError::new(
            self.filename.clone(),
            description.to_string(),
            source_lines[current_line].to_string(),
            current_line,
            self.position.clone(),
        ));
    }

    // helpful functions

    fn peek(&mut self, step: usize) -> Token {
        self.position += step;
        self.tokens[self.position].clone()
    }

    fn next(&mut self) -> Token {
        self.peek(1)
    }

    fn current(&self) -> Token {
        self.tokens[self.position].clone()
    }

    fn expect(&self, expected: TokenType) -> bool {
        self.current().token_type == expected
    }

    // fundamental

    fn statement(&mut self) -> Statements {
        let current = self.current();
        match current.token_type {
            TokenType::Keyword => {
                match current.value.as_str() {
                    "let" => {
                        // variable annotation
                        self.annotation_statement()
                    }
                    _ => Statements::None,
                }
            }
            _ => Statements::None,
        }
    }

    // statements

    fn annotation_statement(&mut self) -> Statements {
        if self.current().value == String::from("let") {
            let _ = self.next();

            if !self.expect(TokenType::Identifier) {
                self.error("Identifier expected after `let` keyword!");
                self.next();
                return Statements::None;
            }

            let id = self.current().value;

            match self.next().token_type {
                TokenType::Equal => {
                    let _ = self.next();
                    let value = self.statement();

                    return Statements::AnnotationStatement {
                        identifier: id,
                        value: Some(Box::new(self.statement())),
                    };
                }
                TokenType::Semicolon => {
                    return Statements::AnnotationStatement {
                        identifier: id,
                        value: None,
                    }
                }
                _ => {
                    self.error("Expected `=` or `;` after variable annotation");
                    return Statements::None;
                }
            }
        } else {
            return Statements::None;
        }
    }

    // main function

    pub fn parse(&mut self) -> Result<Vec<Statements>, ParseErrorHandler> {
        let mut output = Vec::new();

        while self.position < self.tokens.len() {
            let stmt = self.statement();
            output.push(stmt);
        }

        Ok(output)
    }
}
