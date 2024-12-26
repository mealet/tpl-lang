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

const LINKERS: [&str; 3] = ["clang", "gcc", "cc"];

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
        let _ = target_machine.write_to_file(module, inkwell::targets::FileType::Object, path).unwrap();
    }
}

impl ObjectLinker {
    pub fn link(input_file: &String, output_file: &str) -> Result<(), ()> {
        let mut output_path = output_file.to_owned();

        if cfg!(windows) && !output_file.contains(".exe") {
            output_path = format!("{}.exe", output_path);
        }

        for linker in LINKERS {
            let linker_cmd = Command::new(linker)
                .arg(input_file)
                .arg("-o")
                .arg(output_path.clone())
                .output();

            if let Ok(output) = linker_cmd {
                if output.status.success() {
                    return Ok(());
                }
            }
        }

        Err(())
    }

    pub fn compile(input_file: &String, output_file: &String) {
        let link_result = Self::link(input_file, output_file);

        match link_result {
            Ok(()) => {
                let module = "[Compiler]";
                let message = format!(
                    "{} Compilation successful!\n{} {}",
                    module.green(),
                    "|-> output binary:".green(),
                    output_file
                );

                println!("{}", message);
            }
            Err(_) => {
                let module = "[CompilerError]";
                let message = format!(
                    "{} Compilation error!\n{} Maybe you forgot to install clang/gcc/cc or any other C compiler?\n{} Otherwise, please open issue at language repo's.",
                    module.red(),
                    " ".repeat(module.len()),
                    " ".repeat(module.len()),
                );

                println!("{}", message);
            }
        }
    }
}
