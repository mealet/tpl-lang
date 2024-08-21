#[derive(Debug, Clone)]
#[allow(unused)]
pub struct Function<'ctx> {
    pub name: String,
    pub function_type: String,
    pub function_value: inkwell::values::FunctionValue<'ctx>,
    pub arguments_types: Vec<String>,
}
