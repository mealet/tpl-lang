// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

use colored::Colorize;
use std::{path::Path, process::Command};

use inkwell::module::Module;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;

pub struct ObjectCompiler;
pub struct ObjectLinker;

impl ObjectCompiler {
    pub fn compile(
        opt_level: OptimizationLevel,
        reloc_mode: RelocMode,
        code_model: CodeModel,
        module: &Module,
        name: &str,
    ) {
        Target::initialize_all(&InitializationConfig::default());
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple).unwrap();
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                opt_level,
                reloc_mode,
                code_model,
            )
            .expect("Failed to create target machine");

        let path = Path::new(name);
        let _ = target_machine.write_to_file(module, inkwell::targets::FileType::Object, path);
    }
}

impl ObjectLinker {
    pub fn link(input_file: &String, output_file: &String) -> Result<(), i32> {
        let gcc = Command::new("gcc")
            .arg(input_file)
            .arg("-o")
            .arg(output_file)
            .output()
            .expect("Unable to run `gcc` command");

        if !gcc.status.success() {
            return Err(gcc.status.code().unwrap());
        }

        Ok(())
    }

    pub fn compile(input_file: &String, output_file: &String) {
        let link_result = Self::link(input_file, output_file);

        match link_result {
            Ok(()) => {
                let module = "[Compiler]";
                let message = format!(
                    "{} Compilation successful!\n{} {}",
                    module.green(),
                    "|-> output:".green(),
                    output_file
                );

                println!("{}", message);
            }
            Err(code) => {
                let module = "[CompilerError]";
                let message = format!(
                    "{} Compilation error! Exit Status: {}. Please open issue at language repo's\n{} {}",
                    module.red(),
                    code,
                    "|-> output:".red(),
                    output_file
                );

                println!("{}", message);
            }
        }
    }
}
