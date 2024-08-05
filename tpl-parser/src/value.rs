#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(unused)]
pub enum Value {
    Integer(i32),
    String(String),
    Boolean(bool),
    Identifier(String),
}
