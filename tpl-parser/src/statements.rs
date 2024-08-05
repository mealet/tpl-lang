use crate::{expressions::Expressions, value::Value};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum Statements {
    AssignStatement {
        identifier: String,
        value: Value,
    },
    AnnotationStatement {
        identifier: String,
        value: Option<Box<Statements>>,
    },
    FunctionCallStatement {
        function_name: String,
        arguments: Vec<Expressions>,
    },
    Expression(Expressions),
    None,
}
