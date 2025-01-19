use crate::Compiler;
use inkwell::{module::Linkage, values::FunctionValue, AddressSpace};

pub trait Libc {
    type Function;

    // stdio

    fn __c_printf(&mut self) -> Self::Function;
    fn __c_sprintf(&mut self) -> Self::Function;

    // strings

    fn __c_strcat(&mut self) -> Self::Function;
    fn __c_strcmp(&mut self) -> Self::Function;
    fn __c_strlen(&mut self) -> Self::Function;

    fn __c_scanf(&mut self) -> Self::Function;
    fn __c_sscanf(&mut self) -> Self::Function;

    // allocation

    fn __c_malloc(&mut self) -> Self::Function;
    fn __c_realloc(&mut self) -> Self::Function;
    fn __c_free(&mut self) -> Self::Function;

    // filesystem

    fn __c_fopen(&mut self) -> Self::Function;
    fn __c_fclose(&mut self) -> Self::Function;

    fn __c_fprintf(&mut self) -> Self::Function;
    fn __c_fwrite(&mut self) -> Self::Function;
    fn __c_fgetc(&mut self) -> Self::Function;

    fn __c_rewind(&mut self) -> Self::Function;
    fn __c_fseek(&mut self) -> Self::Function;
    fn __c_fsetpos(&mut self) -> Self::Function;
    fn __c_ftell(&mut self) -> Self::Function;
    fn __c_feof(&mut self) -> Self::Function;
}

impl<'ctx> Libc for Compiler<'ctx> {
    type Function = FunctionValue<'ctx>;

    fn __c_sprintf(&mut self) -> FunctionValue<'ctx> {
        if let Some(function_value) = self.built_functions.get("sprintf") {
            return *function_value;
        }

        let sprintf_type = self.context.void_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            true,
        );
        let sprintf_fn = self
            .module
            .add_function("sprintf", sprintf_type, Some(Linkage::External));
        let _ = self
            .built_functions
            .insert("sprintf".to_string(), sprintf_fn);

        sprintf_fn
    }

    fn __c_printf(&mut self) -> FunctionValue<'ctx> {
        if let Some(function_value) = self.built_functions.get("printf") {
            return *function_value;
        }

        let printf_type = self.context.i32_type().fn_type(
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

    fn __c_strcmp(&mut self) -> Self::Function {
        if let Some(function_value) = self.built_functions.get("strcmp") {
            return *function_value;
        }

        let strcmp_type = self.context.i32_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        );
        let strcmp_fn = self
            .module
            .add_function("strcmp", strcmp_type, Some(Linkage::External));
        let _ = self.built_functions.insert("strcmp".to_string(), strcmp_fn);

        strcmp_fn
    }

    fn __c_strlen(&mut self) -> Self::Function {
        if let Some(function_value) = self.built_functions.get("strlen") {
            return *function_value;
        }

        let strlen_type = self.context.i64_type().fn_type(
            &[self.context.ptr_type(AddressSpace::default()).into()],
            false,
        );
        let strlen_fn = self
            .module
            .add_function("strlen", strlen_type, Some(Linkage::External));
        let _ = self.built_functions.insert("strlen".to_string(), strlen_fn);

        strlen_fn
    }

    fn __c_scanf(&mut self) -> Self::Function {
        if let Some(function_value) = self.built_functions.get("scanf") {
            return *function_value;
        }

        let scanf_type = self.context.i32_type().fn_type(
            &[self.context.ptr_type(AddressSpace::default()).into()],
            true,
        );
        let scanf_fn = self
            .module
            .add_function("scanf", scanf_type, Some(Linkage::External));
        let _ = self.built_functions.insert("scanf".to_string(), scanf_fn);

        scanf_fn
    }

    fn __c_sscanf(&mut self) -> Self::Function {
        if let Some(function_value) = self.built_functions.get("sscanf") {
            return *function_value;
        }

        let sscanf_type = self.context.i32_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            true,
        );
        let sscanf_fn = self
            .module
            .add_function("sscanf", sscanf_type, Some(Linkage::External));
        let _ = self.built_functions.insert("sscanf".to_string(), sscanf_fn);

        sscanf_fn
    }

    fn __c_malloc(&mut self) -> Self::Function {
        if let Some(function_value) = self.built_functions.get("malloc") {
            return *function_value;
        }

        let malloc_type = self
            .context
            .ptr_type(AddressSpace::default())
            .fn_type(&[self.context.i64_type().into()], false);
        let malloc_fn = self
            .module
            .add_function("malloc", malloc_type, Some(Linkage::External));
        let _ = self.built_functions.insert("malloc".to_string(), malloc_fn);

        malloc_fn
    }

    fn __c_realloc(&mut self) -> Self::Function {
        if let Some(function_value) = self.built_functions.get("realloc") {
            return *function_value;
        }

        let realloc_type = self.context.ptr_type(AddressSpace::default()).fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.i64_type().into(),
            ],
            false,
        );
        let realloc_fn = self
            .module
            .add_function("realloc", realloc_type, Some(Linkage::External));
        let _ = self
            .built_functions
            .insert("realloc".to_string(), realloc_fn);

        realloc_fn
    }

    fn __c_free(&mut self) -> Self::Function {
        if let Some(function_value) = self.built_functions.get("free") {
            return *function_value;
        }

        let free_type = self.context.void_type().fn_type(
            &[self.context.ptr_type(AddressSpace::default()).into()],
            false,
        );
        let free_fn = self
            .module
            .add_function("free", free_type, Some(Linkage::External));
        let _ = self.built_functions.insert("free".to_string(), free_fn);

        free_fn
    }

    fn __c_fopen(&mut self) -> Self::Function {
        const FN_NAME: &str = "fopen";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.ptr_type(AddressSpace::default()).fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_fwrite(&mut self) -> Self::Function {
        const FN_NAME: &str = "fwrite";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.void_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.i64_type().into(),
                self.context.i64_type().into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_fclose(&mut self) -> Self::Function {
        const FN_NAME: &str = "fclose";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.void_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_fprintf(&mut self) -> Self::Function {
        const FN_NAME: &str = "fprintf";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.void_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            true
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_fgetc(&mut self) -> Self::Function {
        const FN_NAME: &str = "fgetc";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.i32_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_rewind(&mut self) -> Self::Function {
        const FN_NAME: &str = "rewind";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.void_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_fseek(&mut self) -> Self::Function {
        const FN_NAME: &str = "fseek";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.i32_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
                self.context.i64_type().into(),
                self.context.i32_type().into()
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_ftell(&mut self) -> Self::Function {
        const FN_NAME: &str = "ftell";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.i32_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_feof(&mut self) -> Self::Function {
         const FN_NAME: &str = "feof";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.i32_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }

    fn __c_fsetpos(&mut self) -> Self::Function {
        const FN_NAME: &str = "fsetpos";

        if let Some(function_value) = self.built_functions.get(FN_NAME) {
            return *function_value;
        }

        let fn_type = self.context.i32_type().fn_type(
            &[
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false
        );
        let fn_obj = self
            .module
            .add_function(FN_NAME, fn_type, Some(Linkage::External));
        let _ = self.built_functions.insert(FN_NAME.to_string(), fn_obj);

        fn_obj
    }
}
