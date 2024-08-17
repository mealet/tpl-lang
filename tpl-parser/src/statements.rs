// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

// NOTE: `line` field added for error handling on IR stage

use crate::expressions::Expressions;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum Statements {
    AssignStatement {
        identifier: String,
        value: Option<Box<Expressions>>,
        line: usize,
    },
    AnnotationStatement {
        identifier: String,
        datatype: String,
        value: Option<Box<Expressions>>,
        line: usize,
    },
    FunctionCallStatement {
        function_name: String,
        arguments: Vec<Expressions>,
        line: usize,
    },
    IfStatement {
        condition: Expressions,
        then_block: Vec<Statements>,
        else_block: Option<Vec<Statements>>,
        line: usize,
    },
    WhileStatement {
        condition: Expressions,
        block: Vec<Statements>,
        line: usize,
    },
    ForStatement {
        varname: String,
        iterable_object: Expressions,
        block: Vec<Statements>,
        line: usize,
    },
    Expression(Expressions),
    None,
    End,
}
