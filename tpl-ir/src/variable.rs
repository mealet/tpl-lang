// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Variable<'ctx> {
    pub str_type: String,
    pub basic_type: inkwell::types::BasicTypeEnum<'ctx>,
    pub pointer: inkwell::values::PointerValue<'ctx>,
    pub assigned_function: Option<crate::function::Function<'ctx>>,
}

#[allow(unused)]
impl<'ctx> Variable<'ctx> {
    pub fn new(
        str_type: String,
        basic_type: inkwell::types::BasicTypeEnum<'ctx>,
        pointer: inkwell::values::PointerValue<'ctx>,
        assigned_function: Option<crate::function::Function<'ctx>>,
    ) -> Self {
        Self {
            str_type,
            basic_type,
            pointer,
            assigned_function,
        }
    }
}
