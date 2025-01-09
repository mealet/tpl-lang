// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TokenType {
    Identifier, // abc

    Number,  // 123
    String,  // "asd"
    Char, // 'a'
    Boolean, // true/false

    Equal,    // =
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /
    Not,      // !

    Lt,  // <
    Bt,  // >
    Eq,  // ==
    Ne,  // !=
    Or,  // ||
    And, // &&

    Semicolon, // ;
    Ampersand, // &
    Ref, // &_
    Verbar,    // |
    Dot,       // .
    Comma,     // ,
    Quote,     // "
    SingleQuote, // '

    LShift, // <<
    RShift, // >>
    Xor, // ^

    LParen, // (
    RParen, // )

    LBrace, // {
    RBrace, // }

    LBrack, // [
    RBrack, // ]

    Function,
    Keyword,

    EOF,
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
