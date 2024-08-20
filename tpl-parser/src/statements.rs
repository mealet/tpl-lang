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
    // Assign
    AssignStatement {
        identifier: String,
        value: Option<Box<Expressions>>,
        line: usize,
    },
    BinaryAssignStatement {
        identifier: String,
        operand: String,
        value: Option<Box<Expressions>>,
        line: usize,
    },

    // Annotation
    AnnotationStatement {
        identifier: String,
        datatype: String,
        value: Option<Box<Expressions>>,
        line: usize,
    },

    // Functions
    FunctionCallStatement {
        function_name: String,
        arguments: Vec<Expressions>,
        line: usize,
    },

    // Constructions
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
    BreakStatement {
        line: usize,
    },

    // Etc.
    Expression(Expressions),
    None,
    End,
}
