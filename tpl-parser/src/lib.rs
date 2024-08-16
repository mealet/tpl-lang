// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

mod error;
pub mod expressions;
pub mod statements;
pub mod value;

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
        TokenType::Plus, // +
        TokenType::Minus, // -
        TokenType::Divide, // *
        TokenType::Multiply, // /

        TokenType::Lt, // <
        TokenType::Bt, // >
        TokenType::Eq, // ==
        TokenType::Ne // !
    ];
    static ref PRIORITY_BINARY_OPERATORS: Vec<String> = vec!["*".to_string(), "/".to_string()];
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

    fn is_priority_binary_operand(&self, operand: String) -> bool {
        PRIORITY_BINARY_OPERATORS.contains(&operand)
    }

    fn skip_eos(&mut self) {
        // EOS - End Of Statement (in current case this is semicolon)
        if self.current().token_type == END_STATEMENT {
            let _ = self.next();
        }
    }

    // fundamental

    fn term(&mut self) -> Expressions {
        let current = self.current();
        let mut output = Expressions::None;

        match current.token_type {
            TokenType::Number => {
                output = Expressions::Value(Value::Integer(current.value.trim().parse().unwrap()))
            }
            TokenType::String => output = Expressions::Value(Value::String(current.value)),
            TokenType::Boolean => {
                output =
                    Expressions::Value(Value::Boolean(if current.value == String::from("true") {
                        true
                    } else {
                        false
                    }))
            }
            TokenType::Identifier => output = Expressions::Value(Value::Identifier(current.value)),
            _ => {
                self.error("Unexpected term found");
            }
        }

        let _ = self.next();
        return output;
    }

    fn statement(&mut self) -> Statements {
        let current = self.current();
        match current.token_type {
            TokenType::Keyword => {
                match current.value.as_str() {
                    _ if DATATYPES.contains(&current.value.as_str()) => {
                        // variable annotation
                        self.annotation_statement()
                    }
                    "if" => {
                        // `if` or `if/else` construction
                        return self.if_statement();
                    }
                    "else" => {
                        self.error(
                            "Unexpected `else` usage. Please use it in `if/else` construction!",
                        );
                        Statements::None
                    }
                    "while" => {
                        // `while` cycle
                        return self.while_statement();
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
        let mut node = self.term();
        let current = self.current();

        match current.token_type {
            _ if self.is_binary_operand(current.token_type) => {
                node = self.binary_expression(node);
            }
            END_STATEMENT => {
                self.next();
            }
            _ => {}
        }

        return node;
    }

    // expressions

    fn binary_expression(&mut self, node: Expressions) -> Expressions {
        let current_token = self.current();

        match current_token.token_type.clone() {
            _ if self.is_binary_operand(current_token.token_type) => {
                let _ = self.next();

                let lhs = node;
                let rhs = self.expression();

                if self.is_priority_binary_operand(current_token.clone().value) {
                    let mut new_node = rhs.clone();
                    let old_lhs = lhs.clone();

                    if let Expressions::Binary { lhs, rhs, operand } = new_node {
                        let lhs_new = old_lhs;
                        let rhs_new = lhs;

                        // creating new expression

                        let priority_node = Expressions::Binary {
                            lhs: Box::new(lhs_new),
                            rhs: rhs_new,
                            operand: current_token.clone().value,
                        };

                        let output_node = Expressions::Binary {
                            lhs: Box::new(priority_node),
                            rhs,
                            operand,
                        };

                        return output_node;
                    }
                }

                return Expressions::Binary {
                    operand: current_token.value,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
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

    fn if_statement(&mut self) -> Statements {
        if self.current().token_type == TokenType::Keyword {
            // skipping keyword
            let _ = self.next();
            return self.if_statement();
        }

        // parsing condition
        let condition = self.expression();

        // searching for opening block
        if self.current().token_type != TokenType::LBrace {
            self.error("New block expected after condition!");
            return Statements::None;
        }

        let _ = self.next();

        // parsing statements
        let mut stmts = Vec::new();

        while self.current().token_type != TokenType::RBrace {
            if self.current().token_type == TokenType::EOF {
                self.error("Unexpected end-of-file in block after `if` statement. Please add '}'!");
                return Statements::None;
            }

            let statement = self.statement();
            stmts.push(statement);
        }

        // skipping brace
        if self.current().token_type == TokenType::RBrace {
            let _ = self.next();
        }

        // checking if we have `else` construction

        let current_token = self.current();

        match current_token.token_type {
            TokenType::Keyword => {
                // checking for `else` keyword
                if current_token.value != String::from("else") {
                    self.error("Unexpected keyword after `if` statement. Please add ';' for ending statement!");
                    return Statements::None;
                }

                let _ = self.next();

                // checking for opening new block
                if !self.expect(TokenType::LBrace) {
                    self.error("New block expected after `else` keyword!");
                    return Statements::None;
                }

                let _ = self.next();

                // parsing statements for `else` block
                let mut else_stmts = Vec::new();

                while self.current().token_type != TokenType::RBrace {
                    if self.current().token_type == TokenType::EOF {
                        self.error(
                            "Unexpected end-of-file in block after `else` statement. Please add '}'!",
                        );
                        return Statements::None;
                    }

                    let statement = self.statement();
                    else_stmts.push(statement);
                }

                // skipping brace
                if self.current().token_type == TokenType::RBrace {
                    let _ = self.next();
                }

                // checking for semicolon
                if self.current().token_type != TokenType::Semicolon {
                    self.error("Semicolon expected after `if/else` construction!");
                    return Statements::None;
                }

                let _ = self.skip_eos();

                return Statements::IfStatement {
                    condition,
                    then_block: stmts,
                    else_block: Some(else_stmts),
                };
            }
            _ => {
                // skipping semicolon if we have
                self.skip_eos();
                // returning statement
                return Statements::IfStatement {
                    condition,
                    then_block: stmts,
                    else_block: None,
                };
            }
        }
    }

    fn while_statement(&mut self) -> Statements {
        if self.current().token_type == TokenType::Keyword {
            // skipping keyword
            let _ = self.next();
            return self.while_statement();
        }

        // parsing condition
        let condition = self.expression();

        // searching for opening block
        if self.current().token_type != TokenType::LBrace {
            self.error("New block expected after condition!");
            return Statements::None;
        }

        let _ = self.next();

        // parsing statements
        let mut stmts = Vec::new();

        while self.current().token_type != TokenType::RBrace {
            if self.current().token_type == TokenType::EOF {
                self.error(
                    "Unexpected end-of-file in block after `while` statement. Please add '}'!",
                );
                return Statements::None;
            }

            let statement = self.statement();
            stmts.push(statement);
        }

        // skipping brace
        if self.current().token_type == TokenType::RBrace {
            let _ = self.next();
        }

        // skiping semicolon
        self.skip_eos();

        return Statements::WhileStatement {
            condition,
            block: stmts,
        };
    }

    // etc

    fn expressions_enum(
        &mut self,
        start_token_type: TokenType,
        end_token_type: TokenType,
        separator: TokenType,
    ) -> Vec<Expressions> {
        let mut current = self.current();

        match current.token_type.clone() {
            start_token_type => current = self.next(),
            end_token_type => {
                self.error("Unexpected enumeration end");
                return Vec::new();
            }
        }

        let mut output = Vec::new();

        while current.token_type != end_token_type {
            current = self.current();

            if current.token_type == separator {
                let _ = self.next();
            } else if current.token_type == end_token_type {
                let _ = self.next();
                break;
            } else {
                let expression = self.expression();
                output.push(expression);
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
