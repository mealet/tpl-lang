macro_rules! std_symbol {
    ($ch: literal, $typ: expr) => {
        ($ch, Token::new($typ, String::from($ch), 0))
    };
}

macro_rules! std_function {
    ($name: literal) => {
        (
            $name.to_string(),
            Token::new(TokenType::Function, $name.to_string(), 0),
        )
    };
}

macro_rules! std_keyword {
    ($name: literal) => {
        (
            $name.to_string(),
            Token::new(TokenType::Keyword, $name.to_string(), 0),
        )
    };
}

pub(crate) use std_function;
pub(crate) use std_keyword;
pub(crate) use std_symbol;
