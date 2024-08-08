use tpl_lexer::*;
use tpl_parser::*;

fn main() {
    let input = String::from("int a = a + b;");
    let filename = String::from("main.tpl");

    let mut lexer = Lexer::new(input.clone(), filename.clone());
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens, filename.clone(), input.clone());
    let ast = parser.parse();

    match ast {
        Ok(result) => println!("{:#?}", result),
        Err(err) => {
            println!("{}", err.informate())
        }
    }
}
