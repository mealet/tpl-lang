// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

mod error;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValueEnum, FunctionValue, IntValue, PointerValue};

use std::collections::HashMap;

use error::{ErrorType, GenError};
use tpl_parser::{expressions::Expressions, statements::Statements, value::Value};

#[derive(Debug)]
pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,

    variables: HashMap<String, (String, BasicTypeEnum<'ctx>, PointerValue<'ctx>)>,

    // built-in functions
    printf_fn: FunctionValue<'ctx>,
}

#[allow(unused)]
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
                let alloca = self
                    .builder
                    .build_alloca(var_type, &identifier)
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            format!(
                                "Error with creating allocation with identifier `{}`",
                                &identifier
                            ),
                            ErrorType::MemoryError,
                        );
                        std::process::exit(1);
                    });
                self.variables
                    .insert(identifier.clone(), (datatype.clone(), var_type, alloca));

                if let Some(intial_value) = value {
                    let compiled_expression = self.compile_expression(*intial_value, function);

                    // matching datatypes

                    if compiled_expression.0 != datatype {
                        GenError::throw(
                            format!(
                                "Type `{}` expected for '{}' variable, but found `{}`!",
                                datatype,
                                identifier.clone(),
                                compiled_expression.0
                            ),
                            ErrorType::TypeError,
                        );
                        std::process::exit(1);
                    }

                    let _ = self.builder.build_store(alloca, compiled_expression.1);
                }
            }
            Statements::AssignStatement { identifier, value } => {
                if let Some(var_ptr) = self.variables.clone().get(&identifier) {
                    if let Some(expr) = value {
                        let expr_value = self.compile_expression(*expr, function);

                        // matching datatypes

                        if expr_value.0 != var_ptr.0 {
                            GenError::throw(
                                format!(
                                    "Expected type `{}`, but found `{}`!",
                                    var_ptr.0, expr_value.0
                                ),
                                ErrorType::TypeError,
                            );
                            std::process::exit(1);
                        }

                        // storing value

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
            Statements::IfStatement {
                condition,
                then_block,
                else_block,
            } => {
                // compiling condition
                let compiled_condition = self.compile_condition(condition, function);

                // checking for else block
                if let Some(else_matched_block) = else_block {
                    // creating blocks
                    let then_basic_block = self.context.append_basic_block(function, "if_then");
                    let else_basic_block = self.context.append_basic_block(function, "if_else");
                    let merge_basic_block = self.context.append_basic_block(function, "if_merge");

                    // building conditional branch to blocks
                    self.builder.build_conditional_branch(
                        compiled_condition,
                        then_basic_block,
                        else_basic_block,
                    );

                    // building `then` block
                    self.builder.position_at_end(then_basic_block);

                    for stmt in then_block {
                        self.compile_statement(stmt, function);
                    }

                    // building branch to merge point
                    self.builder.build_unconditional_branch(merge_basic_block);

                    // filling `else` block
                    self.builder.position_at_end(else_basic_block);

                    for stmt in else_matched_block {
                        self.compile_statement(stmt, function);
                    }

                    // branch to merge block

                    self.builder.build_unconditional_branch(merge_basic_block);

                    // and changing current builder position
                    self.builder.position_at_end(merge_basic_block);
                } else {
                    // the same but without else block
                    let then_basic_block = self.context.append_basic_block(function, "if_then");
                    let merge_basic_block = self.context.append_basic_block(function, "if_merge");

                    // building conditional branch to blocks
                    self.builder.build_conditional_branch(
                        compiled_condition,
                        then_basic_block,
                        merge_basic_block,
                    );

                    // building `then` block
                    self.builder.position_at_end(then_basic_block);

                    for stmt in then_block {
                        self.compile_statement(stmt, function);
                    }

                    // building branch to merge point
                    self.builder.build_unconditional_branch(merge_basic_block);

                    // and changing current builder position
                    self.builder.position_at_end(merge_basic_block);
                }
            }
            Statements::WhileStatement { condition, block } => {
                // creating basic blocks
                let before_basic_block = self.context.append_basic_block(function, "while_before");
                let then_basic_block = self.context.append_basic_block(function, "while_then");
                let after_basic_block = self.context.append_basic_block(function, "while_after");

                // setting current position to block `before`
                self.builder.build_unconditional_branch(before_basic_block);
                self.builder.position_at_end(before_basic_block);

                // compiling condition
                let compiled_condition = self.compile_condition(condition, function);

                // building conditional branch to blocks
                self.builder.build_conditional_branch(
                    compiled_condition,
                    then_basic_block,
                    after_basic_block,
                );

                // building `then` block
                self.builder.position_at_end(then_basic_block);

                for stmt in block {
                    self.compile_statement(stmt, function);
                }

                // returning to block `before` for comparing condition
                self.builder.build_unconditional_branch(before_basic_block);

                // setting builder position to `after` block
                self.builder.position_at_end(after_basic_block);
            }
            _ => {
                GenError::throw(
                    String::from(
                        "Unsupported statement found! Please open issue with your code on Github!",
                    ),
                    ErrorType::NotSupported,
                );
                std::process::exit(1);
            }
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

                // matching types
                match left.0.as_str() {
                    // int
                    "int" => {
                        // checking if all sides are the same type
                        if right.0 != "int" {
                            GenError::throw(format!("Left and Right sides must be the same types in Binary Expression!"), ErrorType::TypeError);
                            std::process::exit(1);
                        }

                        match operand.as_str() {
                            // NOTE: Basic Binary Operations
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
                            _ => todo!(),
                        }
                    }
                    _ => {
                        GenError::throw(
                            format!("Binary operations is not supported for `{}` type!", left.0),
                            ErrorType::NotSupported,
                        );
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                GenError::throw(
                    format!("`{:?}` is not supported!", expr),
                    ErrorType::NotSupported,
                );
                std::process::exit(1);
            }
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
                let str_val = self
                    .builder
                    .build_global_string_ptr(&str, "str")
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            format!("Error while creating string: `{}`", str),
                            ErrorType::MemoryError,
                        );
                        std::process::exit(1);
                    });
                ("str".to_string(), str_val.as_pointer_value().into())
            }
            Value::Identifier(id) => {
                if let Some(var_ptr) = self.variables.get(&id) {
                    (
                        var_ptr.0.clone(),
                        self.builder
                            .build_load(var_ptr.1, var_ptr.2, &id)
                            .unwrap_or_else(|_| {
                                GenError::throw(
                                    format!("Error with loading `{}` variable", id),
                                    ErrorType::MemoryError,
                                );
                                std::process::exit(1);
                            }),
                    )
                } else {
                    GenError::throw(
                        format!("Undefined variable with id: `{}`!", id),
                        ErrorType::NotSupported,
                    );
                    std::process::exit(1);
                }
            }
        }
    }

    fn compile_condition(
        &mut self,
        condition: Expressions,
        function: FunctionValue<'ctx>,
    ) -> IntValue<'ctx> {
        if let Expressions::Binary { operand, lhs, rhs } = condition {
            let left = self.compile_expression(*lhs, function);
            let right = self.compile_expression(*rhs, function);

            // matching same supported types
            match (left.0.as_str(), right.0.as_str()) {
                ("int", "int") => {
                    // matching operand
                    let predicate = match operand.as_str() {
                        ">" => inkwell::IntPredicate::SGT,
                        "<" => inkwell::IntPredicate::SLT,
                        "==" => inkwell::IntPredicate::EQ,
                        "!=" => inkwell::IntPredicate::NE,
                        _ => {
                            GenError::throw(
                                format!("Compare operand `{}` is not supported!", operand),
                                ErrorType::NotSupported,
                            );
                            std::process::exit(1);
                        }
                    };

                    // creating condition
                    let condition = self.builder.build_int_compare(
                        predicate,
                        left.1.into_int_value(),
                        right.1.into_int_value(),
                        "int_condition",
                    );

                    return condition.unwrap_or_else(|_| {
                        GenError::throw(
                            format!(
                                "An error occured while building condition `{} {} {}`!",
                                left.0, operand, right.0
                            ),
                            ErrorType::BuildError,
                        );
                        std::process::exit(1);
                    });
                }
                _ => {
                    GenError::throw(
                        format!("Cannot compare `{}` and `{}` types!", left.0, right.0),
                        ErrorType::TypeError,
                    );
                    std::process::exit(1);
                }
            }
        } else {
            GenError::throw(
                String::from("Conditions only supported in `Binary Operations`"),
                ErrorType::NotSupported,
            );
            std::process::exit(1);
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
            _ => {
                GenError::throw(
                    format!("Unsupported `{}` datatype!", datatype),
                    ErrorType::NotSupported,
                );
                std::process::exit(1);
            }
        }
    }

    // built-in functions

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
                    _ => {
                        GenError::throw(
                            format!("Unsupported IntValue `{}`!", value.0),
                            ErrorType::NotSupported,
                        );
                        std::process::exit(1);
                    }
                },
                BasicValueEnum::PointerValue(_) => {
                    self.builder.build_global_string_ptr("%s\n", "fmt_str")
                }
                _ => {
                    GenError::throw(
                        format!("Type `{}` is not supported for print function!", value.0),
                        ErrorType::NotSupported,
                    );
                    std::process::exit(1);
                }
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
