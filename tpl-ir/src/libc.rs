use crate::Compiler;
use inkwell::{module::Linkage, values::FunctionValue, AddressSpace};

pub trait Libc {
    type Function;

    fn __c_printf(&mut self) -> Self::Function;
    fn __c_sprintf(&mut self) -> Self::Function;

    fn __c_strcat(&mut self) -> Self::Function;
    fn __c_strcmp(&mut self) -> Self::Function;
    fn __c_strlen(&mut self) -> Self::Function;

    fn __c_scanf(&mut self) -> Self::Function;
    fn __c_sscanf(&mut self) -> Self::Function;

    fn __c_malloc(&mut self) -> Self::Function;
    fn __c_realloc(&mut self) -> Self::Function;
    fn __c_free(&mut self) -> Self::Function;
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn __c_sprintf_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let sprintf = compiler.__c_sprintf();

        assert_eq!(sprintf.get_linkage(), Linkage::External);
        assert!(!sprintf.is_null());
        assert!(!sprintf.is_undef());
        assert!(sprintf.verify(true));
        assert_eq!(
            sprintf.get_name().to_string_lossy().to_string(),
            String::from("sprintf")
        );
        assert_eq!(
            sprintf.get_type(),
            compiler.context.void_type().fn_type(
                &[
                    compiler.context.ptr_type(AddressSpace::default()).into(),
                    compiler.context.ptr_type(AddressSpace::default()).into(),
                ],
                true
            )
        );
    }

    #[test]
    fn __c_strcmp_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let strcmp = compiler.__c_strcmp();

        assert_eq!(strcmp.get_linkage(), Linkage::External);
        assert!(!strcmp.is_null());
        assert!(!strcmp.is_undef());
        assert!(strcmp.verify(true));
        assert_eq!(
            strcmp.get_name().to_string_lossy().to_string(),
            String::from("strcmp")
        );
        assert_eq!(
            strcmp.get_type(),
            compiler.context.i32_type().fn_type(
                &[
                    compiler.context.ptr_type(AddressSpace::default()).into(),
                    compiler.context.ptr_type(AddressSpace::default()).into(),
                ],
                false
            )
        );
    }

    #[test]
    fn __c_scanf_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let scanf = compiler.__c_scanf();

        assert_eq!(scanf.get_linkage(), Linkage::External);
        assert!(!scanf.is_null());
        assert!(!scanf.is_undef());
        assert!(scanf.verify(true));
        assert_eq!(
            scanf.get_name().to_string_lossy().to_string(),
            String::from("scanf")
        );
        assert_eq!(
            scanf.get_type(),
            compiler.context.i32_type().fn_type(
                &[compiler.context.ptr_type(AddressSpace::default()).into(),],
                true
            )
        );
    }

    #[test]
    fn __c_sscanf_test() {
        let ctx = inkwell::context::Context::create();
        let mut compiler =
            Compiler::new(&ctx, "test", String::from("none"), String::from("test.tpl"));
        compiler.builder.position_at_end(compiler.current_block);

        let sscanf = compiler.__c_sscanf();

        assert_eq!(sscanf.get_linkage(), Linkage::External);
        assert!(!sscanf.is_null());
        assert!(!sscanf.is_undef());
        assert!(sscanf.verify(true));
        assert_eq!(
            sscanf.get_name().to_string_lossy().to_string(),
            String::from("sscanf")
        );
        assert_eq!(
            sscanf.get_type(),
            compiler.context.i32_type().fn_type(
                &[
                    compiler.context.ptr_type(AddressSpace::default()).into(),
                    compiler.context.ptr_type(AddressSpace::default()).into(),
                ],
                true
            )
        );
    }
}
