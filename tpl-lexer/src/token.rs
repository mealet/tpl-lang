// ------------------------
// Toy Programming Language
// ------------------------

use crate::token_type::TokenType;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub value: String,
    pub token_type: TokenType,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, value: String, line: usize) -> Self {
        Token {
            value,
            token_type,
            line,
        }
    }
}
