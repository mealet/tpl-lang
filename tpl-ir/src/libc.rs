pub trait Libc {
    type Function;

    fn __c_printf(&mut self) -> Self::Function;
    fn __c_strcat(&mut self) -> Self::Function;
}
