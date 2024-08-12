use std::path::Path;

use inkwell::module::Module;
use inkwell::targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine};
use inkwell::OptimizationLevel;

pub struct ObjectCompiler;

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
