use inkwell::{
    values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue},
    AddressSpace,
};

use crate::{
    error::{ErrorType, GenError},
    get_int_order,
    libc::Libc,
    Compiler,
};

use tpl_parser::{expressions::Expressions, value::Value};

pub trait BuiltIn<'ctx> {
    // input output
    fn build_print_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    );
    fn build_input_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);

    // helpful functions
    fn build_type_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);
    fn build_len_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);
    fn build_size_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);
    fn build_concat_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);

    // conversions
    fn build_to_str_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);
    fn build_to_int8_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);
    fn build_to_int16_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);
    fn build_to_int32_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);
    fn build_to_int64_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);

    // allocation
    fn build_malloc_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);

    fn build_realloc_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);

    fn build_free_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);

    // files
    fn build_file_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);

    fn build_close_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>);
}

impl<'ctx> BuiltIn<'ctx> for Compiler<'ctx> {
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
            std::process::exit(1);
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

        if !Compiler::validate_types(&[left_arg.0, right_arg.0], "str".to_string()) {
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
            .build_call(
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
        let mut fmts: Vec<String> = Vec::new();
        let mut values: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
        let printf_fn = self.__c_printf();

        for arg in arguments {
            let compiled_arg = self.compile_expression(
                arg.clone(),
                line,
                function,
                self.current_expectation_value.clone(),
            );
            let mut basic_value = compiled_arg.1;

            match compiled_arg.0.as_str() {
                "void" => continue,
                _ if compiled_arg.0.contains("[") => {
                    // array
                    let array_value = basic_value.into_vector_value();
                    let array_type = compiled_arg.0.split("[").collect::<Vec<&str>>()[0];

                    let array_len = {
                        let left_parts = compiled_arg.0.split("[").collect::<Vec<&str>>();

                        let right_parts = left_parts[1].split("]").collect::<Vec<&str>>();

                        right_parts[0].parse::<u32>().unwrap_or_else(|_| {
                            GenError::throw(
                                "Unable to get array length!",
                                ErrorType::BuildError,
                                self.module_name.clone(),
                                self.module_source.clone(),
                                line,
                            );
                            std::process::exit(1);
                        })
                    };

                    let mut new_fmts: Vec<&str> = Vec::new();

                    for array_index in 0..array_len {
                        let mut element = array_value.const_extract_element(
                            self.context.i32_type().const_int(array_index as u64, false),
                        );

                        let format_string = match array_type {
                            "int8" => "%d",
                            "int16" => "%hd",
                            "int32" => "%d",
                            "int64" => "%lld",
                            "bool" => {
                                let (_true, _false) = self.__boolean_strings();

                                if let BasicValueEnum::IntValue(int) = element {
                                    element = self
                                        .builder
                                        .build_select(int, _true, _false, "bool_fmt_str")
                                        .unwrap();
                                }

                                "%s"
                            }
                            "str" => "\"%s\"",
                            "char" => "'%c'",
                            _ => {
                                GenError::throw(
                                    format!(
                                        "Type `{}` is not supported for 'print' function!",
                                        array_type
                                    ),
                                    ErrorType::NotSupported,
                                    self.module_name.clone(),
                                    self.module_source.clone(),
                                    line,
                                );
                                std::process::exit(1);
                            }
                        };

                        new_fmts.push(format_string);
                        values.push(element.into());
                    }

                    for (index, fmt) in new_fmts.iter().enumerate() {
                        let mut output_string = format!("{},", fmt);

                        if index == 0 {
                            output_string = format!("[{},", fmt)
                        } else if index == new_fmts.len() - 1 {
                            output_string = format!("{}]", fmt);
                        }

                        fmts.push(output_string);

                        // i know that this code is piece of shit, but i wanna sleep ._.
                        // i'll figure it out tomorrow
                        //
                        // nah i didn't figured it out
                    }

                    continue;
                }
                _ => {}
            }

            let format_string = match compiled_arg.0.as_str() {
                "int8" => "%d",
                "int16" => "%hd",
                "int32" => "%d",
                "int64" => "%lld",
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
                "char" => "%c",
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
            }
            .to_string();

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

        let _ = self.builder.build_call(printf_fn, &printf_arguments, "");
    }

    fn build_input_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() > 1 {
            GenError::throw(
                "Function `input()` takes only 0 or 1 arguments! Example: input(\"Type here: \")",
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        if let Some(argument) = arguments.first() {
            let compiled_argument = self.compile_expression(argument.clone(), line, function, None);
            let printf_fn = self.__c_printf();

            if compiled_argument.0 != "str" {
                GenError::throw(
                    "Function `input()` takes only string as argument!",
                    ErrorType::NotExpected,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }

            let _ = self
                .builder
                .build_call(printf_fn, &[compiled_argument.1.into()], "");
        }

        let scanf_fn = self.__c_scanf();
        let format_string = self
            .builder
            .build_global_string_ptr("%s", "")
            .unwrap()
            .as_basic_value_enum();

        let result_alloca = self
            .builder
            .build_alloca(self.context.ptr_type(AddressSpace::default()), "")
            .unwrap();

        let _ = self
            .builder
            .build_call(scanf_fn, &[format_string.into(), result_alloca.into()], "")
            .unwrap();

        ("str".to_string(), result_alloca.into())
    }

    fn build_type_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `type()` requires only 1 argument, but {} found!",
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_arg = self.compile_expression(arguments[0].clone(), line, function, None);
        let arg_type_string = self
            .builder
            .build_global_string_ptr(compiled_arg.0.as_str(), "_type")
            .unwrap_or_else(|_| {
                GenError::throw(
                    "Unable to allocate memory for type fmt!",
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            })
            .as_pointer_value();

        (String::from("str"), arg_type_string.into())
    }

    fn build_len_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `len()` requires only 1 argument, but {} found!",
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_arg = self.compile_expression(arguments[0].clone(), line, function, None);

        match compiled_arg.0.as_str() {
            argtype if Compiler::__is_arr_type(argtype) => {
                let length = Compiler::get_array_datatype_len(&compiled_arg.0);
                let basic_value = self
                    .context
                    .i64_type()
                    .const_int(length, false)
                    .as_basic_value_enum();

                (String::from("int64"), basic_value)
            }
            "str" => {
                let strlen_fn = self.__c_strlen();
                let value = self
                    .builder
                    .build_call(strlen_fn, &[compiled_arg.1.into()], "")
                    .unwrap()
                    .try_as_basic_value()
                    .left()
                    .unwrap();

                (String::from("int64"), value)
            }
            _ => {
                GenError::throw(
                    format!(
                        "Type `{}` is not supported for `len()` function!",
                        &compiled_arg.0
                    ),
                    ErrorType::NotSupported,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }
        }
    }

    fn build_size_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `len()` requires only 1 argument, but {} found!",
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_type = match arguments[0].clone() {
            Expressions::Value(Value::Keyword(arg_type)) => arg_type,
            _ => {
                self.compile_expression(arguments[0].clone(), line, function, None)
                    .0
            }
        };

        let mut raw_type = compiled_type;
        let mut type_multiplier = 1;

        loop {
            match raw_type {
                ctype if Compiler::__is_ptr_type(&ctype) => {
                    raw_type = Compiler::__unwrap_ptr_type(&ctype);
                }
                ctype if Compiler::__is_arr_type(&ctype) => {
                    raw_type = Compiler::clean_array_datatype(&ctype);
                    type_multiplier *= Compiler::get_array_datatype_len(&ctype);
                }
                ctype if ctype.starts_with("fn<") => {
                    raw_type = ctype.split("fn<").collect::<Vec<&str>>()[0]
                        .split(">")
                        .collect::<Vec<&str>>()[0]
                        .to_string();
                }
                _ => break,
            };
        }

        let size = crate::TYPE_SIZES
            .get(&raw_type.as_str())
            .unwrap_or_else(|| {
                GenError::throw(
                    format!("Unsupported for size type found: `{}`", raw_type),
                    ErrorType::NotSupported,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            })
            * type_multiplier;

        let constant = self.context.i64_type().const_int(size, false);

        (String::from("int64"), constant.into())
    }

    // conversion
    // int

    fn build_to_int8_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        #[allow(non_snake_case)]
        let (TARGET_TYPE, TARGET_BASIC_TYPE, TARGET_TYPE_FORMAT) =
            ("int8", self.context.i8_type(), "%d");

        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `to_{}()` requires only 1 argument, but {} found!",
                    TARGET_TYPE,
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_arg = self.compile_expression(arguments[0].clone(), line, function, None);

        // checks
        match compiled_arg.0.as_str() {
            ctype if ctype == TARGET_TYPE => return compiled_arg,
            "str" => {
                let sscanf_fn = self.__c_sscanf();
                let format_string = self
                    .builder
                    .build_global_string_ptr(TARGET_TYPE_FORMAT, TARGET_TYPE)
                    .unwrap()
                    .as_basic_value_enum();

                let result_alloca = self.builder.build_alloca(TARGET_BASIC_TYPE, "").unwrap();

                let _ = self.builder.build_call(
                    sscanf_fn,
                    &[
                        compiled_arg.1.into(),
                        format_string.into(),
                        result_alloca.into(),
                    ],
                    "",
                );

                let result_value = self
                    .builder
                    .build_load(TARGET_BASIC_TYPE, result_alloca, "")
                    .unwrap();

                return (TARGET_TYPE.to_string(), result_value);
            }
            _ if !compiled_arg.0.contains("int") => {
                GenError::throw(
                    format!("Unable to convert non-int type to `{}`", TARGET_TYPE),
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }
            _ => {}
        }

        let target_order = get_int_order(TARGET_TYPE);
        let compiled_order = get_int_order(compiled_arg.0.as_str());
        let converted_value = if compiled_order > target_order {
            // cutting bits
            let val = compiled_arg.1;
            let truncated = self
                .builder
                .build_int_truncate(
                    val.into_int_value(),
                    TARGET_BASIC_TYPE,
                    format!("to_{}_trunc", TARGET_TYPE).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        "Unable to truncate integer value!",
                        ErrorType::BuildError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            truncated
        } else {
            let val = compiled_arg.1;
            let extended = self
                .builder
                .build_int_s_extend(
                    val.into_int_value(),
                    TARGET_BASIC_TYPE,
                    format!("to_{}_sext", TARGET_TYPE).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        "Unable to extend integer value!",
                        ErrorType::BuildError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            extended
        };

        (String::from(TARGET_TYPE), converted_value.into())
    }

    fn build_to_int16_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        #[allow(non_snake_case)]
        let (TARGET_TYPE, TARGET_BASIC_TYPE, TARGET_TYPE_FORMAT) =
            ("int16", self.context.i16_type(), "%d");

        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `to_{}()` requires only 1 argument, but {} found!",
                    TARGET_TYPE,
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_arg = self.compile_expression(arguments[0].clone(), line, function, None);

        // checks
        match compiled_arg.0.as_str() {
            ctype if ctype == TARGET_TYPE => return compiled_arg,
            "str" => {
                let sscanf_fn = self.__c_sscanf();
                let format_string = self
                    .builder
                    .build_global_string_ptr(TARGET_TYPE_FORMAT, TARGET_TYPE)
                    .unwrap()
                    .as_basic_value_enum();

                let result_alloca = self.builder.build_alloca(TARGET_BASIC_TYPE, "").unwrap();

                let _ = self.builder.build_call(
                    sscanf_fn,
                    &[
                        compiled_arg.1.into(),
                        format_string.into(),
                        result_alloca.into(),
                    ],
                    "",
                );

                let result_value = self
                    .builder
                    .build_load(TARGET_BASIC_TYPE, result_alloca, "")
                    .unwrap();

                return (TARGET_TYPE.to_string(), result_value);
            }

            _ if !compiled_arg.0.contains("int") => {
                GenError::throw(
                    format!("Unable to convert non-int type to `{}`", TARGET_TYPE),
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }
            _ => {}
        }

        let target_order = get_int_order(TARGET_TYPE);
        let compiled_order = get_int_order(compiled_arg.0.as_str());
        let converted_value = if compiled_order > target_order {
            // cutting bits
            let val = compiled_arg.1;
            let truncated = self
                .builder
                .build_int_truncate(
                    val.into_int_value(),
                    TARGET_BASIC_TYPE,
                    format!("to_{}_trunc", TARGET_TYPE).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        "Unable to truncate integer value!",
                        ErrorType::BuildError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            truncated
        } else {
            let val = compiled_arg.1;
            let extended = self
                .builder
                .build_int_s_extend(
                    val.into_int_value(),
                    TARGET_BASIC_TYPE,
                    format!("to_{}_sext", TARGET_TYPE).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        "Unable to extend integer value!",
                        ErrorType::BuildError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            extended
        };

        (String::from(TARGET_TYPE), converted_value.into())
    }

    fn build_to_int32_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        #[allow(non_snake_case)]
        let (TARGET_TYPE, TARGET_BASIC_TYPE, TARGET_TYPE_FORMAT) =
            ("int32", self.context.i32_type(), "%d");

        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `to_{}()` requires only 1 argument, but {} found!",
                    TARGET_TYPE,
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_arg = self.compile_expression(arguments[0].clone(), line, function, None);

        // checks
        match compiled_arg.0.as_str() {
            ctype if ctype == TARGET_TYPE => return compiled_arg,
            "str" => {
                let sscanf_fn = self.__c_sscanf();
                let format_string = self
                    .builder
                    .build_global_string_ptr(TARGET_TYPE_FORMAT, TARGET_TYPE)
                    .unwrap()
                    .as_basic_value_enum();

                let result_alloca = self.builder.build_alloca(TARGET_BASIC_TYPE, "").unwrap();

                let _ = self.builder.build_call(
                    sscanf_fn,
                    &[
                        compiled_arg.1.into(),
                        format_string.into(),
                        result_alloca.into(),
                    ],
                    "",
                );

                let result_value = self
                    .builder
                    .build_load(TARGET_BASIC_TYPE, result_alloca, "")
                    .unwrap();

                return (TARGET_TYPE.to_string(), result_value);
            }

            _ if !compiled_arg.0.contains("int") => {
                GenError::throw(
                    format!("Unable to convert non-int type to `{}`", TARGET_TYPE),
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }
            _ => {}
        }

        let target_order = get_int_order(TARGET_TYPE);
        let compiled_order = get_int_order(compiled_arg.0.as_str());
        let converted_value = if compiled_order > target_order {
            // cutting bits
            let val = compiled_arg.1;
            let truncated = self
                .builder
                .build_int_truncate(
                    val.into_int_value(),
                    TARGET_BASIC_TYPE,
                    format!("to_{}_trunc", TARGET_TYPE).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        "Unable to truncate integer value!",
                        ErrorType::BuildError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            truncated
        } else {
            let val = compiled_arg.1;
            let extended = self
                .builder
                .build_int_s_extend(
                    val.into_int_value(),
                    TARGET_BASIC_TYPE,
                    format!("to_{}_sext", TARGET_TYPE).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        "Unable to extend integer value!",
                        ErrorType::BuildError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            extended
        };

        (String::from(TARGET_TYPE), converted_value.into())
    }

    fn build_to_int64_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        #[allow(non_snake_case)]
        let (TARGET_TYPE, TARGET_BASIC_TYPE, TARGET_TYPE_FORMAT) =
            ("int64", self.context.i64_type(), "%ld");

        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `to_{}()` requires only 1 argument, but {} found!",
                    TARGET_TYPE,
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_arg = self.compile_expression(arguments[0].clone(), line, function, None);

        // checks
        match compiled_arg.0.as_str() {
            ctype if ctype == TARGET_TYPE => return compiled_arg,
            "str" => {
                let sscanf_fn = self.__c_sscanf();
                let format_string = self
                    .builder
                    .build_global_string_ptr(TARGET_TYPE_FORMAT, TARGET_TYPE)
                    .unwrap()
                    .as_basic_value_enum();

                let result_alloca = self.builder.build_alloca(TARGET_BASIC_TYPE, "").unwrap();

                let _ = self.builder.build_call(
                    sscanf_fn,
                    &[
                        compiled_arg.1.into(),
                        format_string.into(),
                        result_alloca.into(),
                    ],
                    "",
                );

                let result_value = self
                    .builder
                    .build_load(TARGET_BASIC_TYPE, result_alloca, "")
                    .unwrap();

                return (TARGET_TYPE.to_string(), result_value);
            }

            _ if !compiled_arg.0.contains("int") => {
                GenError::throw(
                    format!("Unable to convert non-int type to `{}`", TARGET_TYPE),
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            }
            _ => {}
        }

        let target_order = get_int_order(TARGET_TYPE);
        let compiled_order = get_int_order(compiled_arg.0.as_str());
        let converted_value = if compiled_order > target_order {
            // cutting bits
            let val = compiled_arg.1;
            let truncated = self
                .builder
                .build_int_truncate(
                    val.into_int_value(),
                    TARGET_BASIC_TYPE,
                    format!("to_{}_trunc", TARGET_TYPE).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        "Unable to truncate integer value!",
                        ErrorType::BuildError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            truncated
        } else {
            let val = compiled_arg.1;
            let extended = self
                .builder
                .build_int_s_extend(
                    val.into_int_value(),
                    TARGET_BASIC_TYPE,
                    format!("to_{}_sext", TARGET_TYPE).as_str(),
                )
                .unwrap_or_else(|_| {
                    GenError::throw(
                        "Unable to extend integer value!",
                        ErrorType::BuildError,
                        self.module_name.clone(),
                        self.module_source.clone(),
                        line,
                    );
                    std::process::exit(1);
                });

            extended
        };

        (String::from(TARGET_TYPE), converted_value.into())
    }

    // str

    fn build_to_str_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `to_str()` requires only 1 argument, but {} found!",
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_arg = self.compile_expression(arguments[0].clone(), line, function, None);
        let arg_fmt = Compiler::__type_fmt(&compiled_arg.0);
        let arg_fmt_ptr = self
            .builder
            .build_global_string_ptr(&arg_fmt, "_to_str_fmt")
            .unwrap_or_else(|_| {
                GenError::throw(
                    "Unable to allocate format pointer!",
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            })
            .as_basic_value_enum();

        let data_ptr_size = self.context.i8_type().const_int(10, false);
        let data_ptr = self
            .builder
            .build_array_alloca(
                self.context.ptr_type(AddressSpace::default()),
                data_ptr_size,
                "_to_str_alloca",
            )
            .unwrap_or_else(|_| {
                GenError::throw(
                    "Unable to create array alloca!",
                    ErrorType::MemoryError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            });

        let sprintf_fn = self.__c_sprintf();

        let _ = self
            .builder
            .build_call(
                sprintf_fn,
                &[data_ptr.into(), arg_fmt_ptr.into(), compiled_arg.1.into()],
                "_to_string_call",
            )
            .unwrap_or_else(|_| {
                GenError::throw(
                    "Call `to_str()` failed!",
                    ErrorType::BuildError,
                    self.module_name.clone(),
                    self.module_source.clone(),
                    line,
                );
                std::process::exit(1);
            });

        ("str".to_string(), data_ptr.into())
    }

    fn build_malloc_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `malloc` requires 1 argument, but {} found!",
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_size = self.compile_expression(
            arguments[0].clone(),
            line,
            function,
            Some(String::from("int64")),
        );

        if !compiled_size.0.starts_with("int") {
            dbg!(arguments);
            GenError::throw(
                "Non-integer size for allocation found!",
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let malloc_fn = self.__c_malloc();

        let result = self
            .builder
            .build_call(malloc_fn, &[compiled_size.1.into()], "")
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap();

        let output_type = self
            .current_expectation_value
            .clone()
            .unwrap_or(String::from("void*"));

        if !Compiler::__is_ptr_type(&output_type) {
            GenError::throw(
                format!(
                    "Non-pointer type `{}` requested for `malloc()`",
                    output_type
                ),
                ErrorType::TypeError,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        (output_type, result)
    }

    fn build_free_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 1 {
            GenError::throw(
                format!(
                    "Function `free` requires 1 arguments, but {} found!",
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_arg = self.compile_expression(arguments[0].clone(), line, function, None);

        if !Compiler::__is_ptr_type(&compiled_arg.0) {
            GenError::throw(
                "Function `free` requires pointer as an argument!",
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let free_fn = self.__c_free();
        let _ = self
            .builder
            .build_call(free_fn, &[compiled_arg.1.into()], "")
            .unwrap();

        (
            String::from("void"),
            self.context.bool_type().const_zero().into(),
        )
    }

    fn build_realloc_call(
        &mut self,
        arguments: Vec<Expressions>,
        line: usize,
        function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 2 {
            GenError::throw(
                format!(
                    "Function `realloc` requires 2 arguments, but {} found!",
                    arguments.len()
                ),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let argument_ptr = self.compile_expression(arguments[0].clone(), line, function, None);

        if !Compiler::__is_ptr_type(&argument_ptr.0) {
            GenError::throw(
                "Function `realloc` requires pointer as first argument!",
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let compiled_size = self.compile_expression(arguments[1].clone(), line, function, None);

        if !compiled_size.0.starts_with("int") {
            dbg!(arguments);
            GenError::throw(
                "Non-integer size for allocation found!",
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line,
            );
            std::process::exit(1);
        }

        let realloc_fn = self.__c_realloc();
        let result_ptr = self
            .builder
            .build_call(
                realloc_fn,
                &[argument_ptr.1.into(), compiled_size.1.into()],
                "",
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap();

        (argument_ptr.0, result_ptr)
    }

    fn build_file_call(
            &mut self,
            arguments: Vec<Expressions>,
            line: usize,
            function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 2 {
            GenError::throw(
                format!("Function `file` requires 2 arguments, but {} found", arguments.len()),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line
            );
            std::process::exit(1);
        }

        let path_to_file = self.compile_expression(arguments[0].clone(), line, function, None);
        let open_mode = self.compile_expression(arguments[1].clone(), line, function, None);

        if path_to_file.0 != String::from("str")
        && open_mode.0 != String::from("str") {
            GenError::throw(
                "Wrong arguments found! Function `file` takes next arguments: file(str path, str mode)",
                ErrorType::TypeError,
                self.module_name.clone(),
                self.module_source.clone(),
                line
            );
            std::process::exit(1);
        }

        let fopen_fn = self.__c_fopen();
        let call_result = self
            .builder
            .build_call(
                fopen_fn,
                &[
                    path_to_file.1.into(),
                    open_mode.1.into()
                ],
                ""
            )
            .unwrap()
            .try_as_basic_value()
            .left()
            .unwrap();

        (String::from("FILE*"), call_result)
    }

    fn build_close_call(
            &mut self,
            arguments: Vec<Expressions>,
            line: usize,
            function: FunctionValue<'ctx>,
    ) -> (String, BasicValueEnum<'ctx>) {
        if arguments.len() != 1 {
            GenError::throw(
                format!("Function `close` requires 1 argument, but {} found", arguments.len()),
                ErrorType::NotExpected,
                self.module_name.clone(),
                self.module_source.clone(),
                line
            );
            std::process::exit(1);
        }

        let file_ptr = self.compile_expression(arguments[0].clone(), line, function, None);

        if file_ptr.0 != String::from("FILE*") {
            GenError::throw(
                "Function `close` requires file pointer as an argument!",
                ErrorType::TypeError,
                self.module_name.clone(),
                self.module_source.clone(),
                line
            );
            std::process::exit(1);
        }

        let fclose_fn = self.__c_fclose();
        let _ = self
            .builder
            .build_call(
                fclose_fn,
                &[
                    file_ptr.1.into()
                ],
                ""
            )
            .unwrap();

        (String::from("void"), self.context.bool_type().const_zero().into())
    }
}
