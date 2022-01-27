use crate::common::SourceLocation;

#[derive(Debug, Clone, PartialEq)]
pub struct IR {
    pub location: SourceLocation,
    pub kind: IRKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IRKind {
    Exit,

    PushProc { id: usize },
    PushInt { value: isize },

    Add,
    Sub,
    Mul,
    Div,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,
    Equal,
    NotEqual,
    Not,

    Dup,
    Drop,
    Swap,

    Jump { relative_position: isize },
    JumpFalse { relative_position: isize },

    Call,
    Return,

    Print,
}
