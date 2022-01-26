#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Type,
    Integer,
    Bool,
    Procedure {
        parameters: Vec<Type>,
        return_types: Vec<Type>,
    },
}
