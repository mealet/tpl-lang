// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum Value {
    Integer(i32),
    String(String),
    Boolean(bool),
    Identifier(String),
}
