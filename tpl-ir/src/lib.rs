// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

mod error;
mod function;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType},
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue},
};

use std::collections::HashMap;

use error::{ErrorType, GenError};
use function::Function;
use tpl_parser::{expressions::Expressions, statements::Statements, value::Value};

const TEST_OPERATORS: [&'static str; 4] = [">", "<", "==", "!="];

#[derive(Debug)]
pub struct Compiler<'ctx> {
    // important
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,

    // function and block
    main_function: FunctionValue<'ctx>,
    current_block: BasicBlock<'ctx>,

    // hashmaps
    variables: HashMap<String, (String, BasicTypeEnum<'ctx>, PointerValue<'ctx>)>,
    functions: HashMap<String, Function<'ctx>>,

    // flags
    function_returned: bool,

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

        // main function creation
        let i32_type = context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let function = module.add_function("main", fn_type, None);
        let basic_block = context.append_basic_block(function, "entry");

        Compiler {
            context: &context,
            builder,
            module,

            variables: HashMap::new(),
            functions: HashMap::new(),

            current_block: basic_block,
            main_function: function,

            function_returned: false,

            printf_fn,
        }
    }

    pub fn generate(&mut self, statements: Vec<Statements>) {
        self.builder.position_at_end(self.current_block);

        for statement in statements {
            self.compile_statement(statement, self.main_function);
        }

        // returning 0
        if !self.function_returned {
            let _ = self
                .builder
                .build_return(Some(&self.context.i32_type().const_int(0, false)));
        }
    }

    fn switch_block(&mut self, dest: BasicBlock<'ctx>) {
        self.current_block = dest;
        self.builder.position_at_end(dest);
    }

    fn compile_statement(&mut self, statement: Statements, function: FunctionValue<'ctx>) {
        match statement {
            // NOTE: Annotation
            Statements::AnnotationStatement {
                identifier,
                datatype,
                value,
                line,
            } => {
                let var_type = self.get_basic_type(&datatype, line);
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
                            line,
                        );
                        std::process::exit(1);
                    });
                self.variables
                    .insert(identifier.clone(), (datatype.clone(), var_type, alloca));

                if let Some(intial_value) = value {
                    let compiled_expression =
                        self.compile_expression(*intial_value, line, function);

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
                            line,
                        );
                        std::process::exit(1);
                    }

                    let _ = self.builder.build_store(alloca, compiled_expression.1);
                }
            }

            // NOTE: Assignment
            Statements::AssignStatement {
                identifier,
                value,
                line,
            } => {
                if let Some(var_ptr) = self.variables.clone().get(&identifier) {
                    if let Some(expr) = value {
                        let expr_value = self.compile_expression(*expr, line, function);

                        // matching datatypes

                        if expr_value.0 != var_ptr.0 {
                            GenError::throw(
                                format!(
                                    "Expected type `{}`, but found `{}`!",
                                    var_ptr.0, expr_value.0
                                ),
                                ErrorType::TypeError,
                                line,
                            );
                            std::process::exit(1);
                        }

                        // storing value

                        let _ = self.builder.build_store(var_ptr.2, expr_value.1);
                    }
                } else {
                    GenError::throw(
                        format!("Variable `{}` is not defined!", identifier),
                        ErrorType::NotDefined,
                        line,
                    );
                    std::process::exit(1);
                }
            }
            Statements::BinaryAssignStatement {
                identifier,
                operand,
                value,
                line,
            } => {
                if let Some(var_ptr) = self.variables.clone().get(&identifier) {
                    if let Some(expr) = value {
                        // building new binary expression
                        let new_expression = Expressions::Binary {
                            operand,
                            lhs: Box::new(Expressions::Value(Value::Identifier(identifier))),
                            rhs: expr.clone(),
                            line,
                        };

                        let expr_value = self.compile_expression(new_expression, line, function);

                        // matching types
                        if expr_value.0 != var_ptr.0 {
                            GenError::throw(
                                format!(
                                    "Expected type `{}`, but found `{}`!",
                                    var_ptr.0, expr_value.0
                                ),
                                ErrorType::TypeError,
                                line,
                            );
                        }

                        // storing value

                        let _ = self.builder.build_store(var_ptr.2, expr_value.1);
                    }
                }
            }

            // NOTE: Functions
            Statements::FunctionDefineStatement {
                function_name,
                function_type,
                arguments,
                block,
                line,
            } => {
                // compiling args types
                let mut args: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();
                for item in arguments.clone() {
                    let arg = self.get_basic_type(item.1.as_str(), line);
                    args.push(arg.into())
                }

                // creating function type
                let fn_type = self.get_fn_type(function_type.as_str(), &args, false, line);

                // adding function
                let function = self
                    .module
                    .add_function(function_name.as_str(), fn_type, None);

                // creating entry point into function
                let entry = self.context.append_basic_block(function, "entry");

                // storing old builder position and switching to new
                let old_position = self.current_block;
                self.builder.position_at_end(entry);

                // storing arguments values to variables
                let mut old_variables = HashMap::new();

                for (index, arg) in arguments.iter().enumerate() {
                    let varname = arg.0.clone();
                    let arg_value = function.get_nth_param(index as u32).unwrap_or_else(|| {
                        GenError::throw(
                            format!("An error occured with fetching parameter while defining `{}` function!", function_name),
                            ErrorType::BuildError,
                            line
                        );
                        std::process::exit(1);
                    });
                    let old_value = self.variables.remove(&varname);
                    old_variables.insert(varname.clone(), old_value);

                    // storing value
                    let parameter_type = self.get_basic_type(arg.1.as_str(), line);
                    let parameter_alloca = self
                        .builder
                        .build_alloca(
                            parameter_type,
                            format!("{}_param_{}", function_name, index).as_str(),
                        )
                        .unwrap_or_else(|_| {
                            GenError::throw(
                                format!(
                                    "An error occured with creating alloca for parameter `{}`!",
                                    varname.clone()
                                ),
                                ErrorType::BuildError,
                                line,
                            );
                            std::process::exit(1);
                        });

                    self.builder.build_store(parameter_alloca, arg_value);

                    // and inserting variables pointers to main hashmap
                    self.variables
                        .insert(varname, (arg.1.clone(), parameter_type, parameter_alloca));
                }

                // compiling statements
                for stmt in block {
                    self.compile_statement(stmt, function);
                }

                if !function.verify(false) {
                    GenError::throw(
                        format!("Function `{}` failed verification! Please check if you returned a value!", function_name.clone()),
                        ErrorType::VerificationFailure,
                        line
                    );
                    std::process::exit(1);
                }

                // storing function to compiler
                let mut arguments_types = Vec::new();
                for arg in arguments {
                    arguments_types.push(arg.1);
                }

                self.functions.insert(
                    function_name.clone(),
                    Function {
                        name: function_name,
                        function_type,
                        function_value: function,
                        arguments_types,
                    },
                );

                // and switching to old position
                self.builder.position_at_end(old_position);
            }

            Statements::FunctionCallStatement {
                function_name,
                arguments,
                line,
            } => {
                match function_name.as_str() {
                    "print" => {
                        self.build_print_call(arguments, line, function);
                    }
                    _ => {
                        // user defined function
                        self.fn_call(function_name, arguments, line, function);
                    }
                }
            }

            Statements::ReturnStatement { value, line } => {
                let compiled_value = self.compile_expression(value, line, function);
                self.builder.build_return(Some(&compiled_value.1));
            }

            // NOTE: Constructions
            Statements::IfStatement {
                condition,
                then_block,
                else_block,
                line,
            } => {
                // compiling condition
                let compiled_condition = self.compile_condition(condition, line, function);

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
                    self.switch_block(then_basic_block);

                    for stmt in then_block {
                        self.compile_statement(stmt, function);
                    }

                    // building branch to merge point
                    self.builder.build_unconditional_branch(merge_basic_block);

                    // filling `else` block
                    self.switch_block(else_basic_block);

                    for stmt in else_matched_block {
                        self.compile_statement(stmt, function);
                    }

                    // branch to merge block

                    self.builder.build_unconditional_branch(merge_basic_block);

                    // and changing current builder position

                    self.switch_block(merge_basic_block);
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
                    self.switch_block(then_basic_block);

                    for stmt in then_block {
                        self.compile_statement(stmt, function);
                    }

                    // building branch to merge point
                    self.builder.build_unconditional_branch(merge_basic_block);

                    // and changing current builder position
                    self.switch_block(merge_basic_block);
                }
            }

            // NOTE: Cycles
            Statements::WhileStatement {
                condition,
                block,
                line,
            } => {
                // creating basic blocks
                let before_basic_block = self.context.append_basic_block(function, "while_before");
                let then_basic_block = self.context.append_basic_block(function, "while_then");
                let after_basic_block = self.context.append_basic_block(function, "while_after");

                // setting current position to block `before`
                self.builder.build_unconditional_branch(before_basic_block);
                self.switch_block(before_basic_block);

                // compiling condition
                let compiled_condition = self.compile_condition(condition, line, function);

                // building conditional branch to blocks
                self.builder.build_conditional_branch(
                    compiled_condition,
                    then_basic_block,
                    after_basic_block,
                );

                // building `then` block
                self.switch_block(then_basic_block);

                for stmt in block {
                    self.compile_statement(stmt, function);
                }

                // returning to block `before` for comparing condition
                self.builder.build_unconditional_branch(before_basic_block);

                // setting builder position to `after` block
                self.switch_block(after_basic_block);
            }

            Statements::ForStatement {
                varname,
                iterable_object,
                block,
                line,
            } => {
                let curr_line = line;

                // creating basic blocks
                let before_basic_block = self.context.append_basic_block(function, "for_before");
                let then_basic_block = self.context.append_basic_block(function, "for_then");
                let after_basic_block = self.context.append_basic_block(function, "for_after");

                // init iterable variable
                let var_type = self.get_basic_type("int", line);
                let var_alloca = self
                    .builder
                    .build_alloca(var_type, &varname)
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            format!(
                                "Error with creating allocation with identifier `{}`",
                                &varname
                            ),
                            ErrorType::MemoryError,
                            line,
                        );
                        std::process::exit(1);
                    });

                self.builder
                    .build_store(var_alloca, self.context.i32_type().const_zero());

                // setting current position at block `before`
                self.builder.build_unconditional_branch(before_basic_block);
                self.switch_block(before_basic_block);

                let old_variable = self.variables.remove(&varname);

                self.variables
                    .insert(varname.clone(), ("int".to_string(), var_type, var_alloca));

                // creating condition
                let cond = Expressions::Binary {
                    operand: String::from("<"),
                    lhs: Box::new(Expressions::Value(Value::Identifier(varname.clone()))),
                    rhs: Box::new(iterable_object),
                    line,
                };

                // and compiling it
                let compiled_condition = self.compile_condition(cond, line, function);

                // doing conditional branch
                self.builder.build_conditional_branch(
                    compiled_condition,
                    then_basic_block,
                    after_basic_block,
                );

                // building `then` block
                self.switch_block(then_basic_block);

                for stmt in block {
                    self.compile_statement(stmt, function);
                }

                // incrementing iter variable
                let current_value = self
                    .builder
                    .build_load(var_type, var_alloca, "itertmp")
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            format!("Unable to get access `{}` in for cycle!", &varname),
                            ErrorType::BuildError,
                            line,
                        );
                        std::process::exit(1);
                    });
                let incremented_var = self
                    .builder
                    .build_int_add(
                        current_value.into_int_value(),
                        self.context.i32_type().const_int(1, false),
                        "iter_increment_tmp",
                    )
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            format!("Unable to increment `{}` in for cycle!", &varname),
                            ErrorType::BuildError,
                            line,
                        );
                        std::process::exit(1);
                    });

                // and storing incremented value
                self.builder.build_store(var_alloca, incremented_var);

                // returning to block `before` for comparing condition
                self.builder.build_unconditional_branch(before_basic_block);

                // setting builder position to `after` block
                self.switch_block(after_basic_block);

                // returning old variable
                if let Some(val) = old_variable {
                    self.variables.insert(varname, val);
                }
            }

            Statements::BreakStatement { line } => {
                GenError::throw(
                    "`break` keyword is not supported yet.",
                    ErrorType::NotSupported,
                    line,
                );
                std::process::exit(1);
            }

            // NOTE: Not supported
            _ => {
                GenError::throw(
                    "Unsupported statement found! Please open issue with your code on Github!",
                    ErrorType::NotSupported,
                    0,
                );
                std::process::exit(1);
            }
        }
    }

    fn compile_expression(
        &mut self,
        expr: Expressions,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        match expr.clone() {
            Expressions::Value(val) => self.compile_value(val, line),
            Expressions::Call {
                function_name,
                arguments,
                line,
            } => {
                // calling and taking value from user defined function
                return self.fn_call(function_name, arguments, line, function);
            }
            Expressions::Binary {
                operand,
                lhs,
                rhs,
                line,
            } => {
                let left = self.compile_expression(*lhs, line, function);
                let right = self.compile_expression(*rhs, line, function);

                // matching types
                match left.0.as_str() {
                    // int
                    "int" => {
                        // checking if all sides are the same type
                        if right.0 != "int" {
                            GenError::throw(format!("Left and Right sides must be the same types in Binary Expression!"), ErrorType::TypeError, line);
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
                            _ if TEST_OPERATORS.contains(&operand.as_str()) => {
                                return (
                                    "bool".to_string(),
                                    self.compile_condition(expr.clone(), line, function).into(),
                                )
                            }
                            _ => todo!(),
                        }
                    }
                    _ => {
                        GenError::throw(
                            format!("Binary operations is not supported for `{}` type!", left.0),
                            ErrorType::NotSupported,
                            line,
                        );
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                GenError::throw(
                    format!("`{:?}` is not supported!", expr),
                    ErrorType::NotSupported,
                    0,
                );
                std::process::exit(1);
            }
        }
    }

    fn compile_value(&self, value: Value, line: usize) -> (String, BasicValueEnum<'ctx>) {
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
                            line,
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
                                    line,
                                );
                                std::process::exit(1);
                            }),
                    )
                } else {
                    GenError::throw(
                        format!("Undefined variable with id: `{}`!", id),
                        ErrorType::NotDefined,
                        line,
                    );
                    std::process::exit(1);
                }
            }
        }
    }

    fn compile_condition(
        &mut self,
        condition: Expressions,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> IntValue<'ctx> {
        match condition {
            Expressions::Binary {
                operand,
                lhs,
                rhs,
                line,
            } => {
                let left = self.compile_expression(*lhs, line, function);
                let right = self.compile_expression(*rhs, line, function);

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
                                    line,
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
                                line,
                            );
                            std::process::exit(1);
                        });
                    }
                    _ => {
                        GenError::throw(
                            format!("Cannot compare `{}` and `{}` types!", left.0, right.0),
                            ErrorType::TypeError,
                            line,
                        );
                        std::process::exit(1);
                    }
                }
            }
            Expressions::Value(val) => {
                let compiled_value = self.compile_value(val, line);

                if compiled_value.0 != "bool" {
                    GenError::throw(
                        format!(
                            "Unsupported `{}` type found for condition!",
                            compiled_value.0
                        ),
                        ErrorType::NotSupported,
                        line,
                    );
                    std::process::exit(1);
                }

                return compiled_value.1.into_int_value();
            }
            _ => {
                GenError::throw(
                    "Unexpected expression found on condition!",
                    ErrorType::NotExpected,
                    line,
                );
                std::process::exit(1);
            }
        }
    }

    // user defined call
    fn fn_call(
        &mut self,
        function_name: String,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if let Some(func) = self.functions.clone().get(&function_name) {
            // compiling args len
            if arguments.len() != func.arguments_types.len() {
                GenError::throw(
                    format!(
                        "Function `{}` has {} arguments, but {} found!",
                        function_name,
                        func.arguments_types.len(),
                        arguments.len()
                    ),
                    ErrorType::NotExpected,
                    line,
                );
                std::process::exit(1);
            }

            // compiling args
            let compiled_args: Vec<(String, BasicValueEnum<'ctx>)> = arguments
                .iter()
                .map(|x| self.compile_expression(x.clone(), line, function))
                .collect();

            // matching arguments types
            let mut arguments_error = false;
            let mut values: Vec<BasicMetadataValueEnum> = Vec::new();

            for (index, arg) in compiled_args.iter().enumerate() {
                if arg.0 != func.arguments_types[index] {
                    arguments_error = true;
                    GenError::throw(
                        format!(
                            "Argument {} must be `{}` type, but found `{}`!",
                            index + 1,
                            func.arguments_types[index],
                            arg.0
                        ),
                        ErrorType::TypeError,
                        line,
                    );
                } else {
                    values.push(arg.1.into());
                }
            }

            if arguments_error {
                std::process::exit(1);
            }

            // calling function
            let call_result = self.builder
                .build_call(
                    func.function_value,
                    &values,
                    format!("{}_call", &func.name).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        format!("An error occured while calling `{}` function!", &func.name),
                        ErrorType::BuildError,
                        line,
                    );
                    std::process::exit(1);
                })
                .try_as_basic_value()
                .left()
                .unwrap_or_else(|| {
                    GenError::throw("Error with compiling function's returned value to basic datatype! Please open issue on github repo!", ErrorType::BuildError, line);
                    std::process::exit(1);
                });

            return (func.function_type.clone(), call_result);
        } else {
            GenError::throw(
                format!("Function `{}` is not defined here!", function_name),
                ErrorType::NotDefined,
                line,
            );
            std::process::exit(1);
        }
    }

    // getting types

    fn get_basic_type(&self, datatype: &str, line: usize) -> BasicTypeEnum<'ctx> {
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
                    line,
                );
                std::process::exit(1);
            }
        }
    }

    fn get_fn_type(
        &self,
        datatype: &str,
        params: &[BasicMetadataTypeEnum<'ctx>],
        is_var_args: bool,
        line: usize,
    ) -> FunctionType<'ctx> {
        match datatype {
            "int" => self.context.i32_type().fn_type(params, is_var_args),
            "bool" => self.context.bool_type().fn_type(params, is_var_args),
            "str" => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .fn_type(params, is_var_args),
            _ => {
                GenError::throw(
                    format!("Unsupported `{}` function type found!", datatype),
                    ErrorType::NotSupported,
                    line,
                );
                std::process::exit(1);
            }
        }
    }

    // built-in functions

    fn build_print_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) {
        for arg in arguments {
            let value = self.compile_expression(arg, line, function);
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
                            line,
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
                        line,
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
