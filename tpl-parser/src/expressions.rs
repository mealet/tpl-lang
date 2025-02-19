// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

// NOTE: `line` field added for error handling on IR stage

use crate::{statements::Statements, value::Value};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum Expressions {
    Binary {
        operand: String,
        lhs: Box<Expressions>,
        rhs: Box<Expressions>,
        line: usize,
    },
    Boolean {
        operand: String,
        lhs: Box<Expressions>,
        rhs: Box<Expressions>,
        line: usize,
    },
    Bitwise {
        operand: String,
        lhs: Box<Expressions>,
        rhs: Box<Expressions>,
        line: usize,
    },

    Argument {
        name: String,
        datatype: String,
    },
    SubElement {
        parent: Box<Expressions>,
        child: Box<Expressions>,
        line: usize,
    },

    Call {
        function_name: String,
        arguments: Vec<Expressions>,
        line: usize,
    },
    Lambda {
        arguments: Vec<(String, String)>,
        statements: Vec<Statements>,
        ftype: String,
        line: usize,
    },

    Reference {
        object: Box<Expressions>,
        line: usize,
    },
    Dereference {
        object: Box<Expressions>,
        line: usize,
    },

    Array {
        values: Vec<Expressions>,
        len: usize,
        line: usize,
    },
    Slice {
        object: Box<Expressions>,
        index: Box<Expressions>,
        line: usize,
    },

    Value(Value),
    None,
}
