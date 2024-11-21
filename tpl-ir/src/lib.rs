// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

mod error;
mod function;
mod import;
mod libc;
mod variable;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType},
    values::{BasicMetadataValueEnum, BasicValueEnum, FunctionValue, IntValue, PointerValue},
    AddressSpace,
};

use libc::Libc;
use std::collections::HashMap;

use error::{ErrorType, GenError};
use function::Function;
use import::ImportObject;
use variable::Variable;

use tpl_parser::{expressions::Expressions, statements::Statements, value::Value};

const TEST_OPERATORS: [&str; 4] = [">", "<", "==", "!="];
const LAMBDA_NAME: &str = "i_need_newer_inkwell_version"; // :D

impl<'ctx> Libc for Compiler<'ctx> {
    type Function = FunctionValue<'ctx>;

    fn __c_printf(&mut self) -> FunctionValue<'ctx> {
        if let Some(function_value) = self.built_functions.get("printf") {
            return *function_value;
        }

        let printf_type = self.context.void_type().fn_type(
            &[self.context.ptr_type(AddressSpace::default()).into()],
            true,
        );
        let printf_fn = self
            .module
            .add_function("printf", printf_type, Some(Linkage::External));
        let _ = self.built_functions.insert("printf".to_string(), printf_fn);

        printf_fn
    }

    fn __c_strcat(&mut self) -> FunctionValue<'ctx> {
        if let Some(function_value) = self.built_functions.get("strcat") {
            return *function_value;
        }

        let strcat_type = self.context.ptr_type(AddressSpace::default()).fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            true,
        );
        let strcat_fn = self
            .module
            .add_function("strcat", strcat_type, Some(Linkage::External));
        let _ = self.built_functions.insert("strcat".to_string(), strcat_fn);

        strcat_fn
    }
}

#[derive(Debug)]
pub struct Compiler<'ctx> {
    // module info
    module_name: String,
    module_source: String,

    // important
    context: &'ctx Context,
    builder: Builder<'ctx>,
    module: Module<'ctx>,

    // function and block
    main_function: FunctionValue<'ctx>,
    current_block: BasicBlock<'ctx>,

    // hashmaps
    variables: HashMap<String, Variable<'ctx>>,
    functions: HashMap<String, Function<'ctx>>,
    imports: HashMap<String, ImportObject>,

    // tech
    built_functions: HashMap<String, FunctionValue<'ctx>>,
    current_expectation_value: Option<String>,
    current_assign_function: Option<Function<'ctx>>,
    boolean_strings_ptr: Option<(PointerValue<'ctx>, PointerValue<'ctx>)>,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(
        context: &'ctx Context,
        module_name: &str,
        module_filename: String,
        module_source: String,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();

        // main function creation
        let main_fn_type = context.i8_type();
        let fn_type = main_fn_type.fn_type(&[], false);
        let function = module.add_function("main", fn_type, None);
        let basic_block = context.append_basic_block(function, "entry");

        // collection of build-functions
        let built_functions = HashMap::new();

        Compiler {
            module_name: module_filename,
            module_source,

            context,
            builder,
            module,

            variables: HashMap::new(),
            functions: HashMap::new(),
            imports: HashMap::new(),

            current_block: basic_block,
            main_function: function,

            built_functions,
            current_expectation_value: None,
            current_assign_function: None,
            boolean_strings_ptr: None,
        }
    }

    pub fn generate(&mut self, statements: Vec<Statements>) {
        self.builder.position_at_end(self.current_block);

        for statement in statements {
            self.compile_statement(statement, self.main_function);
        }

        // returning 0
        let _ = self
            .builder
            .build_return(Some(&self.context.i8_type().const_int(0, false)));

        if !self.main_function.verify(true) {
            self.module.print_to_stderr();

            GenError::throw(
                "Verification failure for `main` function! Please remove all returns blocks outside definitions!",
                ErrorType::VerificationFailure,
                self.module_name.clone(),
                self.module_source.clone(),
                0
            );
            std::process::exit(1);
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
                if datatype == *"auto" {
                    let initial_value = value.unwrap_or_else(|| {
                        GenError::throw(
                            "Variable with `auto` type cannot be empty!",
                            ErrorType::TypeError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    });

                    let compiled_expression = self.compile_expression(
                        *initial_value,
                        line,
                        function,
                        self.current_expectation_value.clone(),
                    );
                    let var_type = self.get_basic_type(compiled_expression.0.as_str(), line);
                    let alloca = self
                        .builder
                        .build_alloca(var_type, &identifier)
                        .unwrap_or_else(|_| {
                            GenError::throw(
                                "Unable to create 'automated' type alloca!",
                                ErrorType::BuildError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        });

                    self.variables.insert(
                        identifier.clone(),
                        Variable::new(compiled_expression.0, var_type, alloca, None),
                    );

                    let _ = self.builder.build_store(alloca, compiled_expression.1);
                } else {
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
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        });

                    let assigned_function = self.current_assign_function.clone();

                    self.variables.insert(
                        identifier.clone(),
                        Variable::new(datatype.clone(), var_type, alloca, assigned_function),
                    );

                    if let Some(intial_value) = value {
                        let compiled_expression = self.compile_expression(
                            *intial_value,
                            line,
                            function,
                            Some(datatype.clone()),
                        );

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
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        }

                        let _ = self.builder.build_store(alloca, compiled_expression.1);

                        // rewriting variable for assigning function

                        if datatype.starts_with("fn") {
                            self.variables.insert(
                                identifier.clone(),
                                Variable::new(
                                    datatype.clone(),
                                    var_type,
                                    alloca,
                                    self.current_assign_function.clone(),
                                ),
                            );
                        }
                    }
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
                        let expr_value = self.compile_expression(
                            *expr,
                            line,
                            function,
                            Some(var_ptr.str_type.clone()),
                        );

                        // matching datatypes

                        if expr_value.0 != var_ptr.str_type {
                            GenError::throw(
                                format!(
                                    "Expected type `{}`, but found `{}`!",
                                    var_ptr.str_type, expr_value.0
                                ),
                                ErrorType::TypeError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        }

                        // storing value

                        let _ = self.builder.build_store(var_ptr.pointer, expr_value.1);
                    }
                } else {
                    GenError::throw(
                        format!("Variable `{}` is not defined!", identifier),
                        ErrorType::NotDefined,
                        self.module_name.clone(),
                        self.module_source.clone(),
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

                        let expr_value = self.compile_expression(
                            new_expression,
                            line,
                            function,
                            self.current_expectation_value.clone(),
                        );

                        // matching types
                        if expr_value.0 != var_ptr.str_type {
                            GenError::throw(
                                format!(
                                    "Expected type `{}`, but found `{}`!",
                                    var_ptr.str_type, expr_value.0
                                ),
                                ErrorType::TypeError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                        }

                        // storing value

                        let _ = self.builder.build_store(var_ptr.pointer, expr_value.1);
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
                self.define_user_function(function_name, function_type, arguments, block, line);
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
                    "concat" => {
                        self.build_concat_call(arguments, line, function);
                    }
                    _ => {
                        // user defined function
                        self.fn_call(function_name, arguments, line, function);
                    }
                }
            }

            Statements::ReturnStatement { value, line } => {
                let compiled_value = self.compile_expression(
                    value,
                    line,
                    function,
                    self.current_expectation_value.clone(),
                );
                let _ = self.builder.build_return(Some(&compiled_value.1));
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
                    let _ = self.builder.build_conditional_branch(
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
                    if let Some(last_instruction) = then_basic_block.get_last_instruction() {
                        if last_instruction.get_opcode()
                            != inkwell::values::InstructionOpcode::Return
                        {
                            let _ = self.builder.build_unconditional_branch(merge_basic_block);
                        }
                    }

                    // filling `else` block
                    self.switch_block(else_basic_block);

                    for stmt in else_matched_block {
                        self.compile_statement(stmt, function);
                    }

                    // branch to merge block

                    if let Some(last_instruction) = else_basic_block.get_last_instruction() {
                        if last_instruction.get_opcode()
                            != inkwell::values::InstructionOpcode::Return
                        {
                            let _ = self.builder.build_unconditional_branch(merge_basic_block);
                        }
                    }

                    // and changing current builder position

                    self.switch_block(merge_basic_block);
                } else {
                    // the same but without else block
                    let then_basic_block = self.context.append_basic_block(function, "if_then");
                    let merge_basic_block = self.context.append_basic_block(function, "if_merge");

                    // building conditional branch to blocks
                    let _ = self.builder.build_conditional_branch(
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
                    if let Some(last_instruction) = then_basic_block.get_last_instruction() {
                        if last_instruction.get_opcode()
                            != inkwell::values::InstructionOpcode::Return
                        {
                            let _ = self.builder.build_unconditional_branch(merge_basic_block);
                        }
                    }

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

                if let Some(last_instruction) = self.current_block.get_last_instruction() {
                    if last_instruction.get_opcode() != inkwell::values::InstructionOpcode::Return {
                        let _ = self.builder.build_unconditional_branch(before_basic_block);
                    }
                }

                let _ = self.builder.build_unconditional_branch(before_basic_block);
                self.switch_block(before_basic_block);

                // compiling condition
                let compiled_condition = self.compile_condition(condition, line, function);

                // building conditional branch to blocks
                let _ = self.builder.build_conditional_branch(
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
                if let Some(last_instruction) = then_basic_block.get_last_instruction() {
                    if last_instruction.get_opcode() != inkwell::values::InstructionOpcode::Return {
                        let _ = self.builder.build_unconditional_branch(before_basic_block);
                    }
                }

                // setting builder position to `after` block
                self.switch_block(after_basic_block);
            }

            Statements::ForStatement {
                varname,
                iterable_object,
                block,
                line,
            } => {
                // creating basic blocks
                let before_basic_block = self.context.append_basic_block(function, "for_before");
                let then_basic_block = self.context.append_basic_block(function, "for_then");
                let after_basic_block = self.context.append_basic_block(function, "for_after");

                // init iterable variable

                // getting iterable object type
                let iterator_vartype = self
                    .compile_expression(
                        iterable_object.clone(),
                        line,
                        function,
                        self.current_expectation_value.clone(),
                    )
                    .0;

                let var_type = self.get_basic_type(iterator_vartype.as_str(), line);
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
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    });

                let _ = self.builder.build_store(var_alloca, var_type.const_zero());

                // setting current position at block `before`
                if let Some(last_instruction) = self.current_block.get_last_instruction() {
                    if last_instruction.get_opcode() != inkwell::values::InstructionOpcode::Return {
                        let _ = self.builder.build_unconditional_branch(before_basic_block);
                    }
                }
                self.switch_block(before_basic_block);

                let old_variable = self.variables.remove(&varname);

                self.variables.insert(
                    varname.clone(),
                    Variable::new(iterator_vartype.to_string(), var_type, var_alloca, None),
                );

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
                let _ = self.builder.build_conditional_branch(
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
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    });
                let incremented_var = self
                    .builder
                    .build_int_add(
                        current_value.into_int_value(),
                        var_type.into_int_type().const_int(1, false),
                        "iter_increment_tmp",
                    )
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            format!("Unable to increment `{}` in for cycle!", &varname),
                            ErrorType::BuildError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    });

                // and storing incremented value
                let _ = self.builder.build_store(var_alloca, incremented_var);

                // returning to block `before` for comparing condition
                if let Some(last_instruction) = self.current_block.get_last_instruction() {
                    if last_instruction.get_opcode() != inkwell::values::InstructionOpcode::Return {
                        let _ = self.builder.build_unconditional_branch(before_basic_block);
                    }
                }

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
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }

            // NOTE: Import
            Statements::ImportStatement { path, line } => {
                if let Expressions::Value(Value::String(stringified_path)) = path {
                    // getting import object
                    let obj = ImportObject::from(stringified_path);

                    // testing if import already exists
                    if self.imports.contains_key(&obj.name) {
                        GenError::throw(
                            format!("Imported module `{}` already exists!", obj.name),
                            ErrorType::ImportError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    }

                    // initializating lightweight compiler
                    // lexer
                    let mut lw_lexer = tpl_lexer::Lexer::new(obj.source.clone(), obj.name.clone());
                    let tokens = match lw_lexer.tokenize() {
                        Ok(tokens) => tokens,
                        Err(e) => {
                            let info = e.informate();
                            eprintln!("{}", info);
                            std::process::exit(1);
                        }
                    };

                    // parser
                    let mut parser =
                        tpl_parser::Parser::new(tokens, obj.name.clone(), obj.source.clone());
                    let ast = parser.parse();

                    // compiling statements

                    match ast {
                        Ok(stmts) => {
                            for stmt in stmts {
                                self.compile_statement(stmt, function);
                            }

                            // adding function to imported
                            self.imports.insert(obj.name.clone(), obj);
                        }
                        Err(err) => {
                            // printing all errors in terminal and quitting
                            eprintln!("{}", err.informate());
                            std::process::exit(1);
                        }
                    }
                } else {
                    GenError::throw(
                        "Unexpected import found!",
                        ErrorType::NotExpected,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                }
            }

            // NOTE: Not supported
            _ => {
                GenError::throw(
                    "Unsupported statement found! Please open issue with your code on Github!",
                    ErrorType::NotSupported,
                    self.module_name.clone(),
                    self.module_source.clone(),
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
        expected_datatype: Option<String>,
    ) -> (String, BasicValueEnum<'ctx>) {
        match expr.clone() {
            Expressions::Value(val) => self.compile_value(val, line, expected_datatype),
            Expressions::Call {
                function_name,
                arguments,
                line,
            } => {
                // calling and taking value from user defined function
                self.fn_call(function_name, arguments, line, function)
            }
            Expressions::Lambda {
                arguments,
                statements,
                ftype,
                line,
            } => {
                let func = self.define_user_function(
                    LAMBDA_NAME.to_string(),
                    ftype.clone(),
                    arguments,
                    statements,
                    line,
                );

                self.current_assign_function = Some(func);

                (
                    format!("fn<{}>", ftype),
                    self.context.i8_type().const_zero().into(),
                )
            }
            Expressions::Binary {
                operand,
                lhs,
                rhs,
                line,
            } => {
                let left = self.compile_expression(*lhs, line, function, expected_datatype.clone());
                let right = self.compile_expression(*rhs, line, function, expected_datatype);

                // matching types
                match left.0.as_str() {
                    // int
                    "int8" | "int16" | "int32" | "int64" | "int128" => {
                        // checking if all sides are the same type
                        if !["int8", "int16", "int32", "int64", "int128"]
                            .contains(&right.0.as_str())
                        {
                            GenError::throw(
                                "Left and Right sides must be the same types in Binary Expression!"
                                    .to_string(),
                                ErrorType::TypeError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        }

                        match operand.as_str() {
                            // NOTE: Basic Binary Operations
                            "+" => {
                                // add
                                (
                                    right.0,
                                    self.builder
                                        .build_int_add(
                                            left.1.into_int_value(),
                                            right.1.into_int_value(),
                                            "tmpadd",
                                        )
                                        .unwrap()
                                        .into(),
                                )
                            }
                            "-" => {
                                // substract
                                (
                                    right.0,
                                    self.builder
                                        .build_int_sub(
                                            left.1.into_int_value(),
                                            right.1.into_int_value(),
                                            "tmpsub",
                                        )
                                        .unwrap()
                                        .into(),
                                )
                            }
                            "*" => {
                                // multiply
                                (
                                    right.0,
                                    self.builder
                                        .build_int_mul(
                                            left.1.into_int_value(),
                                            right.1.into_int_value(),
                                            "tmpmul",
                                        )
                                        .unwrap()
                                        .into(),
                                )
                            }
                            "/" => {
                                // divide
                                (
                                    right.0,
                                    self.builder
                                        .build_int_signed_div(
                                            left.1.into_int_value(),
                                            right.1.into_int_value(),
                                            "tmpdiv",
                                        )
                                        .unwrap()
                                        .into(),
                                )
                            }
                            _ if TEST_OPERATORS.contains(&operand.as_str()) => (
                                "bool".to_string(),
                                self.compile_condition(expr.clone(), line, function).into(),
                            ),
                            _ => {
                                GenError::throw(
                                    format!("Unsupported binary operation found: `{}`", operand),
                                    ErrorType::NotSupported,
                                    self.module_name.clone(),
                                    self.module_source.clone(),
                                    line,
                                );
                                std::process::exit(1);
                            }
                        }
                    }
                    _ => {
                        GenError::throw(
                            format!("Binary operations is not supported for `{}` type!", left.0),
                            ErrorType::NotSupported,
                            self.module_name.clone(),
                            self.module_source.clone(),
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
                    self.module_name.clone(),
                    self.module_source.clone(),
                    0,
                );
                std::process::exit(1);
            }
        }
    }

    fn compile_value(
        &self,
        value: Value,
        line: usize,
        expected: Option<String>,
    ) -> (String, BasicValueEnum<'ctx>) {
        match value {
            Value::Integer(i) => {
                if let Some(exp) = expected {
                    if exp != "void" {
                        let basic_type = self.get_basic_type(exp.as_str(), line).into_int_type();

                        return (exp.to_string(), basic_type.const_int(i as u64, true).into());
                    }
                }

                match i {
                    -255..=255 => (
                        "int8".to_string(),
                        self.context.i8_type().const_int(i as u64, true).into(),
                    ),
                    -65_535..65_535 => (
                        "int16".to_string(),
                        self.context.i16_type().const_int(i as u64, true).into(),
                    ),
                    -2_147_483_648..2_147_483_648 => (
                        "int32".to_string(),
                        self.context.i32_type().const_int(i as u64, true).into(),
                    ),
                    -9_223_372_036_854_775_808..9_223_372_036_854_775_808 => (
                        "int64".to_string(),
                        self.context.i64_type().const_int(i as u64, true).into(),
                    ),
                    -170_141_183_460_469_231_731_687_303_715_884_105_728
                        ..=170_141_183_460_469_231_731_687_303_715_884_105_727 => (
                        "int128".to_string(),
                        self.context.i128_type().const_int(i as u64, true).into(),
                    ), // Even the compiler says that number bigger 128-bits is unreachable. xD

                       // _ => {
                       //     GenError::throw(
                       //         "Provided integer is too big! Max supported type is 128-bit number!",
                       //         ErrorType::TypeError,
                       //         self.module_name.clone(),
                       //         self.module_source.clone(),
                       //         line
                       //     );
                       //     std::process::exit(1);
                       // }
                }
            }
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
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    });

                str_val.set_constant(false);
                ("str".to_string(), str_val.as_pointer_value().into())
            }
            Value::Identifier(id) => {
                if let Some(var_ptr) = self.variables.get(&id) {
                    (
                        var_ptr.str_type.clone(),
                        self.builder
                            .build_load(var_ptr.basic_type, var_ptr.pointer, &id)
                            .unwrap_or_else(|_| {
                                GenError::throw(
                                    format!("Error with loading `{}` variable", id),
                                    ErrorType::MemoryError,
                                    self.module_name.clone(),
                                    self.module_source.clone(),
                                    line,
                                );
                                std::process::exit(1);
                            }),
                    )
                } else {
                    GenError::throw(
                        format!("Undefined variable with id: `{}`!", id),
                        ErrorType::NotDefined,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                }
            }
            _ => todo!(),
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
                let left = self.compile_expression(
                    *lhs,
                    line,
                    function,
                    self.current_expectation_value.clone(),
                );

                // fix different size type comparison
                let _old_exp_value = self.current_expectation_value.clone();
                self.current_expectation_value = Some(left.0.clone());

                let right = self.compile_expression(
                    *rhs,
                    line,
                    function,
                    self.current_expectation_value.clone(),
                );

                // matching same supported types
                match (left.0.as_str(), right.0.as_str()) {
                    ("int8", "int8")
                    | ("int16", "int16")
                    | ("int32", "int32")
                    | ("int64", "int64")
                    | ("int128", "int128") => {
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
                                    self.module_name.clone(),
                                    self.module_source.clone(),
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

                        condition.unwrap_or_else(|_| {
                            GenError::throw(
                                format!(
                                    "An error occured while building condition `{} {} {}`!",
                                    left.0, operand, right.0
                                ),
                                ErrorType::BuildError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        })
                    }
                    _ => {
                        GenError::throw(
                            format!("Cannot compare `{}` and `{}` types!", left.0, right.0),
                            ErrorType::TypeError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    }
                }
            }
            Expressions::Value(val) => {
                let compiled_value = self.compile_value(val, line, None);

                if compiled_value.0 != "bool" {
                    GenError::throw(
                        format!(
                            "Unsupported `{}` type found for condition!",
                            compiled_value.0
                        ),
                        ErrorType::NotSupported,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                }

                compiled_value.1.into_int_value()
            }
            _ => {
                GenError::throw(
                    "Unexpected expression found on condition!",
                    ErrorType::NotExpected,
                    self.module_name.clone(),
                    self.module_source.clone(),
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
        let mut is_var_stored = false;

        if !self.functions.contains_key(&function_name) {
            match function_name.as_str() {
                "concat" => return self.build_concat_call(arguments, line, function),
                "print" => {
                    GenError::throw(
                        "Function `print` is 'void' type!",
                        ErrorType::TypeError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                }
                _ => {
                    if let Some(var) = self.variables.get(&function_name) {
                        if var.assigned_function.is_some() {
                            is_var_stored = true;
                        } else {
                            GenError::throw(
                                format!("Variable `{}` is not a function!", function_name),
                                ErrorType::TypeError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        }
                    } else {
                        GenError::throw(
                            format!("Function `{}` is not defined!", function_name),
                            ErrorType::NotDefined,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    }
                }
            };
        }

        let func = if is_var_stored {
            self.variables
                .get(&function_name)
                .unwrap()
                .clone()
                .assigned_function
                .unwrap()
        } else {
            self.functions.get(&function_name).unwrap().clone()
        };

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
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        // matching arguments types
        let mut arguments_error = false;
        let mut arguments_types = Vec::new();

        let mut values: Vec<BasicMetadataValueEnum> = Vec::new();

        for (index, arg) in arguments.iter().enumerate() {
            let compiled_arg = self.compile_expression(
                arg.clone(),
                line,
                function,
                Some(func.arguments_types[index].clone()),
            );

            if compiled_arg.0 != func.arguments_types[index] {
                arguments_error = true;
            } else {
                values.push(compiled_arg.1.into());
            }

            arguments_types.push(compiled_arg.0.clone());
        }

        if arguments_error {
            if func.name == LAMBDA_NAME {
                GenError::throw(
                    format!(
                        "Lambda function expected arguments types [{}], but found [{}]!",
                        func.arguments_types.clone().join(", "),
                        arguments_types.join(", "),
                    ),
                    ErrorType::TypeError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }
            GenError::throw(
                format!(
                    "Function `{}` expected arguments types [{}], but found [{}]!",
                    func.name,
                    func.arguments_types.clone().join(", "),
                    arguments_types.join(", "),
                ),
                ErrorType::TypeError,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
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
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            })
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| {
                if func.function_type == "void" {
                    self.context.i8_type().const_zero().into()
                } else {
                    GenError::throw("Error with compiling function's returned value to basic datatype! Please open issue on github repo!", ErrorType::BuildError, self.module_name.clone(), self.module_source.clone(), line);
                    std::process::exit(1);
                }
            });

        (func.function_type.clone(), call_result)
    }

    // getting types

    fn get_basic_type(&self, datatype: &str, line: usize) -> BasicTypeEnum<'ctx> {
        match datatype {
            _ if datatype.starts_with("fn<") => {
                let fn_type = datatype.replace("fn<", "").replace(">", "");
                self.get_basic_type(fn_type.as_str(), line)
            }
            "int8" => self.context.i8_type().into(),
            "int16" => self.context.i16_type().into(),
            "int32" => self.context.i32_type().into(),
            "int64" => self.context.i64_type().into(),
            "int128" => self.context.i128_type().into(),
            "bool" => self.context.bool_type().into(),
            "str" => self.context.ptr_type(AddressSpace::default()).into(),
            "auto" => self.context.i8_type().into(),
            "void" => self.context.ptr_type(AddressSpace::default()).into(),
            // Yep, this seems like a very bad idea, but
            // `void` type requires AnyTypeEnum, which is not allowed for the whole builder's
            // functions
            _ => {
                GenError::throw(
                    format!("Unsupported `{}` datatype!", datatype),
                    ErrorType::NotSupported,
                    self.module_name.clone(),
                    self.module_source.clone(),
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
            "int8" => self.context.i8_type().fn_type(params, is_var_args),
            "int16" => self.context.i16_type().fn_type(params, is_var_args),
            "int32" => self.context.i32_type().fn_type(params, is_var_args),
            "int64" => self.context.i64_type().fn_type(params, is_var_args),
            "int128" => self.context.i128_type().fn_type(params, is_var_args),
            "bool" => self.context.bool_type().fn_type(params, is_var_args),
            "void" => self.context.void_type().fn_type(params, is_var_args),
            "str" => self
                .context
                .ptr_type(AddressSpace::default())
                .fn_type(params, is_var_args),
            _ => {
                GenError::throw(
                    format!("Unsupported `{}` function type found!", datatype),
                    ErrorType::NotSupported,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }
        }
    }

    // built-in functions

    fn build_concat_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 2 {
            GenError::throw(
                "`concat` function takes 2 arguments!",
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
        }

        let left_arg = self.compile_expression(
            arguments[0].clone(),
            line,
            function,
            self.current_expectation_value.clone(),
        );
        let right_arg = self.compile_expression(
            arguments[1].clone(),
            line,
            function,
            self.current_expectation_value.clone(),
        );

        let strcat_fn = self.__c_strcat();

        if !self.validate_types(&[left_arg.0, right_arg.0], "str".to_string()) {
            GenError::throw(
                "`concat` function takes only string types!",
                ErrorType::TypeError,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
        }

        let val: BasicValueEnum<'ctx> = self
            .builder
            .build_direct_call(
                strcat_fn,
                &[left_arg.1.into(), right_arg.1.into()],
                "concat",
            )
            .unwrap_or_else(|_| {
                GenError::throw(
                    "An error occured while calling `concat` function!",
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            })
            .try_as_basic_value()
            .left()
            .unwrap_or_else(|| {
                GenError::throw(
                    "Unable to get basic value from `concat` function!",
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            });

        (String::from("str"), val)

        // `strcat` provides concatenating left and right values to left variable, which
        // means that call `concat(a, b)` will insert result into variable 'a'.
        // To fix it we need to copy both of values and use it in function (and try to free memory)
    }

    fn build_print_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) {
        let mut fmts: Vec<&str> = Vec::new();
        let mut values: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
        let printf_fn = self.__c_printf();

        for arg in arguments {
            let compiled_arg = self.compile_expression(
                arg,
                line,
                function,
                self.current_expectation_value.clone(),
            );
            let mut basic_value = compiled_arg.1;

            if compiled_arg.0 == *"void" {
                continue;
            }

            let format_string = match compiled_arg.0.as_str() {
                "int8" => "%d",
                "int16" => "%hd",
                "int32" => "%d",
                "int64" => "%lld",
                "int128" => "%lld", // now int128 isn't supported for print
                "bool" => {
                    let (_true, _false) = self.__boolean_strings();

                    if let BasicValueEnum::IntValue(int) = basic_value {
                        basic_value = self
                            .builder
                            .build_select(int, _true, _false, "bool_fmt_str")
                            .unwrap();
                    }

                    "%s"
                }
                "str" => "%s",
                _ => {
                    GenError::throw(
                        format!(
                            "Type `{}` is not supported for 'print' function!",
                            compiled_arg.0
                        ),
                        ErrorType::NotSupported,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                }
            };

            fmts.push(format_string);
            values.push(basic_value.into());
        }

        let complete_fmt_string = self
            .builder
            .build_global_string_ptr(format!("{}\n", fmts.join(" ")).as_str(), "printf_fmt")
            .unwrap_or_else(|_| {
                GenError::throw(
                    "Unable to create format string for C function!",
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            })
            .as_pointer_value();

        let mut printf_arguments = vec![complete_fmt_string.into()];
        printf_arguments.append(&mut values);

        let _ = self
            .builder
            .build_call(printf_fn, &printf_arguments, "printf_call");
    }

    pub fn define_user_function(
        &mut self,
        function_name: String,
        function_type: String,
        arguments: Vec<(String, String)>,
        block: Vec<Statements>,
        line: usize,
    ) -> Function<'ctx> {
        // setting function expected return value
        let old_expectation_value = self.current_expectation_value.clone();
        self.current_expectation_value = Some(function_type.clone());

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
                    format!(
                        "An error occured with fetching parameter while defining `{}` function!",
                        function_name
                    ),
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
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
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            let _ = self.builder.build_store(parameter_alloca, arg_value);

            // and inserting variables pointers to main hashmap
            self.variables.insert(
                varname,
                Variable::new(arg.1.clone(), parameter_type, parameter_alloca, None),
            );
        }

        // Storing function moved before statements to correct recursion work
        let mut arguments_types = Vec::new();
        for arg in arguments {
            arguments_types.push(arg.1);
        }

        let function_object = Function {
            name: function_name.clone(),
            function_type: function_type.clone(),
            function_value: function,
            arguments_types,
        };

        self.functions
            .insert(function_name.clone(), function_object.clone());

        // compiling statements
        for stmt in block {
            self.compile_statement(stmt, function);
        }

        if !function.verify(false) {
            if function_type == "void" {
                self.builder.build_return(None)
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            format!("An error occured with wrapping void '{}' function!\nPlease open an issue on project's repo!", function_name),
                            ErrorType::BuildError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line
                        );
                        std::process::exit(1);
                    });
            } else {
                if function_name == LAMBDA_NAME {
                    function.print_to_stderr();

                    GenError::throw(
                        "Lambda failed verification! Here's the possible reasons:\n* Function doesn't returns value or returns wrong value's type.\n* Function have branches after returning a value.\n* Function doesn't matches types, or matches wrong.\nPlease check your code or open issue on github repo!".to_string(),
                        ErrorType::VerificationFailure,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line
                    );
                    std::process::exit(1);
                }
                GenError::throw(
                    format!("Function `{}` failed verification! Here's the possible reasons:\n* Function doesn't returns value or returns wrong value's type.\n* Function have branches after returning a value.\n* Function doesn't matches types, or matches wrong.\nPlease check your code or open issue on github repo!", &function_name),
                    ErrorType::VerificationFailure,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line
                );
                std::process::exit(1);
            }
        }

        // and switching to old position
        self.builder.position_at_end(old_position);

        // returning old variables
        for opt in old_variables {
            if let Some(value) = opt.1 {
                self.variables.insert(opt.0, value);
            }
        }

        // returning expectation value
        self.current_expectation_value = old_expectation_value;

        function_object
    }

    fn validate_types(&self, types: &[String], expected_type: String) -> bool {
        for typ in types {
            if typ != &expected_type {
                return false;
            }
        }

        true
    }

    #[allow(non_snake_case)]
    fn __boolean_strings(&mut self) -> (PointerValue<'ctx>, PointerValue<'ctx>) {
        if let Some(allocated_values) = self.boolean_strings_ptr {
            return allocated_values;
        }

        let fmts = (
            self.builder
                .build_global_string_ptr("true", "true_fmt")
                .unwrap()
                .as_pointer_value(),
            self.builder
                .build_global_string_ptr("false", "false_str")
                .unwrap()
                .as_pointer_value(),
        );

        self.boolean_strings_ptr = Some(fmts);
        fmts
    }

    pub fn get_module(&self) -> &Module<'ctx> {
        &self.module
    }
}
