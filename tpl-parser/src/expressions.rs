// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

// NOTE: `line` field added for error handling on IR stage

use crate::value::Value;
use crate::statements::Statements;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum Expressions {
    Binary {
        operand: String,
        lhs: Box<Expressions>,
        rhs: Box<Expressions>,
        line: usize,
    },
    Unary {
        operand: String,
        value: Value,
        line: usize,
    },
    Argument {
        name: String,
        datatype: String,
    },
    Call {
        function_name: String,
        arguments: Vec<Expressions>,
        line: usize,
    },
    Lambda {
        arguments: Vec<Expressions>,
        statements: Vec<Statements>,
        line: usize
    },
    Value(Value),
    None,
}
