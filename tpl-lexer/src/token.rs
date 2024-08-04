// ------------------------
// Toy Programming Language
// ------------------------

use crate::token_type::TokenType;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub value: String,
    pub token_type: TokenType,
}

impl Token {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Token { value, token_type }
    }
}
