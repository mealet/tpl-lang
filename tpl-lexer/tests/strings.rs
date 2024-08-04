use tpl_lexer::{token::Token, token_type::TokenType, Lexer};

#[test]
fn tokenize_string() {
    let input = String::from("\"hello world\"");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("strings.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::String, String::from("hello world")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_difficult_string() {
    let input = String::from("\"█string║ \n\r\"");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("strings.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::String, String::from("█string║ \n\r")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}

#[test]
fn tokenize_multiple_strings() {
    let input = String::from("\"hello world\" \"hey hey\" \"hola hola\"");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("strings.tpl"));

    let tokens = lexer.tokenize().unwrap();

    // expected tokens
    let expected = vec![
        Token::new(TokenType::String, String::from("hello world")),
        Token::new(TokenType::String, String::from("hey hey")),
        Token::new(TokenType::String, String::from("hola hola")),
        Token::new(TokenType::EOF, String::new()),
    ];

    println!("{:#?}", tokens);
    assert_eq!(tokens, expected);
}
