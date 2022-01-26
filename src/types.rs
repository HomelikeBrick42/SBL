#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
    Bool,
    Procedure {
        parameters: Vec<Type>,
        return_types: Vec<Type>,
    },
}
