use tpl_lexer::{
    error::LexerError, error::LexerErrorHandler, token::Token, token_type::TokenType, Lexer,
};

#[test]
fn handling_error_object() {
    let input = String::from("¤");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("error_handling.tpl"));

    let tokens = lexer.tokenize();

    // checking if variable is error-handler
    match tokens {
        Ok(_) => panic!("`tokens` variable isn't error!"),
        Err(_) => {}
    }
}

#[test]
#[should_panic]
fn unwrapping_error() {
    let input = String::from("¤");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("error_handling.tpl"));

    let tokens = lexer.tokenize();

    // catching panic

    tokens.unwrap();
}

#[test]
fn printing_error_info() {
    let input = String::from("¤");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("error_handling.tpl"));

    let tokens = lexer.tokenize();

    // checking if variable is error-handler
    match tokens {
        Ok(_) => panic!("`tokens` variable isn't error!"),
        Err(err) => {
            println!("{}", err.informate());
        }
    }
}

#[test]
fn printing_multiple_errors_info() {
    let input = String::from("¤\n▐\n \n╚\n╟");

    // initializing lexer
    let mut lexer = Lexer::new(input, String::from("error_handling.tpl"));

    let tokens = lexer.tokenize();

    // checking if variable is error-handler
    match tokens {
        Ok(_) => panic!("`tokens` variable isn't error!"),
        Err(err) => {
            println!("{}", err.informate());
        }
    }
}
