use crate::value::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum Expressions {
    Binary {
        operand: String,
        lhs: Box<Expressions>,
        rhs: Box<Expressions>,
    },
    Unary {
        operand: String,
        value: Value,
    },
    Value(Value),
    None,
}
