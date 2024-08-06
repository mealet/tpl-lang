use crate::value::Value;

// Avaible unary expressions:
//      Unary Minus (-x)
//      Unary Plus (+x)
//      Increment(++)
//      Decrement(--)
//      Logical NOT (!x)

// Avaible binary expressions:
//      Addition (+)
//      Substraction (-)
//      Multiplication (*)
//      Division (/)

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
