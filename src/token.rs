use crate::common::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    EndOfFile,

    Integer,
    Name,

    Print,
    If,
    Else,
    While,

    Proc,
    Call,

    Dup,
    Drop,
    Swap,

    Memory,

    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,

    Not,

    RightArrow,

    Plus,
    Minus,
    Asterisk,
    Slash,

    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,

    Equal,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenData {
    None,
    Integer(isize),
    String(String),
}

impl TokenData {
    pub fn get_integer(self: &TokenData) -> isize {
        if let TokenData::Integer(value) = self {
            *value
        } else {
            unreachable!()
        }
    }

    pub fn get_integer_mut(self: &mut TokenData) -> &mut isize {
        if let TokenData::Integer(value) = self {
            value
        } else {
            unreachable!()
        }
    }

    pub fn get_string(self: &TokenData) -> String {
        if let TokenData::String(value) = self {
            value.clone()
        } else {
            unreachable!()
        }
    }

    pub fn get_string_mut(self: &mut TokenData) -> &mut String {
        if let TokenData::String(value) = self {
            value
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub location: SourceLocation,
    pub length: usize,
    pub data: TokenData,
}
