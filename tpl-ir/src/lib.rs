// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};

use std::collections::HashMap;

use tpl_parser::{expressions::Expressions, statements::Statements, value::Value};

#[derive(Debug)]
pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,

    variables: HashMap<String, (String, BasicTypeEnum<'ctx>, PointerValue<'ctx>)>,
    printf_fn: FunctionValue<'ctx>,
}

// #[allow(unused)]
impl<'a, 'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        // defining built-in functions
        let printf_type = context.i8_type().fn_type(
            &[context.ptr_type(inkwell::AddressSpace::default()).into()],
            true,
        );
        let printf_fn = module.add_function("printf", printf_type, None);

        Compiler {
            context: &context,
            builder,
            module,
            variables: HashMap::new(),
            printf_fn,
        }
    }

    pub fn generate(&mut self, statements: Vec<Statements>) {
        // main function creation
        let i32_type = self.context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        for statement in statements {
            self.compile_statement(statement, function);
        }

        // returning 0
        let _ = self
            .builder
            .build_return(Some(&i32_type.const_int(0, false)));
    }

    fn compile_statement(&mut self, statement: Statements, function: FunctionValue<'ctx>) {
        match statement {
            Statements::AnnotationStatement {
                identifier,
                datatype,
                value,
            } => {
                let var_type = self.get_basic_type(&datatype);
                let alloca = self.builder.build_alloca(var_type, &identifier).unwrap();
                self.variables
                    .insert(identifier.clone(), (datatype, var_type, alloca));

                if let Some(intial_value) = value {
                    let compiled_expression = self.compile_expression(*intial_value, function);
                    let _ = self.builder.build_store(alloca, compiled_expression.1);
                }
            }
            Statements::AssignStatement { identifier, value } => {
                if let Some(var_ptr) = self.variables.clone().get(&identifier) {
                    if let Some(expr) = value {
                        let expr_value = self.compile_expression(*expr, function);
                        let _ = self.builder.build_store(var_ptr.2, expr_value.1);
                    }
                }
            }
            Statements::FunctionCallStatement {
                function_name,
                arguments,
            } => {
                if function_name == "print" {
                    self.build_print_call(arguments, function);
                }
            }
            _ => {}
        }
    }

    fn compile_expression(
        &mut self,
        expr: Expressions,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        match expr {
            Expressions::Value(val) => self.compile_value(val),
            Expressions::Binary { operand, lhs, rhs } => {
                let left = self.compile_expression(*lhs, function);
                let right = self.compile_expression(*rhs, function);

                match operand.as_str() {
                    "+" => {
                        // add
                        return (
                            right.0,
                            self.builder
                                .build_int_add(
                                    left.1.into_int_value(),
                                    right.1.into_int_value(),
                                    "tmpadd",
                                )
                                .unwrap()
                                .into(),
                        );
                    }
                    "-" => {
                        // substract
                        return (
                            right.0,
                            self.builder
                                .build_int_sub(
                                    left.1.into_int_value(),
                                    right.1.into_int_value(),
                                    "tmpsub",
                                )
                                .unwrap()
                                .into(),
                        );
                    }
                    "*" => {
                        // multiply
                        return (
                            right.0,
                            self.builder
                                .build_int_mul(
                                    left.1.into_int_value(),
                                    right.1.into_int_value(),
                                    "tmpmul",
                                )
                                .unwrap()
                                .into(),
                        );
                    }
                    "/" => {
                        // divide
                        return (
                            right.0,
                            self.builder
                                .build_int_signed_div(
                                    left.1.into_int_value(),
                                    right.1.into_int_value(),
                                    "tmpdiv",
                                )
                                .unwrap()
                                .into(),
                        );
                    }
                    _ => panic!("Unsupported expression"),
                }
            }
            _ => panic!("Unsupported expression"),
        }
    }

    fn compile_value(&self, value: Value) -> (String, BasicValueEnum<'ctx>) {
        match value {
            Value::Integer(i) => (
                "int".to_string(),
                self.context.i32_type().const_int(i as u64, false).into(),
            ),
            Value::Boolean(b) => (
                "bool".to_string(),
                self.context.bool_type().const_int(b as u64, false).into(),
            ),
            Value::String(str) => {
                let str_val = self.builder.build_global_string_ptr(&str, "str");
                (
                    "str".to_string(),
                    str_val.unwrap().as_pointer_value().into(),
                )
            }
            Value::Identifier(id) => {
                if let Some(var_ptr) = self.variables.get(&id) {
                    (
                        var_ptr.0.clone(),
                        self.builder.build_load(var_ptr.1, var_ptr.2, &id).unwrap(),
                    )
                } else {
                    panic!("Undefined variable: {}", id);
                }
            }
        }
    }

    fn get_basic_type(&self, datatype: &str) -> BasicTypeEnum<'ctx> {
        match datatype {
            "int" => self.context.i32_type().into(),
            "bool" => self.context.bool_type().into(),
            "str" => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            _ => panic!("Unsupported datatype"),
        }
    }

    fn build_print_call(&mut self, arguments: Vec<Expressions>, function: FunctionValue<'ctx>) {
        for arg in arguments {
            let value = self.compile_expression(arg, function);
            let mut output_value = value.1.clone();

            let format_string = match value.1 {
                BasicValueEnum::IntValue(int) => match value.0.as_str() {
                    "int" => self.builder.build_global_string_ptr("%d\n", "fmt"),
                    "bool" => {
                        let true_str = self
                            .builder
                            .build_global_string_ptr("true", "true_str")
                            .unwrap()
                            .as_pointer_value();
                        let false_str = self
                            .builder
                            .build_global_string_ptr("false", "false_str")
                            .unwrap()
                            .as_pointer_value();

                        output_value = self
                            .builder
                            .build_select(int, true_str, false_str, "bool_str")
                            .unwrap();

                        self.builder.build_global_string_ptr("%s\n", "fmt_bool")
                    }
                    _ => panic!("Unsupported IntValue"),
                },
                BasicValueEnum::PointerValue(_) => {
                    self.builder.build_global_string_ptr("%s\n", "fmt_str")
                }
                _ => panic!("Unsupported type for print"),
            };

            let _ = self.builder.build_call(
                self.printf_fn,
                &[
                    format_string.unwrap().as_pointer_value().into(),
                    output_value.into(),
                ],
                "printf",
            );
        }
    }

    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }
}
