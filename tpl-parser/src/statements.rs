// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

use crate::expressions::Expressions;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum Statements {
    AssignStatement {
        identifier: String,
        value: Option<Box<Expressions>>,
    },
    AnnotationStatement {
        identifier: String,
        datatype: String,
        value: Option<Box<Expressions>>,
    },
    FunctionCallStatement {
        function_name: String,
        arguments: Vec<Expressions>,
    },
    IfStatement {
        condition: Expressions,
        then_block: Vec<Statements>,
        else_block: Option<Vec<Statements>>,
    },
    WhileStatement {
        condition: Expressions,
        block: Vec<Statements>,
    },
    ForStatement {
        varname: String,
        iterable_object: Expressions,
        block: Vec<Statements>,
    },
    Expression(Expressions),
    None,
    End,
}
