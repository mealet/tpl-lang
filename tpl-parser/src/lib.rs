// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

mod error;
pub mod expressions;
pub mod statements;
pub mod value;

use error::ParseErrorHandler;
use lazy_static::lazy_static;
use tpl_lexer::{token::Token, token_type::TokenType};

use expressions::Expressions;
use statements::Statements;
use value::Value;

// globals

lazy_static! {
    static ref DATATYPES: Vec<&'static str> = vec![
        "int8",
        "int16",
        "int32",
        "int64",
        "int128",

        "str",
        "bool",

        "auto",
        "void",
        "fn"
    ];
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
            self.position,
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
                    "import" => {
                        // file import
                        self.import_statement()
                    }

                    "if" => {
                        // `if` or `if/else` construction
                        self.if_statement()
                    }
                    "else" => {
                        self.error(
                            "Unexpected `else` usage. Please use it in `if/else` construction!",
                        );
                        Statements::None
                    }

                    "while" => {
                        // `while` cycle
                        self.while_statement()
                    }
                    "for" => {
                        // `for` cycle
                        self.for_statement()
                    }

                    "define" => {
                        // function definition
                        self.define_statement()
                    }
                    "return" => {
                        // returning value
                        self.return_statement()
                    }

                    "break" => {
                        // `break` keyword
                        let _ = self.next();
                        self.skip_eos();
                        Statements::BreakStatement { line: current.line }
                    }
                    _ => Statements::None,
                }
            }
            TokenType::Function => self.function_call_statement(current.value),
            TokenType::Identifier => {
                let next = self.next();

                match next.token_type {
                    TokenType::Equal => self.assign_statement(current.value),
                    TokenType::LParen => self.call_statement(current.value),
                    _ if BINARY_OPERATORS.contains(&next.token_type) => {
                        match self.next().token_type {
                            TokenType::Equal => {
                                // parsing binary assignment
                                self.binary_assign_statement(current.value, next.value)
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
                                self.skip_eos();

                                // and returning as binary assignment

                                Statements::BinaryAssignStatement {
                                    identifier: current.value,
                                    operand: first_operand,
                                    value: Some(Box::new(Expressions::Value(Value::Integer(1)))),
                                    line: current.line,
                                }
                            }
                            _ => {
                                self.error("Unexpected Binary Operation in statement found!");
                                Statements::None
                            }
                        }
                    }
                    END_STATEMENT => {
                        Statements::Expression(Expressions::Value(Value::Identifier(current.value)))
                    }
                    _ => {
                        self.error("Unexpected expression/statement after identifier");
                        self.next();
                        Statements::None
                    }
                }
            }
            TokenType::EOF => {
                self.eof = true;
                Statements::None
            }
            _ => Statements::Expression(self.expression()),
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
                output = Expressions::Value(Value::Boolean(current.value == "true"))
            }
            TokenType::Identifier => {
                output = Expressions::Value(Value::Identifier(current.value.clone()));

                let next = self.next();

                if let TokenType::LParen = next.token_type {
                    // calling function
                    return self.call_expression(current.value);
                }

                return output;
            }
            _ if DATATYPES.contains(&current.value.as_str()) => {
                // parsing argument
                let datatype = current.value;
                let identifier = self.next();

                if !self.expect(TokenType::Identifier) {
                    return Expressions::Value(Value::Keyword(datatype));
                }

                let _ = self.next();

                return Expressions::Argument {
                    name: identifier.value,
                    datatype,
                };
            }
            TokenType::Function => {
                return self.call_expression(current.value);
            }
            TokenType::Keyword => {
                return Expressions::Value(Value::Keyword(current.value));
            }
            _ => {
                self.error(format!(
                    "Unexpected term '{:?}' found",
                    self.current().value
                ));
                let _ = self.next();
                return Expressions::None;
            }
        }

        let _ = self.next();
        output
    }

    fn expression(&mut self) -> Expressions {
        let mut node = self.term();
        let current = self.current();

        match current.token_type {
            _ if self.is_binary_operand(current.token_type) => {
                node = self.binary_expression(node);
            }
            TokenType::LParen => {
                if let Expressions::Value(Value::Keyword(keyword)) = node.clone() {
                    if !DATATYPES.contains(&keyword.as_str()) {
                        self.error(format!("Unexpected keyword `{}` in expression", keyword));
                        let _ = self.next();
                        return Expressions::None;
                    }

                    let lambda_arguments = self.expressions_enum(
                        TokenType::LParen,
                        TokenType::RParen,
                        TokenType::Comma,
                    );
                    let lambda_type = keyword;
                    let mut function_statements: Vec<Statements> = Vec::new();

                    let mut arguments_tuples = Vec::new();

                    // checking for right arguments definition
                    if !lambda_arguments.is_empty() {
                        for arg in lambda_arguments {
                            match arg {
                                Expressions::Argument { name, datatype } => {
                                    arguments_tuples.push((name, datatype));
                                }
                                _ => {
                                    self.error("All arguments in definition must be `type name` (example: `int32 a`)");
                                    return Expressions::None;
                                }
                            }
                        }
                    }

                    if !self.expect(TokenType::LBrace) {
                        self.error("Expected block after lambda function definition!");
                        let _ = self.next();
                        return Expressions::None;
                    }

                    let _ = self.next();

                    while !self.expect(TokenType::RBrace) {
                        if self.expect(TokenType::RBrace) {
                            break;
                        }

                        function_statements.push(self.statement());
                    }

                    if self.expect(TokenType::RBrace) {
                        let _ = self.next();
                    }

                    return Expressions::Lambda {
                        arguments: arguments_tuples,
                        statements: function_statements,
                        ftype: lambda_type,
                        line: current.line,
                    };
                }

                self.error("Unexpected parentheses in expression found".to_string());
                let _ = self.next();
                return Expressions::None;
            }
            END_STATEMENT => {
                self.next();
            }
            _ => {}
        }

        node
    }

    // expressions

    fn binary_expression(&mut self, node: Expressions) -> Expressions {
        let current_token = self.current();
        let current_line = current_token.line;

        match current_token.token_type {
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

                Expressions::Binary {
                    operand: current_token.value,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                    line: current_line,
                }
            }
            _ => {
                self.error("Unexpected token at binary expression!");
                self.next();
                Expressions::None
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
            TokenType::Function => {
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

        self.skip_eos();

        Expressions::Call {
            function_name,
            arguments,
            line,
        }
    }

    // statements

    fn function_call_statement(&mut self, function_name: String) -> Statements {
        let mut current = self.current();
        let line = current.line;

        match current.token_type {
            TokenType::Function => {
                current = self.next();
                return self.function_call_statement(function_name);
            }
            TokenType::LParen => {}
            _ => {
                self.error(format!("Unexpected usage of `{}` statement", function_name));
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

        Statements::FunctionCallStatement {
            function_name,
            arguments,
            line,
        }
    }

    fn annotation_statement(&mut self) -> Statements {
        let line = self.current().line;

        if DATATYPES.contains(&self.current().value.as_str()) {
            let mut datatype = self.current().value;
            let _ = self.next();

            if self.expect(TokenType::Lt) {
                // example: fn<int32>

                // In future there must be a special function for parsing nested datatypes

                let _ = self.next();

                if !self.expect(TokenType::Keyword) {
                    self.error("Unexpected nested datatype found!");
                    self.next();

                    return Statements::None;
                }

                let subtype = self.current().value;
                let _ = self.next();

                if !self.expect(TokenType::Bt) {
                    self.error("Wrong nested type definition! Must be like: fn<int32>");
                    self.next();

                    return Statements::None;
                }

                let _ = self.next();
                datatype = format!("{}<{}>", datatype, subtype);
            }

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

                    Statements::AnnotationStatement {
                        identifier: id,
                        datatype,
                        value: Some(Box::new(value)),
                        line,
                    }
                }
                END_STATEMENT => {
                    self.skip_eos();

                    Statements::AnnotationStatement {
                        identifier: id,
                        datatype,
                        value: None,
                        line,
                    }
                }
                _ => {
                    self.error("Expected `=` or `;` after variable annotation");

                    self.next();
                    Statements::None
                }
            }
        } else {
            Statements::None
        }
    }

    fn assign_statement(&mut self, identifier: String) -> Statements {
        let line = self.current().line;

        match self.current().token_type {
            TokenType::Equal => {
                self.next();
                self.assign_statement(identifier)
            }
            END_STATEMENT => {
                self.error("Expressions expected in assign statement, but `;` found!");
                self.next();
                Statements::None
            }
            _ => Statements::AssignStatement {
                identifier,
                value: Some(Box::new(self.expression())),
                line,
            },
        }
    }

    fn binary_assign_statement(&mut self, identifier: String, operand: String) -> Statements {
        let line = self.current().line;

        match self.current().token_type {
            TokenType::Equal => {
                self.next();
                self.binary_assign_statement(identifier, operand)
            }
            END_STATEMENT => {
                self.error("Expressions expected in binary assignment, but `;` found!");
                self.next();
                Statements::None
            }
            _ => Statements::BinaryAssignStatement {
                identifier,
                operand,
                value: Some(Box::new(self.expression())),
                line,
            },
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
                if current_token.value != *"else" {
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

                self.skip_eos();

                Statements::IfStatement {
                    condition,
                    then_block: stmts,
                    else_block: Some(else_stmts),
                    line,
                }
            }
            _ => {
                // skipping semicolon if we have
                self.skip_eos();
                // returning statement
                Statements::IfStatement {
                    condition,
                    then_block: stmts,
                    else_block: None,
                    line,
                }
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

        Statements::WhileStatement {
            condition,
            block: stmts,
            line,
        }
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

            Statements::ForStatement {
                varname,
                iterable_object,
                block: stmts,
                line,
            }
        } else {
            self.error("Expected keyword 'in` after variable name in `for` statement!");
            Statements::None
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
        self.skip_eos();

        Statements::FunctionCallStatement {
            function_name,
            arguments,
            line,
        }
    }

    fn define_statement(&mut self) -> Statements {
        let line = self.current().line;

        match self.current().token_type {
            TokenType::Keyword => {
                if self.current().value == *"define" {
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
                if !args.is_empty() {
                    for arg in args {
                        match arg {
                            Expressions::Argument { name, datatype } => {
                                arguments_tuples.push((name, datatype));
                            }
                            _ => {
                                self.error("All arguments in definition must be `type name` (example: `int32 a`)");
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

                self.skip_eos();

                // returning function

                Statements::FunctionDefineStatement {
                    function_name,
                    function_type,
                    arguments: arguments_tuples,
                    block: stmts,
                    line,
                }
            }
            _ => {
                self.error("Unexpected variation of defining function");
                Statements::None
            }
        }
    }

    fn return_statement(&mut self) -> Statements {
        if self.current().token_type == TokenType::Keyword {
            let _ = self.next();
        }

        let line = self.current().line;
        let value = self.expression();

        self.skip_eos();

        Statements::ReturnStatement { value, line }
    }

    fn import_statement(&mut self) -> Statements {
        if self.current().token_type == TokenType::Keyword {
            let _ = self.next();
        }

        let line = self.current().line;
        let path = self.expression();

        self.skip_eos();

        // checking if path is string
        if let Expressions::Value(Value::String(_)) = path {
            Statements::ImportStatement { path, line }
        } else {
            self.error("Unexpected import value found!");
            Statements::None
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

            if current.token_type == separator {
                let _ = self.next();
            } else if current.token_type == end_token_type {
                break;
            } else {
                let expression = self.expression();
                output.push(expression);
            }
        }

        if self.current().token_type == end_token_type {
            let _ = self.next();
        }

        output
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
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tpl_lexer::{token::Token, token_type::TokenType, Lexer};

    #[test]
    fn peek_fn_test() {
        let input = String::from("a b");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);

        assert_eq!(
            parser.peek(0),
            Token::new(TokenType::Identifier, String::from("a"), 0)
        );

        assert_eq!(
            parser.peek(1),
            Token::new(TokenType::Identifier, String::from("b"), 0)
        );

        assert_eq!(
            parser.peek(1),
            Token::new(TokenType::EOF, String::from(""), 0)
        );
    }

    #[test]
    fn next_fn_test() {
        let input = String::from("a b");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);

        assert_eq!(
            parser.next(),
            Token::new(TokenType::Identifier, String::from("b"), 0)
        );

        assert_eq!(
            parser.next(),
            Token::new(TokenType::EOF, String::from(""), 0)
        );
    }

    #[test]
    fn current_fn_test() {
        let input = String::from("a b");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);

        assert_eq!(
            parser.current(),
            Token::new(TokenType::Identifier, String::from("a"), 0)
        );

        let _ = parser.next();

        assert_eq!(
            parser.current(),
            Token::new(TokenType::Identifier, String::from("b"), 0)
        );

        let _ = parser.next();

        assert_eq!(
            parser.current(),
            Token::new(TokenType::EOF, String::from(""), 0)
        );
    }

    #[test]
    fn expect_fn_test() {
        let input = String::from("a b");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);

        assert!(parser.expect(TokenType::Identifier));

        let _ = parser.next();

        assert!(parser.expect(TokenType::Identifier));

        let _ = parser.next();

        assert!(parser.expect(TokenType::EOF));
    }

    #[test]
    fn is_bin_operand_test() {
        let input = String::from("a b");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let parser = Parser::new(tokens, "test".to_string(), input);

        assert!(parser.is_binary_operand(TokenType::Plus));
        assert!(parser.is_binary_operand(TokenType::Minus));
        assert!(parser.is_binary_operand(TokenType::Multiply));
        assert!(parser.is_binary_operand(TokenType::Divide));
        assert!(parser.is_binary_operand(TokenType::Eq));
        assert!(parser.is_binary_operand(TokenType::Ne));
        assert!(parser.is_binary_operand(TokenType::Lt));
        assert!(parser.is_binary_operand(TokenType::Bt));
    }

    #[test]
    fn is_priority_bin_operand_test() {
        let input = String::from("a b");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let parser = Parser::new(tokens, "test".to_string(), input);

        assert!(parser.is_priority_binary_operand("/".to_string()));
        assert!(parser.is_priority_binary_operand("*".to_string()));
        assert!(!parser.is_priority_binary_operand("+".to_string()));
        assert!(!parser.is_priority_binary_operand("-".to_string()));
    }

    #[test]
    fn skip_eos_fn_test() {
        let input = String::from("a; b");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);

        assert!(parser.expect(TokenType::Identifier));
        let _ = parser.next();

        assert!(parser.expect(TokenType::Semicolon));
        parser.skip_eos();

        assert!(parser.expect(TokenType::Identifier));
        parser.skip_eos();

        assert!(parser.expect(TokenType::Identifier));
    }

    #[test]
    fn annotation_stmt_test() {
        let input = String::from("int32 a = 5;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::AnnotationStatement {
                identifier: String::from("a"),
                datatype: String::from("int32"),
                value: Some(Box::new(Expressions::Value(Value::Integer(5)))),
                line: 0
            }
        );
    }

    #[test]
    fn assign_stmt_test() {
        let input = String::from("a = 5;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::AssignStatement {
                identifier: String::from("a"),
                value: Some(Box::new(Expressions::Value(Value::Integer(5)))),
                line: 0
            }
        );
    }

    #[test]
    fn binary_assign_stmt_test() {
        let input = String::from("a += 5;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::BinaryAssignStatement {
                identifier: String::from("a"),
                value: Some(Box::new(Expressions::Value(Value::Integer(5)))),
                operand: String::from("+"),
                line: 0
            }
        );
    }

    #[test]
    fn function_define_stmt_test() {
        let input = String::from("define int8 foo() {};");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::FunctionDefineStatement {
                function_name: String::from("foo"),
                function_type: String::from("int8"),
                arguments: Vec::new(),
                block: Vec::new(),
                line: 0
            }
        );
    }

    #[test]
    fn function_define_with_args_stmt_test() {
        let input = String::from("define int8 foo(int8 a, int8 b) {};");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::FunctionDefineStatement {
                function_name: String::from("foo"),
                function_type: String::from("int8"),
                arguments: vec![
                    ("a".to_string(), "int8".to_string()),
                    ("b".to_string(), "int8".to_string()),
                ],
                block: Vec::new(),
                line: 0
            }
        );
    }

    #[test]
    fn function_define_with_block_stmt_test() {
        let input = String::from("define int8 foo() { a = 5 };");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::FunctionDefineStatement {
                function_name: String::from("foo"),
                function_type: String::from("int8"),
                arguments: Vec::new(),
                block: vec![Statements::AssignStatement {
                    identifier: "a".to_string(),
                    value: Some(Box::new(Expressions::Value(Value::Integer(5)))),
                    line: 0
                }],
                line: 0
            }
        );
    }

    #[test]
    fn function_define_with_block_and_args_stmt_test() {
        let input = String::from("define int8 foo(int8 a, int8 b) { a = 5 };");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::FunctionDefineStatement {
                function_name: String::from("foo"),
                function_type: String::from("int8"),
                arguments: vec![
                    ("a".to_string(), "int8".to_string()),
                    ("b".to_string(), "int8".to_string()),
                ],
                block: vec![Statements::AssignStatement {
                    identifier: "a".to_string(),
                    value: Some(Box::new(Expressions::Value(Value::Integer(5)))),
                    line: 0
                }],
                line: 0
            }
        );
    }

    #[test]
    fn function_call_stmt_test() {
        let input = String::from("foo()");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::FunctionCallStatement {
                function_name: String::from("foo"),
                arguments: Vec::new(),
                line: 0
            }
        );
    }

    #[test]
    fn function_call_with_args_stmt() {
        let input = String::from("foo(5, 1, 4)");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::FunctionCallStatement {
                function_name: String::from("foo"),
                arguments: vec![
                    Expressions::Value(Value::Integer(5)),
                    Expressions::Value(Value::Integer(1)),
                    Expressions::Value(Value::Integer(4))
                ],
                line: 0
            }
        );
    }

    #[test]
    fn function_call_with_advanced_args_stmt() {
        let input = String::from("foo(5 + 6, 2 * 2)");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::FunctionCallStatement {
                function_name: String::from("foo"),
                arguments: vec![
                    Expressions::Binary {
                        operand: String::from("+"),
                        lhs: Box::new(Expressions::Value(Value::Integer(5))),
                        rhs: Box::new(Expressions::Value(Value::Integer(6))),
                        line: 0
                    },
                    Expressions::Binary {
                        operand: String::from("*"),
                        lhs: Box::new(Expressions::Value(Value::Integer(2))),
                        rhs: Box::new(Expressions::Value(Value::Integer(2))),
                        line: 0
                    },
                ],
                line: 0
            }
        );
    }

    #[test]
    fn function_call_expr_test() {
        let input = String::from("int32 a = foo(5 + 6, 2 * 2);");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::AnnotationStatement {
                identifier: String::from("a"),
                datatype: String::from("int32"),
                value: Some(Box::new(Expressions::Call {
                    function_name: String::from("foo"),
                    arguments: vec![
                        Expressions::Binary {
                            operand: String::from("+"),
                            lhs: Box::new(Expressions::Value(Value::Integer(5))),
                            rhs: Box::new(Expressions::Value(Value::Integer(6))),
                            line: 0
                        },
                        Expressions::Binary {
                            operand: String::from("*"),
                            lhs: Box::new(Expressions::Value(Value::Integer(2))),
                            rhs: Box::new(Expressions::Value(Value::Integer(2))),
                            line: 0
                        },
                    ],
                    line: 0
                })),
                line: 0
            }
        );
    }

    #[test]
    fn if_stmt_test() {
        let input = String::from("if 1 < 2 {};");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::IfStatement {
                condition: Expressions::Binary {
                    operand: String::from("<"),
                    lhs: Box::new(
                        Expressions::Value(Value::Integer(1))
                    ),
                    rhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    line: 0
                },
                then_block: Vec::new(),
                else_block: None,
                line: 0
            }
        );
    }

    #[test]
    fn if_else_stmt_test() {
        let input = String::from("if 1 < 2 {} else {};");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::IfStatement {
                condition: Expressions::Binary {
                    operand: String::from("<"),
                    lhs: Box::new(
                        Expressions::Value(Value::Integer(1))
                    ),
                    rhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    line: 0
                },
                then_block: Vec::new(),
                else_block: Some(Vec::new()),
                line: 0
            }
        );
    }

    #[test]
    fn if_else_with_blocks_stmt_test() {
        let input = String::from("if 1 < 2 { return 1; } else { return 2 };");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!( ast[0],
            Statements::IfStatement {
                condition: Expressions::Binary {
                    operand: String::from("<"),
                    lhs: Box::new(
                        Expressions::Value(Value::Integer(1))
                    ),
                    rhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    line: 0
                },
                then_block: vec![
                    Statements::ReturnStatement {
                        value: Expressions::Value(Value::Integer(1)),
                        line: 0
                    }
                ],
                else_block: Some(vec![
                    Statements::ReturnStatement {
                        value: Expressions::Value(Value::Integer(2)),
                        line: 0
                }]),
                line: 0
            }
        );
    }

    #[test]
    fn return_stmt_test() {
        let input = String::from("return 0;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::ReturnStatement {
                value: Expressions::Value(Value::Integer(0)),
                line: 0
            }
        );
    }

    #[test]
    fn return_advanced_stmt_test() {
        let input = String::from("return 2 + 2;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::ReturnStatement {
                value: Expressions::Binary {
                    operand: String::from("+"),
                    lhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    rhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    line: 0
                },
                line: 0
            }
        );
    }

    #[test]
    fn binary_operations_test() {
        let input = String::from("2 + 2;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::Expression (
                Expressions::Binary {
                    operand: String::from("+"),
                    lhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    rhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    line: 0
                }
            )
        );
    }

    #[test]
    fn binary_operations_advanced_test() {
        let input = String::from("2 + 2 * 2;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::Expression (
                Expressions::Binary {
                    operand: String::from("+"),
                    lhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    rhs: Box::new(
                        Expressions::Binary {
                            operand: String::from("*"),
                            lhs: Box::new(
                                Expressions::Value(Value::Integer(2))
                            ),
                            rhs: Box::new(
                                Expressions::Value(Value::Integer(2))
                            ),
                            line: 0
                        }
                    ),
                    line: 0
                }
            )
        );
    }

    #[test]
    fn while_stmt_test() {
        let input = String::from("while 1 < 2 {};");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::WhileStatement {
                condition: Expressions::Binary {
                    operand: String::from("<"),
                    lhs: Box::new(
                        Expressions::Value(Value::Integer(1))
                    ),
                    rhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    line: 0
                },
                block: Vec::new(),
                line: 0
            }
        );
    }

    #[test]
    fn while_with_block_stmt_test() {
        let input = String::from("while 1 < 2 { break };");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::WhileStatement {
                condition: Expressions::Binary {
                    operand: String::from("<"),
                    lhs: Box::new(
                        Expressions::Value(Value::Integer(1))
                    ),
                    rhs: Box::new(
                        Expressions::Value(Value::Integer(2))
                    ),
                    line: 0
                },
                block: vec![
                    Statements::BreakStatement { line: 0 }
                ],
                line: 0
            }
        );
    }

    #[test]
    fn break_stmt_test() {
        let input = String::from("break");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::BreakStatement {
                line: 0
            }
        );
    }

    #[test]
    fn for_stmt_test() {
        let input = String::from("for i in 10 {};");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t, Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::ForStatement {
                varname: String::from("i"),
                iterable_object: Expressions::Value(Value::Integer(10)),
                block: Vec::new(),
                line: 0
            }
        );
    }

    #[test]
    fn for_with_block_stmt_test() {
        let input = String::from("for i in 10 { break };");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::ForStatement {
                varname: String::from("i"),
                iterable_object: Expressions::Value(Value::Integer(10)),
                block: vec![
                    Statements::BreakStatement { line: 0 }
                ],
                line: 0
            }
        );
    }

    #[test]
    fn import_statement() {
        let input = String::from("import \"std.tpl\"");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::ImportStatement {
                path: Expressions::Value(Value::String("std.tpl".to_string())),
                line: 0
            }
        );
    }

    #[test]
    fn lambda_expr_test() {
        let input = String::from("fn<int8> a = int8 (int8 a, int8 b) { return 0 };");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse().unwrap();

        assert_eq!(
            ast[0],
            Statements::AnnotationStatement {
                identifier: String::from("a"),
                datatype: String::from("fn<int8>"),
                value: Some(Box::new(
                    Expressions::Lambda {
                        arguments: vec![
                            ("a".to_string(), "int8".to_string()),
                            ("b".to_string(), "int8".to_string()),
                        ],
                        statements: vec![
                            Statements::ReturnStatement {
                                value: Expressions::Value(Value::Integer(0)),
                                line: 0
                            }
                        ],
                        ftype: String::from("int8"),
                        line: 0
                    }
                )),
                line: 0
            }
        );
    }

    #[test]
    fn expressions_enum_test() {
        let input = String::from("(1, true, \"a\")");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.expressions_enum(TokenType::LParen, TokenType::RParen, TokenType::Comma);

        assert_eq!(
            ast,
            vec![
                Expressions::Value(Value::Integer(1)),
                Expressions::Value(Value::Boolean(true)),
                Expressions::Value(Value::String("a".to_string())),
            ]
        );
    }

    #[test]
    fn expressions_enum_test_2() {
        let input = String::from("[1; true; \"a\"]");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.expressions_enum(TokenType::LBrack, TokenType::RBrack, TokenType::Semicolon);

        assert_eq!(
            ast,
            vec![
                Expressions::Value(Value::Integer(1)),
                Expressions::Value(Value::Boolean(true)),
                Expressions::Value(Value::String("a".to_string())),
            ]
        );
    }

    #[test]
    fn error_test() {
        let input = String::from("int0 a;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let ast = parser.parse();

        assert!(ast.is_err());
    }

    #[test]
    #[should_panic]
    fn should_panic_test() {
        let input = String::from("int0 a;");
        let mut lexer = Lexer::new(input.clone(), "test".to_string());

        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => panic!("Lexer side error occured!"),
        };

        let mut parser = Parser::new(tokens, "test".to_string(), input);
        let _ = parser.parse().unwrap();
    }
}
