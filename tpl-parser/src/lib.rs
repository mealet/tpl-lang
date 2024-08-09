mod error;
mod expressions;
mod statements;
mod value;

use lazy_static::lazy_static;

use error::ParseErrorHandler;
use tpl_lexer::{token::Token, token_type::TokenType};

use expressions::Expressions;
use statements::Statements;
use value::Value;

// globals

lazy_static! {
    static ref DATATYPES: Vec<&'static str> = vec!["int", "str", "bool"];
    static ref BINARY_OPERATORS: Vec<TokenType> = vec![
        TokenType::Plus,
        TokenType::Minus,
        TokenType::Divide,
        TokenType::Multiply
    ];
}

const END_STATEMENT: TokenType = TokenType::Semicolon;

// struct and impl

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser {
    filename: String,
    source: String,

    tokens: Vec<Token>,
    position: usize,

    errors: ParseErrorHandler,
    eof: bool,
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
            eof: false,
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

    fn is_binary_operand(&self, token_type: TokenType) -> bool {
        BINARY_OPERATORS.contains(&token_type)
    }

    fn skip_eos(&mut self) {
        // EOS - End Of Statement (in current case this is semicolon)
        if self.current().token_type == END_STATEMENT {
            let _ = self.next();
        }
    }

    // fundamental

    fn statement(&mut self) -> Statements {
        let current = self.current();
        match current.token_type {
            TokenType::Keyword => {
                match current.value.as_str() {
                    _ if DATATYPES.contains(&current.value.as_str()) => {
                        // variable annotation
                        self.annotation_statement()
                    }
                    _ => Statements::None,
                }
            }
            TokenType::Function => {
                // searching for built-in functions

                match current.value.as_str() {
                    "print" => self.print_statement(),
                    _ => Statements::None,
                }
            }
            TokenType::Identifier => {
                let next = self.next();

                match next.token_type {
                    TokenType::Equal => self.assign_statement(current.value),
                    END_STATEMENT => {
                        Statements::Expression(Expressions::Value(Value::Identifier(current.value)))
                    }
                    _ => {
                        self.error("Unexpected expression/statement after identifier");
                        self.next();
                        return Statements::None;
                    }
                }
            }
            TokenType::EOF => {
                self.eof = true;
                return Statements::None;
            }
            _ => return Statements::Expression(self.expression()),
        }
    }

    fn expression(&mut self) -> Expressions {
        let current = self.current();

        match current.token_type {
            TokenType::Identifier => {
                let mut node = Expressions::Value(Value::Identifier(current.value));
                let next_token = self.next();

                match next_token.token_type.clone() {
                    _ if self.is_binary_operand(next_token.token_type) => {
                        node = self.binary_expression(node);
                    }
                    END_STATEMENT => {
                        self.next();
                    }
                    _ => {
                        node = Expressions::None;
                        self.error("Unexpected operation in expression");
                    }
                }

                return node;
            }
            TokenType::Number => {
                let mut node =
                    Expressions::Value(Value::Integer(current.value.trim().parse().unwrap()));
                let next_token = self.next();

                match next_token.token_type.clone() {
                    _ if self.is_binary_operand(next_token.token_type) => {
                        node = self.binary_expression(node);
                    }
                    END_STATEMENT => {
                        self.next();
                    }
                    _ => {}
                }

                return node;
            }
            TokenType::String => {
                let mut node = Expressions::Value(Value::String(current.value));
                let next_token = self.next();

                // for now nothing here

                return node;
            }
            END_STATEMENT => {
                let _ = self.next();
                return Expressions::None;
            }
            _ => {
                self.error("Expression or Statement expected");
                self.next();
            }
        }

        return Expressions::None;
    }

    // expressions

    fn binary_expression(&mut self, node: Expressions) -> Expressions {
        let current_token = self.current();

        match current_token.token_type.clone() {
            _ if self.is_binary_operand(current_token.token_type) => {
                let _ = self.next();

                let lhs = Box::new(node);
                let rhs = Box::new(self.expression());

                return Expressions::Binary {
                    operand: current_token.value,
                    lhs,
                    rhs,
                };
            }
            _ => {
                self.error("Unexpected token at binary expression!");
                self.next();
                return Expressions::None;
            }
        }
    }

    // statements

    fn print_statement(&mut self) -> Statements {
        let mut current = self.current();

        match current.token_type {
            TokenType::Function => {
                current = self.next();
                return self.print_statement();
            }
            TokenType::LParen => {}
            _ => {
                self.error("Unexpected usage of `print` statement");
                while self.current().token_type != END_STATEMENT {
                    self.next();
                }
                return Statements::None;
            }
        }

        let arguments =
            self.expressions_enum(TokenType::LParen, TokenType::RParen, TokenType::Comma);

        if self.current().token_type == END_STATEMENT {
            let _ = self.next();
        }

        return Statements::FunctionCallStatement {
            function_name: String::from("print"),
            arguments,
        };
    }

    fn annotation_statement(&mut self) -> Statements {
        if DATATYPES.contains(&self.current().value.as_str()) {
            let datatype = self.current().value;
            let _ = self.next();

            if !self.expect(TokenType::Identifier) {
                self.error("Identifier expected after type keyword!");
                self.next();

                return Statements::None;
            }

            let id = self.current().value;

            match self.next().token_type {
                TokenType::Equal => {
                    let _ = self.next();
                    let value = self.expression();

                    self.skip_eos(); // skipping semicolon if it exists

                    return Statements::AnnotationStatement {
                        identifier: id,
                        datatype,
                        value: Some(Box::new(value)),
                    };
                }
                END_STATEMENT => {
                    self.skip_eos();

                    return Statements::AnnotationStatement {
                        identifier: id,
                        datatype,
                        value: None,
                    };
                }
                _ => {
                    self.error("Expected `=` or `;` after variable annotation");

                    self.next();
                    return Statements::None;
                }
            }
        } else {
            return Statements::None;
        }
    }

    fn assign_statement(&mut self, identifier: String) -> Statements {
        match self.current().token_type {
            TokenType::Equal => {
                self.next();
                return self.assign_statement(identifier);
            }
            END_STATEMENT => {
                self.error("Expressions expected in assign statement, but `;` found!");
                self.next();
                return Statements::None;
            }
            _ => {
                return Statements::AssignStatement {
                    identifier,
                    value: Some(Box::new(self.expression())),
                };
            }
        }
    }

    // etc

    fn expressions_enum(
        &mut self,
        start_token_type: TokenType,
        end_token_type: TokenType,
        separator: TokenType,
    ) -> Vec<Expressions> {
        let mut current = self.current();

        match current.token_type {
            start_token_type => current = self.next(),
            end_token_type => {
                self.error("Unexpected enumeration end");
                return Vec::new();
            }
        }

        let mut output = Vec::new();

        while current.token_type != end_token_type {
            current = self.current();

            match current.token_type {
                separator => {
                    let _ = self.next();
                    continue;
                }
                end_token_type => {
                    let _ = self.next();
                    break;
                }
                _ => {
                    // FIXME: Idk why, but that case never catches and parser cant see any
                    // arguments
                    let expression = self.expression();
                    output.push(expression);
                }
            }
        }

        if self.current().token_type == end_token_type {
            let _ = self.next();
        }

        return output;
    }

    // main function

    pub fn parse(&mut self) -> Result<Vec<Statements>, ParseErrorHandler> {
        let mut output = Vec::new();

        while self.position < self.tokens.len() - 1 {
            let stmt = self.statement();
            output.push(stmt);

            if self.eof {
                break;
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }
        return Ok(output);
    }
}
