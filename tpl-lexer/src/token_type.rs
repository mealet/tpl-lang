// ------------------------
// Toy Programming Language
// ------------------------

#[derive(Debug, Eq, PartialEq, Clone)]
#[allow(unused)]
pub enum TokenType {
    Identifier, // abc

    Number,  // 123
    String,  // "asd"
    Boolean, // true/false

    Equal,    // =
    Plus,     // +
    Minus,    // -
    Multiply, // *
    Divide,   // /

    Semicolon, // ;
    Dot,       // .
    Comma,     // ,
    Quote,     // "

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

#[allow(unused)]
impl TokenType {
    pub fn to_string(&self) -> String {
        format!("{:?}", self.clone())
    }
}
