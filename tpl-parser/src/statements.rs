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
        value: Option<Box<Expressions>>,
    },
    FunctionCallStatement {
        function_name: String,
        arguments: Vec<Expressions>,
    },
    Expression(Expressions),
    None,
    End,
}
