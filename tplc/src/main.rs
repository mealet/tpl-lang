use tpl_ir::*;
use tpl_lexer::*;
use tpl_parser::*;

fn main() {
    // WARNING: This is a testing code for developing

    let input = String::from("int a = 1;\nstr b = \"hello\";\nprint(a + b);");
    let filename = String::from("main.tpl");

    let mut lexer = Lexer::new(input.clone(), filename.clone());
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens, filename.clone(), input.clone());
    let ast = parser.parse();

    match ast {
        Ok(stmts) => {}
        Err(err) => {
            println!("{}", err.informate())
        }
    }
}
