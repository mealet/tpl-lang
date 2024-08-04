use tpl_lexer::{token::Token, token_type::TokenType, Lexer};

#[test]
fn tokenize_numbers() {
    let input = String::from("1 2 3 123");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("numbers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Number, String::from("1")),
        Token::new(TokenType::Number, String::from("2")),
        Token::new(TokenType::Number, String::from("3")),
        Token::new(TokenType::Number, String::from("123")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_negative_numbers() {
    let input = String::from("-1 -2 -3 -123");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("numbers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Number, String::from("-1")),
        Token::new(TokenType::Number, String::from("-2")),
        Token::new(TokenType::Number, String::from("-3")),
        Token::new(TokenType::Number, String::from("-123")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_numbers_with_underlines() {
    let input = String::from("100_000 1_000_000");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("numbers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Number, String::from("100000")),
        Token::new(TokenType::Number, String::from("1000000")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_negative_numbers_with_underlines() {
    let input = String::from("-100_000 -1_000_000");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("tokenize_numbers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Number, String::from("-100000")),
        Token::new(TokenType::Number, String::from("-1000000")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}
