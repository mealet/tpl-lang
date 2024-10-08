// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Function<'ctx> {
    pub name: String,
    pub function_type: String,
    pub function_value: inkwell::values::FunctionValue<'ctx>,
    pub arguments_types: Vec<String>,
}
