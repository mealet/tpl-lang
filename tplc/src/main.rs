// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

use tpl_ir::*;
use tpl_lexer::*;
use tpl_parser::*;

use colored::Colorize;

mod compiler;
mod greeting;

const OPTIMIZATION_LEVEL: inkwell::OptimizationLevel = inkwell::OptimizationLevel::Default;
const RELOC_MODE: inkwell::targets::RelocMode = inkwell::targets::RelocMode::PIC;
const CODE_MODEL: inkwell::targets::CodeModel = inkwell::targets::CodeModel::Default;

const COMMENTS_START: &str = "//";

struct Config {
    pub input: String,
    pub output: String,
    pub source: String,
}

impl Config {
    fn parse(arguments: Vec<String>) -> Result<Self, String> {
        // checking arguments count

        if arguments.len() < 3 {
            return Err(String::from("Not enough arguments! See `Usage`."));
        }

        // getting source code
        let source_file = arguments[1].clone();
        let source = match std::fs::read_to_string(source_file) {
            Ok(code) => code,
            Err(_) => {
                return Err(String::from(
                    "Error with parsing source code! Check file and try again.",
                ))
            }
        };

        // deleting comments

        let formatted_source = source
            .lines()
            .map(|line| {
                if let Some(index) = line.find(COMMENTS_START) {
                    &line[..index]
                } else {
                    line
                }
            })
            .collect::<Vec<&str>>()
            .join("\n");

        // returning config

        Ok(Self {
            input: arguments[1].clone(),
            output: arguments[2].clone(),
            source: formatted_source,
        })
    }
}

fn main() {
    // greeting user
    greeting::print_greeting();

    // trying parse config
    let arguments = std::env::args().collect();
    let config = match Config::parse(arguments) {
        Ok(config) => config,
        Err(e) => {
            // if error just print usage and error to user
            greeting::print_usage();

            println!("| {} {}", "error:".red(), e);

            std::process::exit(1);
        }
    };

    // creating llvm context and compiler

    let ctx = inkwell::context::Context::create();
    let mut compiler = Compiler::new(&ctx, config.output.as_str());

    // creating lexical analyzer and getting tokens

    let mut lexer = Lexer::new(config.source.clone(), config.output.clone());
    let tokens = match lexer.tokenize() {
        Ok(tokens) => tokens,
        Err(e) => {
            let info = e.informate();
            eprintln!("{}", info);
            std::process::exit(1);
        }
    };

    // creating parser and getting Abstract Syntax Tree

    let mut parser = Parser::new(tokens, config.input.clone(), config.source.clone());
    let ast = parser.parse();

    // catching errors

    match ast {
        Ok(stmts) => {
            // compiling statements to module
            let _ = compiler.generate(stmts);
            let module = compiler.get_module();

            // // debug
            // let _ = module.print_to_stderr();

            // compiling module to object file

            let object_file = format!("{}.o", config.output.clone());

            let _ = compiler::ObjectCompiler::compile(
                OPTIMIZATION_LEVEL,
                RELOC_MODE,
                CODE_MODEL,
                module,
                object_file.as_str(),
            );

            // linking and deleting object file

            let _ = compiler::ObjectLinker::compile(object_file.clone(), &config.output.clone());

            let _ = std::fs::remove_file(object_file);
        }
        Err(err) => {
            // printing all errors in terminal and quitting
            eprintln!("{}", err.informate());
            std::process::exit(1);
        }
    }
}

// TODO: Create `for` cycle.
// TODO: Add showing line when throwing GenError
