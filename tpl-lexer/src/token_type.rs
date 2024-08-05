// ------------------------
// Toy Programming Language
// ------------------------

#[derive(Debug, Eq, PartialEq, Clone)]
#[allow(unused)]
pub enum TokenType {
    Identifier,

    Number,
    String,
    Boolean,

    Equal,
    Plus,
    Minus,
    Multiply,
    Divide,

    Semicolon,
    Dot,
    Quote,

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
