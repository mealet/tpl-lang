use crate::Compiler;
use inkwell::{module::Linkage, values::FunctionValue, AddressSpace};

pub trait Libc {
    type Function;

    fn __c_printf(&mut self) -> Self::Function;
    fn __c_sprintf(&mut self) -> Self::Function;
    fn __c_strcat(&mut self) -> Self::Function;
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
