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

    fn error<T: std::fmt::Display>(&mut self, description: T) {
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
                    "for" => {
                        // `for` cycle
                        return self.for_statement();
                    }
                    "define" => {
                        // function definition
                        return self.define_statement();
                    }
                    "return" => {
                        // returning value
                        return self.return_statement();
                    }
                    "break" => {
                        // `break` keyword
                        let _ = self.next();
                        let _ = self.skip_eos();
                        return Statements::BreakStatement { line: current.line };
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
                    TokenType::LParen => self.call_statement(current.value),
                    _ if BINARY_OPERATORS.contains(&next.token_type) => {
                        match self.next().token_type {
                            TokenType::Equal => {
                                // parsing binary assignment
                                return self.binary_assign_statement(current.value, next.value);
                            }
                            TokenType::Plus | TokenType::Minus => {
                                // getting operands
                                let first_operand = next.value;
                                let second_operand = self.current().value;

                                // comparing both

                                if first_operand != second_operand {
                                    self.error(
                                        "Unexpected variation of increment/decrement found!",
                                    );
                                    return Statements::None;
                                }

                                let _ = self.next();
                                let _ = self.skip_eos();

                                // and returning as binary assignment

                                return Statements::BinaryAssignStatement {
                                    identifier: current.value,
                                    operand: first_operand,
                                    value: Some(Box::new(Expressions::Value(Value::Integer(1)))),
                                    line: current.line,
                                };
                            }
                            _ => {
                                self.error("Unexpected Binary Operation in statement found!");
                                return Statements::None;
                            }
                        }
                    }
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
            _ if DATATYPES.contains(&current.value.as_str()) => {
                // parsing argument
                let datatype = current.value;
                let identifier = self.next();

                if !self.expect(TokenType::Identifier) {
                    self.error("Unexpected token found after data-type in expression!");
                    return Expressions::None;
                }

                let _ = self.next();

                return Expressions::Argument {
                    name: identifier.value,
                    datatype,
                };
            }
            _ => {
                self.error("Unexpected term found");
            }
        }

        let _ = self.next();
        return output;
    }

    fn expression(&mut self) -> Expressions {
        let mut node = self.term();
        let current = self.current();

        match current.token_type {
            _ if self.is_binary_operand(current.token_type) => {
                node = self.binary_expression(node);
            }

            TokenType::LParen => {
                // calling function
                if let Expressions::Value(Value::Identifier(val)) = node {
                    return self.call_expression(val);
                } else {
                    self.error("Unexpected parenthesis found after identifier in expression!");
                    return Expressions::None;
                }
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
        let current_line = current_token.line;

        match current_token.token_type.clone() {
            _ if self.is_binary_operand(current_token.token_type) => {
                let _ = self.next();

                let lhs = node;
                let rhs = self.expression();

                if self.is_priority_binary_operand(current_token.clone().value) {
                    let mut new_node = rhs.clone();
                    let old_lhs = lhs.clone();

                    if let Expressions::Binary {
                        lhs,
                        rhs,
                        operand,
                        line,
                    } = new_node
                    {
                        let lhs_new = old_lhs;
                        let rhs_new = lhs;

                        // creating new expression

                        let priority_node = Expressions::Binary {
                            lhs: Box::new(lhs_new),
                            rhs: rhs_new,
                            operand: current_token.clone().value,
                            line: current_line,
                        };

                        let output_node = Expressions::Binary {
                            lhs: Box::new(priority_node),
                            rhs,
                            operand,
                            line: current_line,
                        };

                        return output_node;
                    }
                }

                return Expressions::Binary {
                    operand: current_token.value,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    line: current_line,
                };
            }
            _ => {
                self.error("Unexpected token at binary expression!");
                self.next();
                return Expressions::None;
            }
        }
    }

    fn call_expression(&mut self, function_name: String) -> Expressions {
        let line = self.current().line;

        match self.current().token_type {
            TokenType::Identifier => {
                let _ = self.next();
                return self.call_expression(function_name);
            }
            TokenType::LParen => {}
            _ => {
                self.error("Unexpected variation of call expression!");
                return Expressions::None;
            }
        }

        // parsing arguments
        let arguments =
            self.expressions_enum(TokenType::LParen, TokenType::RParen, TokenType::Comma);
        let _ = self.skip_eos();

        return Expressions::Call {
            function_name,
            arguments,
            line,
        };
    }

    // statements

    fn print_statement(&mut self) -> Statements {
        let mut current = self.current();
        let line = current.line;

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
            line,
        };
    }

    fn annotation_statement(&mut self) -> Statements {
        let line = self.current().line;

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
                        line,
                    };
                }
                END_STATEMENT => {
                    self.skip_eos();

                    return Statements::AnnotationStatement {
                        identifier: id,
                        datatype,
                        value: None,
                        line,
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
        let line = self.current().line;

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
                    line,
                };
            }
        }
    }

    fn binary_assign_statement(&mut self, identifier: String, operand: String) -> Statements {
        let line = self.current().line;

        match self.current().token_type {
            TokenType::Equal => {
                self.next();
                return self.binary_assign_statement(identifier, operand);
            }
            END_STATEMENT => {
                self.error("Expressions expected in binary assignment, but `;` found!");
                self.next();
                return Statements::None;
            }
            _ => {
                return Statements::BinaryAssignStatement {
                    identifier,
                    operand,
                    value: Some(Box::new(self.expression())),
                    line,
                }
            }
        }
    }

    fn if_statement(&mut self) -> Statements {
        let line = self.current().line;

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
                    line,
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
                    line,
                };
            }
        }
    }

    fn while_statement(&mut self) -> Statements {
        let line = self.current().line;

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
            line,
        };
    }

    fn for_statement(&mut self) -> Statements {
        let line = self.current().line;

        if self.current().token_type == TokenType::Keyword {
            // skipping keyword
            let _ = self.next();
            return self.for_statement();
        }

        // getting variable name
        if !self.expect(TokenType::Identifier) {
            self.error("Variable name expected after keyword `for`!");
            return Statements::None;
        }

        let varname = self.current().value;

        // searching for `in` keyword

        let keyword = self.next();

        if let (TokenType::Keyword, "in") = (keyword.token_type, keyword.value.as_str()) {
            let _ = self.next();

            // parsing iter object
            let iterable_object = self.expression();

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
                        "Unexpected end-of-file in block after `for` statement. Please add '}'!",
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

            return Statements::ForStatement {
                varname,
                iterable_object,
                block: stmts,
                line,
            };
        } else {
            self.error("Expected keyword 'in` after variable name in `for` statement!");
            return Statements::None;
        }
    }

    fn call_statement(&mut self, function_name: String) -> Statements {
        let line = self.current().line;

        match self.current().token_type {
            TokenType::Identifier => {
                let _ = self.next();
                return self.call_statement(function_name);
            }
            TokenType::LParen => {}
            _ => {
                self.error("Unexpected variation of call statement!");
                return Statements::None;
            }
        }

        // parsing arguments
        let arguments =
            self.expressions_enum(TokenType::LParen, TokenType::RParen, TokenType::Comma);
        let _ = self.skip_eos();

        return Statements::FunctionCallStatement {
            function_name,
            arguments,
            line,
        };
    }

    fn define_statement(&mut self) -> Statements {
        let line = self.current().line;

        match self.current().token_type {
            TokenType::Keyword => {
                if self.current().value == String::from("define") {
                    let _ = self.next();
                }

                if !DATATYPES.contains(&self.current().value.as_str()) {
                    self.error("Unexpected keyword found after `define`!");
                    return Statements::None;
                }

                // keeping datatype
                let function_type = self.current().value;

                // searching for the function name
                let identifier = self.next();

                if !self.expect(TokenType::Identifier) {
                    self.error("Identifier for function expected, but found anything else!");
                    return Statements::None;
                }

                // getting function name
                let function_name = identifier.value;

                // getting arguments
                let _ = self.next();
                let args =
                    self.expressions_enum(TokenType::LParen, TokenType::RParen, TokenType::Comma);

                let mut arguments_tuples = Vec::new();

                // checking for right arguments definition
                if args.len() > 0 {
                    for arg in args {
                        match arg {
                            Expressions::Argument { name, datatype } => {
                                arguments_tuples.push((name, datatype));
                            }
                            _ => {
                                self.error("All arguments in definition must be `type name` (example: `int a`)");
                                return Statements::None;
                            }
                        }
                    }
                }

                // parsing block
                if !self.expect(TokenType::LBrace) {
                    self.error("Expected block with code after function declaration!");
                    return Statements::None;
                }

                let _ = self.next();

                let mut stmts = Vec::new();

                while self.current().token_type != TokenType::RBrace {
                    if self.current().token_type == TokenType::EOF {
                        self.error(
                        "Unexpected end-of-file in block after `for` statement. Please add '}'!",
                    );
                        return Statements::None;
                    }

                    let statement = self.statement();
                    stmts.push(statement);
                }

                // skipping brace and semicolon
                if self.current().token_type == TokenType::RBrace {
                    let _ = self.next();
                }

                let _ = self.skip_eos();

                // returning function

                return Statements::FunctionDefineStatement {
                    function_name,
                    function_type,
                    arguments: arguments_tuples,
                    block: stmts,
                    line,
                };
            }
            _ => {
                self.error("Unexpected variation of defining function");
                return Statements::None;
            }
        }
    }

    fn return_statement(&mut self) -> Statements {
        if self.current().token_type == TokenType::Keyword {
            let _ = self.next();
        }

        let line = self.current().line;
        let value = self.expression();

        let _ = self.skip_eos();

        return Statements::ReturnStatement { value, line };
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
