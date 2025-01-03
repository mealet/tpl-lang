// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

mod builtin;
mod error;
mod function;
mod import;
mod libc;
mod variable;

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::{
        BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
    },
    AddressSpace,
};

use builtin::BuiltIn;
use std::{collections::HashMap, sync::LazyLock};

use error::{ErrorType, GenError};
use function::Function;
use import::ImportObject;
use variable::Variable;

use tpl_parser::{expressions::Expressions, statements::Statements, value::Value};

const LAMBDA_NAME: &str = "i_need_newer_inkwell_version"; // :D
static INT_TYPES_ORDER: LazyLock<HashMap<&str, u8>> =
    LazyLock::new(|| HashMap::from([("int8", 0), ("int16", 1), ("int32", 2), ("int64", 3)]));

pub fn get_int_order(o_type: &str) -> i8 {
    if let Some(order) = INT_TYPES_ORDER.get(o_type) {
        return *order as i8;
    }
    -1
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
        let main_fn_type = context.i32_type();
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
            .build_return(Some(&self.context.i32_type().const_int(0, false)));
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
                    let var_type = if Compiler::__is_ptr_type(&datatype) {
                        self.context
                            .ptr_type(AddressSpace::default())
                            .as_basic_type_enum()
                    } else {
                        self.get_basic_type(&datatype, line)
                    };

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
                        Variable::new(
                            datatype.clone(),
                            var_type,
                            alloca,
                            assigned_function.clone(),
                        ),
                    );

                    if let Some(intial_value) = value {
                        let expected_type = match datatype.clone().as_str() {
                            _ if datatype.contains("[") => {
                                Some(Compiler::clean_array_datatype(&datatype))
                            }
                            _ => Some(datatype.clone()),
                        };

                        let compiled_expression =
                            self.compile_expression(*intial_value, line, function, expected_type);

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

                        if Compiler::__is_ptr_type(&datatype) {
                            self.variables.insert(
                                identifier.clone(),
                                Variable::new(
                                    datatype.clone(),
                                    var_type,
                                    alloca,
                                    // compiled_expression.1.into_pointer_value(),
                                    assigned_function,
                                ),
                            );
                            let _ = self.builder.build_store(alloca, compiled_expression.1);
                        } else {
                            let _ = self.builder.build_store(alloca, compiled_expression.1);
                        }

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
                    let expr_value = self.compile_expression(
                        *value,
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
            Statements::SliceAssignStatement {
                identifier,
                index,
                value,
                line,
            } => {
                if let Some(var_ptr) = self.variables.clone().get(&identifier) {
                    let expr_value = self.compile_expression(
                        *value,
                        line,
                        function,
                        Some(Compiler::clean_array_datatype(&var_ptr.str_type)),
                    );

                    // matching datatypes

                    if expr_value.0 != Compiler::clean_array_datatype(&var_ptr.str_type) {
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

                    // loading array from pointer

                    let array = self
                        .builder
                        .build_load(var_ptr.basic_type, var_ptr.pointer, "")
                        .unwrap_or_else(|_| {
                            GenError::throw(
                                "Unable to load pointer value!",
                                ErrorType::BuildError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        })
                        .into_vector_value();

                    let index_value = self.compile_expression(*index, line, function, None);

                    // checking index value type

                    if !index_value.0.starts_with("int") {
                        GenError::throw(
                            "Non-integer index found!",
                            ErrorType::NotExpected,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    }

                    let new_vector = self
                        .builder
                        .build_insert_element(
                            array,
                            expr_value.1,
                            index_value.1.into_int_value(),
                            "",
                        )
                        .unwrap_or_else(|_| {
                            GenError::throw(
                                "Unable to insert element into vector!",
                                ErrorType::NotExpected,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        });

                    // storing new vector into pointer

                    let _ = self.builder.build_store(var_ptr.pointer, new_vector);
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
                    // building new binary expression
                    let new_expression = Expressions::Binary {
                        operand,
                        lhs: Box::new(Expressions::Value(Value::Identifier(identifier))),
                        rhs: value.clone(),
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
            Statements::DerefAssignStatement {
                identifier,
                value,
                line,
            } => {
                if let Some(var_ptr) = self.variables.clone().get(&identifier) {
                    let expr_value = self.compile_expression(
                        *value,
                        line,
                        function,
                        Some(var_ptr.str_type.clone()),
                    );

                    // matching datatypes

                    let raw_type = Compiler::__unwrap_ptr_type(&var_ptr.str_type);
                    if expr_value.0 != raw_type {
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

                    // loading pointer from a pointer

                    let ptr_type = self
                        .context
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum();
                    let raw_ptr = self
                        .builder
                        .build_load(ptr_type, var_ptr.pointer, "")
                        .unwrap_or_else(|_| {
                            GenError::throw(
                                "Unable to load a pointer!",
                                ErrorType::BuildError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        });

                    // storing value

                    let _ = self
                        .builder
                        .build_store(raw_ptr.into_pointer_value(), expr_value.1);
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
                    if then_basic_block.get_terminator().is_none() {
                        let _ = self.builder.build_unconditional_branch(merge_basic_block);
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

            Statements::ForStatement { initializer, condition, iterator, block, line } => {
                 // creating basic blocks
                let before_basic_block = self.context.append_basic_block(function, "for_before");
                let then_basic_block = self.context.append_basic_block(function, "for_then");
                let after_basic_block = self.context.append_basic_block(function, "for_after");

                // building initializer
                let _ = self.compile_statement(*initializer, function);

                // setting current position to block `before`

                if let Some(last_instruction) = self.current_block.get_last_instruction() {
                    if last_instruction.get_opcode() != inkwell::values::InstructionOpcode::Return {
                        let _ = self.builder.build_unconditional_branch(before_basic_block);
                    }
                }

                self.switch_block(before_basic_block);

                // building condition
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

                // building iterator

                let _ = self.compile_statement(*iterator, function);

                // returning to block `before` for comparing condition
                if let Some(last_instruction) = then_basic_block.get_last_instruction() {
                    if last_instruction.get_opcode() != inkwell::values::InstructionOpcode::Return {
                        let _ = self.builder.build_unconditional_branch(before_basic_block);
                    }
                }

                // setting builder position to `after` block
                self.switch_block(after_basic_block);
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

            Statements::Expression(expr) => match expr {
                Expressions::SubElement {
                    parent,
                    child,
                    line,
                } => {
                    self.compile_subelement(
                        Expressions::SubElement {
                            parent,
                            child,
                            line,
                        },
                        function,
                    );
                }
                _ => {
                    GenError::throw(
                        "Unsupported expression found! Please open issue with your code on Github!",
                        ErrorType::NotSupported,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        0,
                    );
                    std::process::exit(1);
                }
            },

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
            Expressions::Slice {
                object,
                index,
                line,
            } => {
                let obj =
                    self.compile_expression(*object, line, function, expected_datatype.clone());
                let idx = self.compile_expression(*index, line, function, expected_datatype);

                match obj.0.as_str() {
                    array_type if Compiler::__is_arr_type(array_type) => {
                        let raw_type = Compiler::clean_array_datatype(array_type);
                        let raw_len = Compiler::get_array_datatype_len(array_type);

                        let int_index = match idx.0 {
                            itype if itype.starts_with("int") => idx.1.into_int_value(),
                            _ => {
                                GenError::throw(
                                    "Non-integer slice index found!",
                                    ErrorType::TypeError,
                                    self.module_name.clone(),
                                    self.module_source.clone(),
                                    line,
                                );
                                std::process::exit(1);
                            }
                        };

                        let raw_index = int_index.get_sign_extended_constant().unwrap_or(0);
                        // if we cannot verify index on build, it will cause some bugs on runtime

                        if raw_index > raw_len as i64 - 1 || raw_index < 0 && raw_index != 0 {
                            GenError::throw(
                                format!(
                                    "Wrong array index found! Array len is {} but index is {}",
                                    raw_len, raw_index
                                ),
                                ErrorType::NotExpected,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        }

                        let output_value = self
                            .builder
                            .build_extract_element(obj.1.into_vector_value(), int_index, "")
                            .unwrap_or_else(|_| {
                                GenError::throw(
                                    "Unable to extract array element!",
                                    ErrorType::BuildError,
                                    self.module_name.clone(),
                                    self.module_source.clone(),
                                    line,
                                );
                                std::process::exit(1);
                            });

                        (raw_type, output_value)
                    }
                    _ => {
                        GenError::throw(
                            format!("Unsupported slicing type found: {}", obj.0),
                            ErrorType::NotSupported,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    }
                }
            }
            Expressions::Reference { object, line } => {
                match *object {
                    Expressions::Value(Value::Identifier(id)) => {
                        // referencing to a variable

                        let variable = self.variables.get(&id).unwrap_or_else(|| {
                            GenError::throw(
                                format!("Variable `{}` is not defined!", id),
                                ErrorType::NotDefined,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        });

                        (format!("{}*", variable.str_type), variable.pointer.into())
                    }
                    _ => {
                        GenError::throw(
                            "Unsupported expression for reference found",
                            ErrorType::NotSupported,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    }
                }
            }
            Expressions::Dereference { object, line } => {
                let value = self.compile_expression(
                    *object,
                    line,
                    function,
                    Some(
                        String::from("*"), // requesting raw pointer
                    ),
                );

                if !Compiler::__is_ptr_type(&value.0) {
                    GenError::throw(
                        format!("Non pointer type `{}` cannot by dereferenced!", value.0),
                        ErrorType::TypeError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                }

                let raw_type = Compiler::__unwrap_ptr_type(&value.0);
                let raw_basic_type = self.get_basic_type(&raw_type, line);

                let ptr_value = value.1.into_pointer_value();
                let ptr_type = self.context.ptr_type(AddressSpace::default());

                let loaded_ptr = self
                    .builder
                    .build_load(ptr_type, ptr_value, "")
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            "Unable to load pointer for dereference!",
                            ErrorType::BuildError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    });

                let loaded_value = self
                    .builder
                    .build_load(raw_basic_type, loaded_ptr.into_pointer_value(), "")
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            "Unable to load a pointer value for dereference!",
                            ErrorType::BuildError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    });

                (raw_type, loaded_value)
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
                    "int8" | "int16" | "int32" | "int64" => {
                        // checking if all sides are the same type
                        if !["int8", "int16", "int32", "int64"].contains(&right.0.as_str()) {
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
            Expressions::Boolean {
                operand,
                lhs,
                rhs,
                line,
            } => {
                let _ = (operand, lhs, rhs); // 0_0

                (
                    "bool".to_string(),
                    self.compile_condition(expr.clone(), line, function).into(),
                )
            }
            Expressions::SubElement {
                parent,
                child,
                line,
            } => self.compile_subelement(
                Expressions::SubElement {
                    parent,
                    child,
                    line,
                },
                function,
            ),
            Expressions::Array { values, len, line } => {
                let mut compiled_values = Vec::new();
                for val in values {
                    let compiled =
                        self.compile_expression(val, line, function, expected_datatype.clone());
                    compiled_values.push(compiled);
                }

                let types: Vec<String> = compiled_values.iter().map(|x| x.0.clone()).collect();
                let values: Vec<BasicValueEnum> = compiled_values.iter().map(|x| x.1).collect();

                let arr_type = types[0].clone();
                let arr_type_basic = match self.get_basic_type(&arr_type, line) {
                    BasicTypeEnum::IntType(int) => int.vec_type(len as u32),
                    BasicTypeEnum::PointerType(ptr) => ptr.vec_type(len as u32),
                    _ => unreachable!(),
                };

                if !Compiler::validate_types(&types, arr_type.clone()) {
                    GenError::throw(
                        format!(
                            "Array has type `{}`, but found: {}",
                            &arr_type,
                            types.join(", ")
                        ),
                        ErrorType::TypeError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                }

                let expr_type = format!("{}[{}]", arr_type, len);
                let mut expr_value = arr_type_basic.const_zero().as_basic_value_enum();

                for (index, value) in values.iter().enumerate() {
                    let index = self.context.i8_type().const_int(index as u64, false);
                    expr_value = expr_value
                        .into_vector_value()
                        .const_insert_element(index, *value);
                }

                (expr_type, expr_value)
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

    #[inline]
    fn clean_array_datatype(val: &str) -> String {
        val.split("[").collect::<Vec<&str>>()[0].to_string()
    }

    #[inline]
    fn get_array_datatype_len(val: &str) -> u64 {
        val.split("[").collect::<Vec<&str>>()[1]
            .split("]")
            .collect::<Vec<&str>>()[0]
            .trim()
            .parse::<u64>()
            .unwrap()
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
                        let unwrapped_type = Compiler::__unwrap_ptr_type(&exp);
                        let basic_type = self.get_basic_type(exp.as_str(), line).into_int_type();
                        let avaible_type = self.compile_value(Value::Integer(i), line, None);

                        if get_int_order(&avaible_type.0) > get_int_order(&unwrapped_type) {
                            GenError::throw(
                                format!(
                                    "Unable to compile `{}` value on `{}` type!",
                                    avaible_type.0, exp
                                ),
                                ErrorType::TypeError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1)
                        }

                        return (
                            unwrapped_type.to_string(),
                            basic_type.const_int(i as u64, true).into(),
                        );
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
                    i64::MIN..=i64::MAX => (
                        "int64".to_string(),
                        self.context.i64_type().const_int(i as u64, true).into(),
                    ),
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
                    let exp = expected.unwrap_or_default();

                    let value = if Compiler::__is_ptr_type(&exp) {
                        var_ptr.pointer.into()
                    } else {
                        self.builder
                            .build_load(var_ptr.basic_type, var_ptr.pointer, "")
                            .unwrap_or_else(|_| {
                                GenError::throw(
                                    format!("Error with loading `{}` variable", id),
                                    ErrorType::MemoryError,
                                    self.module_name.clone(),
                                    self.module_source.clone(),
                                    line,
                                );
                                std::process::exit(1);
                            })
                    };

                    (var_ptr.str_type.clone(), value)
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

    fn compile_subelement(
        &mut self,
        subelement: Expressions,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        match subelement {
            Expressions::SubElement {
                parent,
                child,
                line,
            } => {
                match *child {
                    Expressions::Call {
                        function_name,
                        arguments,
                        line,
                    } => {
                        // inserting parent as a first argument
                        let modified_args = [vec![*parent], arguments].concat();
                        let call = self.fn_call(function_name, modified_args, line, function);

                        call
                    }
                    _ => {
                        GenError::throw(
                            "Unsupported subelement found! Please open issue on github repo for bug report!",
                            ErrorType::TypeError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line
                        );
                        std::process::exit(1);
                    }
                }
            }
            _ => {
                GenError::throw(
                    "`compile_subelement` takes only 'SubElement' expression!",
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    0,
                );
                std::process::exit(1);
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
            Expressions::Boolean {
                operand,
                lhs,
                rhs,
                line,
            } => {
                match operand.as_str() {
                    "&&" => {
                        let left_condition = self.compile_condition(*lhs, line, function);
                        let right_condition = self.compile_condition(*rhs, line, function);

                        return self
                            .builder
                            .build_and(left_condition, right_condition, "and_cmp")
                            .unwrap_or_else(|_| {
                                GenError::throw(
                                    "Unable to build AND comparison!",
                                    ErrorType::BuildError,
                                    self.module_name.clone(),
                                    self.module_source.clone(),
                                    line,
                                );
                                std::process::exit(1);
                            });
                    }
                    "||" => {
                        let left_condition = self.compile_condition(*lhs, line, function);
                        let right_condition = self.compile_condition(*rhs, line, function);

                        return self
                            .builder
                            .build_or(left_condition, right_condition, "and_cmp")
                            .unwrap_or_else(|_| {
                                GenError::throw(
                                    "Unable to build OR comparison!",
                                    ErrorType::BuildError,
                                    self.module_name.clone(),
                                    self.module_source.clone(),
                                    line,
                                );
                                std::process::exit(1);
                            });
                    }
                    _ => {}
                }

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
                    | ("int64", "int64") => {
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
                "type" => return self.build_type_call(arguments, line, function),
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

                "to_str" => return self.build_to_str_call(arguments, line, function),
                "to_int8" => return self.build_to_int8_call(arguments, line, function),
                "to_int16" => return self.build_to_int16_call(arguments, line, function),
                "to_int32" => return self.build_to_int32_call(arguments, line, function),
                "to_int64" => return self.build_to_int64_call(arguments, line, function),
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

    #[inline]
    fn get_basic_type(&self, datatype: &str, line: usize) -> BasicTypeEnum<'ctx> {
        match datatype {
            _ if datatype.starts_with("fn<") => {
                let fn_type = datatype.replace("fn<", "").replace(">", "");
                self.get_basic_type(fn_type.as_str(), line)
            }
            _ if datatype.contains("[") => {
                let type_parts = datatype.split("[").collect::<Vec<&str>>();
                let raw_type = type_parts[0];
                let array_len: u32 = type_parts[1].split("]").collect::<Vec<&str>>()[0]
                    .parse()
                    .unwrap_or_else(|_| {
                        GenError::throw(
                            "Unable to compile array's length!",
                            ErrorType::BuildError,
                            self.module_name.clone(),
                            self.module_source.clone(),
                            line,
                        );
                        std::process::exit(1);
                    });

                match self.get_basic_type(raw_type, line) {
                    BasicTypeEnum::IntType(int) => int.vec_type(array_len).into(),
                    BasicTypeEnum::PointerType(ptr) => ptr.vec_type(array_len).into(),
                    _ => unreachable!(),
                }
            }
            _ if Compiler::__is_ptr_type(datatype) => {
                let unwrapped_type = Compiler::__unwrap_ptr_type(datatype);
                self.get_basic_type(&unwrapped_type, line)
            }
            "int8" => self.context.i8_type().into(),
            "int16" => self.context.i16_type().into(),
            "int32" => self.context.i32_type().into(),
            "int64" => self.context.i64_type().into(),
            "bool" => self.context.bool_type().into(),
            "str" => self.context.ptr_type(AddressSpace::default()).into(),
            "auto" => self.context.i8_type().into(),
            "void" => self.context.ptr_type(AddressSpace::default()).into(),
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

    #[inline]
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

        // add terminator if dont have
        let terminator_instructions = self
            .builder
            .get_insert_block()
            .unwrap()
            .get_instructions()
            .filter(|x| x.is_terminator());
        if terminator_instructions.count() < 1 && function_type != *"void" {
            let _ = self
                .builder
                .build_return(Some(&match function_type.as_str() {
                    "int8" | "int16" | "int32" | "int64" => {
                        self.compile_value(Value::Integer(0), line, Some(function_type.clone()))
                            .1
                    }
                    "str" => {
                        self.compile_value(
                            Value::String("@tplc:auto-return".to_string()),
                            line,
                            None,
                        )
                        .1
                    }
                    "bool" => self.compile_value(Value::Boolean(false), line, None).1,
                    _ => unreachable!(),
                }));
        };

        // verification

        if !function.verify(true) {
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

    fn validate_types(types: &[String], expected_type: String) -> bool {
        for typ in types {
            if typ != &expected_type {
                return false;
            }
        }

        true
    }

    #[allow(non_snake_case)]
    #[inline]
    fn __is_ptr_type(type_str: &str) -> bool {
        type_str.chars().last().unwrap_or('\0') == '*'
    }

    #[allow(non_snake_case)]
    #[inline]
    fn __is_arr_type(type_str: &str) -> bool {
        type_str.contains("[") && type_str.contains("]")
    }

    #[allow(non_snake_case)]
    #[inline]
    fn __unwrap_ptr_type(type_str: &str) -> String {
        if Compiler::__is_ptr_type(type_str) {
            let chars = type_str.chars().collect::<Vec<char>>();
            return chars[0..chars.len() - 1].iter().collect::<String>();
        };
        type_str.to_string()
    }

    #[allow(non_snake_case)]
    #[inline]
    fn __type_fmt(type_str: &str) -> String {
        match type_str {
            "int8" => "%d",
            "int16" => "%hd",
            "int32" => "%d",
            "int64" => "%lld",
            "bool" => "%s",
            "str" => "%s",
            _ => unreachable!(),
        }
        .to_string()
    }

    #[allow(non_snake_case)]
    #[inline]
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
                .build_global_string_ptr("false", "false_fmt")
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

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::module::Linkage;
    use libc::Libc;

    #[test]
    fn validate_types_test() {
        let types_array = [
            "int8".to_string(),
            "int8".to_string(),
            "int8".to_string(),
            "int8".to_string(),
        ];
        let expected_type = "int8".to_string();

        assert!(Compiler::validate_types(&types_array, expected_type));
    }

    #[test]
    #[should_panic]
    fn validate_types_test_2() {
        let types_array = [
            "int8".to_string(),
            "int8".to_string(),
            "int32".to_string(),
            "int8".to_string(),
        ];
        let expected_type = "int8".to_string();

        assert!(Compiler::validate_types(&types_array, expected_type));
    }

    #[test]
    fn boolean_strings_alloca_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let ptrs = compiler.__boolean_strings();

        assert_eq!(
            (
                ptrs.0.get_name().to_string_lossy().to_string(),
                ptrs.1.get_name().to_string_lossy().to_string()
            ),
            (String::from("true_fmt"), String::from("false_fmt"))
        );

        assert_eq!((ptrs.0.is_null(), ptrs.1.is_null()), (false, false));

        assert_eq!((ptrs.0.is_undef(), ptrs.1.is_undef()), (false, false));

        assert_eq!((ptrs.0.is_const(), ptrs.1.is_const()), (true, true));
    }

    #[test]
    fn __c_printf_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let printf_function = compiler.__c_printf();

        assert_eq!(printf_function.get_linkage(), Linkage::External);
        assert!(!printf_function.is_null());
        assert!(!printf_function.is_undef());
        assert!(printf_function.verify(true));
        assert_eq!(
            printf_function.get_name().to_string_lossy().to_string(),
            String::from("printf")
        );
        assert_eq!(
            printf_function.get_type(),
            compiler.context.i32_type().fn_type(
                &[compiler.context.ptr_type(AddressSpace::default()).into()],
                true
            )
        );
    }

    #[test]
    fn __c_strcat_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let strcat = compiler.__c_strcat();

        assert_eq!(strcat.get_linkage(), Linkage::External);
        assert!(!strcat.is_null());
        assert!(!strcat.is_undef());
        assert!(strcat.verify(true));
        assert_eq!(
            strcat.get_name().to_string_lossy().to_string(),
            String::from("strcat")
        );
        assert_eq!(
            strcat.get_type(),
            compiler.context.ptr_type(AddressSpace::default()).fn_type(
                &[
                    compiler.context.ptr_type(AddressSpace::default()).into(),
                    compiler.context.ptr_type(AddressSpace::default()).into(),
                ],
                true
            )
        );
    }

    #[test]
    fn switch_block_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);
        let main_block = compiler.current_block;
        let block = compiler
            .context
            .append_basic_block(compiler.main_function, "test_block");

        compiler.switch_block(block);

        assert_eq!(compiler.builder.get_insert_block().unwrap(), block);
        assert_eq!(block.get_previous_basic_block().unwrap(), main_block);

        assert_eq!(
            compiler.builder.get_insert_block().unwrap().get_name(),
            block.get_name()
        );

        compiler.switch_block(main_block);

        assert_eq!(compiler.builder.get_insert_block().unwrap(), main_block);
    }

    #[test]
    fn compile_value_test() {
        let ctx = inkwell::context::Context::create();
        let compiler = Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let int8 = compiler.compile_value(Value::Integer(15), 0, None);
        let int16 = compiler.compile_value(Value::Integer(256), 0, None);
        let int32 = compiler.compile_value(Value::Integer(65_535), 0, None);
        let int64 = compiler.compile_value(Value::Integer(2_147_483_648), 0, None);

        let boolean_true = compiler.compile_value(Value::Boolean(true), 0, None);
        let boolean_false = compiler.compile_value(Value::Boolean(false), 0, None);

        let str = compiler.compile_value(Value::String(String::from("some")), 0, None);

        assert_eq!(
            (
                int8.0,
                int16.0,
                int32.0,
                int64.0,
                boolean_true.0,
                boolean_false.0,
                str.0
            ),
            (
                String::from("int8"),
                String::from("int16"),
                String::from("int32"),
                String::from("int64"),
                String::from("bool"),
                String::from("bool"),
                String::from("str"),
            )
        );

        assert!(int8.1.is_int_value());
        assert!(int16.1.is_int_value());
        assert!(int32.1.is_int_value());
        assert!(int64.1.is_int_value());
        assert!(boolean_true.1.is_int_value());
        assert!(boolean_false.1.is_int_value());
        assert!(str.1.is_pointer_value());
    }

    #[test]
    fn compile_condition_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let condition_true = Expressions::Boolean {
            operand: String::from("=="),
            lhs: Box::new(Expressions::Value(Value::Integer(123))),
            rhs: Box::new(Expressions::Value(Value::Integer(123))),
            line: 0,
        };

        let condition_false = Expressions::Boolean {
            operand: String::from("=="),
            lhs: Box::new(Expressions::Value(Value::Integer(0))),
            rhs: Box::new(Expressions::Value(Value::Integer(123))),
            line: 0,
        };

        let compiled_true_condition =
            compiler.compile_condition(condition_true, 0, compiler.main_function);
        let compiled_false_condition =
            compiler.compile_condition(condition_false, 0, compiler.main_function);

        assert_eq!(
            compiled_true_condition
                .get_zero_extended_constant()
                .unwrap(),
            1
        );

        assert_eq!(
            compiled_false_condition
                .get_zero_extended_constant()
                .unwrap(),
            0
        );
    }

    #[test]
    fn compile_array_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let array_expr = Expressions::Array {
            values: vec![
                Expressions::Value(Value::Integer(5)),
                Expressions::Value(Value::Integer(3)),
                Expressions::Value(Value::Integer(4)),
            ],
            len: 3,
            line: 0,
        };

        let compiled = compiler.compile_expression(array_expr, 0, compiler.main_function, None);
        assert_eq!(compiled.0, String::from("int8[3]"))
    }

    #[test]
    fn type_function_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let value_int8 = Expressions::Value(Value::Integer(0));

        let call_result = compiler.build_type_call(vec![value_int8], 0, compiler.main_function);
        let ptr_value = call_result.1.into_pointer_value().to_string();

        assert_eq!(call_result.0, "str".to_string());
        assert!(ptr_value.contains("int8"));
    }
}
