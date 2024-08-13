use tpl_ir::*;
use tpl_lexer::*;
use tpl_parser::*;

mod compiler;

const OPTIMIZATION_LEVEL: inkwell::OptimizationLevel = inkwell::OptimizationLevel::Default; // Optimization
                                                                                            // set to None, because in any other way `gcc` or `clang` returns error
const RELOC_MODE: inkwell::targets::RelocMode = inkwell::targets::RelocMode::PIC;
const CODE_MODEL: inkwell::targets::CodeModel = inkwell::targets::CodeModel::Default;

fn main() {
    // WARNING: This is a testing code for developing

    let input = String::from("str a = \"Hello World!\";\nprint(a);");
    let filename = String::from("main.tpl");

    let ctx = inkwell::context::Context::create();
    let mut compiler = Compiler::new(&ctx, "main");

    let mut lexer = Lexer::new(input.clone(), filename.clone());
    let tokens = lexer.tokenize().unwrap();

    let mut parser = Parser::new(tokens, filename.clone(), input.clone());
    let ast = parser.parse();

    match ast {
        Ok(stmts) => {
            compiler.generate(stmts);
            compiler.get_module().print_to_stderr();

            let module = compiler.get_module();

            let _ = compiler::ObjectCompiler::compile(
                OPTIMIZATION_LEVEL,
                RELOC_MODE,
                CODE_MODEL,
                module,
                "output.o",
            );
        }
        Err(err) => {
            println!("{}", err.informate())
        }
    }
}

// TODO: Add linker in compiler.rs
// TODO: Optimize and make code more cleaner
// TODO: Boolean types printing like numbers
