use tpl_lexer::{token::Token, token_type::TokenType, Lexer};

#[test]
fn tokenize_id() {
    let input = String::from("a");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("identifiers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Identifier, String::from("a")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_longer_id() {
    let input = String::from("abcdefgh");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("identifiers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Identifier, String::from("abcdefgh")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_numeric_id() {
    let input = String::from("a123");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("identifiers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Identifier, String::from("a123")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_harder_id() {
    let input = String::from("example_identifier-1");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("identifiers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Identifier, String::from("example_identifier-1")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_multiple_ids() {
    let input = String::from("id1 id2 id3");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("identifiers.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::Identifier, String::from("id1")),
        Token::new(TokenType::Identifier, String::from("id2")),
        Token::new(TokenType::Identifier, String::from("id3")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}
