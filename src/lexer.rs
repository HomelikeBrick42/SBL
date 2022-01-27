use std::{array::IntoIter, collections::HashMap};

use crate::{
    common::{Error, SourceLocation},
    token::{Token, TokenData, TokenKind},
    tokenizer::Tokenizer,
};

#[derive(Debug, Clone)]
pub struct Lexer {
    source: Vec<char>,
    location: SourceLocation,
}

lazy_static::lazy_static! {
    static ref LEXER_SINGLE_CHARS: HashMap<char, TokenKind> =
        HashMap::from_iter(IntoIter::new([
            ('(', TokenKind::OpenParenthesis),
            (')', TokenKind::CloseParenthesis),
            ('{', TokenKind::OpenBrace),
            ('}', TokenKind::CloseBrace),

            ('!', TokenKind::Not),

            ('+', TokenKind::Plus),
            ('-', TokenKind::Minus),
            ('*', TokenKind::Asterisk),
            ('/', TokenKind::Slash),

            ('=', TokenKind::Equal),

            ('<', TokenKind::LessThan),
            ('>', TokenKind::GreaterThan),
        ]));

    static ref LEXER_DOUBLE_CHARS: HashMap<char, HashMap<char, TokenKind>> =
        HashMap::from_iter(IntoIter::new([
            ('=',  HashMap::from_iter(IntoIter::new([('=', TokenKind::EqualEqual)]))),

            ('<',  HashMap::from_iter(IntoIter::new([('=', TokenKind::LessThanEqual)]))),
            ('>',  HashMap::from_iter(IntoIter::new([('=', TokenKind::GreaterThanEqual)]))),

            ('-',  HashMap::from_iter(IntoIter::new([('>', TokenKind::RightArrow)]))),

            ('!',  HashMap::from_iter(IntoIter::new([('=', TokenKind::NotEqual)]))),
        ]));

    static ref LEXER_KEYWORDS: HashMap<&'static str, TokenKind> =
        HashMap::from_iter(IntoIter::new([
            ("print", TokenKind::Print),

            ("if", TokenKind::If),
            ("else", TokenKind::Else),

            ("while", TokenKind::While),

            ("const", TokenKind::Const),

            ("dup", TokenKind::Dup),
            ("drop", TokenKind::Drop),
            ("swap", TokenKind::Swap),

            ("memory", TokenKind::Memory),

            ("proc", TokenKind::Proc),
            ("call", TokenKind::Call),
        ]));
}

impl Lexer {
    pub fn new(filepath: String, source: &str) -> Lexer {
        Lexer {
            source: source.chars().into_iter().collect(),
            location: SourceLocation {
                filepath,
                position: 0,
                line: 1,
                column: 1,
            },
        }
    }

    fn peek_char(self: &Lexer) -> char {
        if self.location.position < self.source.len() {
            self.source[self.location.position]
        } else {
            '\0'
        }
    }

    fn next_char(self: &mut Lexer) -> char {
        let chr = self.peek_char();
        self.location.position += 1;
        self.location.column += 1;
        if chr == '\n' {
            self.location.line += 1;
            self.location.column = 1;
        }
        return chr;
    }
}

impl Tokenizer for Lexer {
    fn next_token(self: &mut Self) -> Result<Token, Error> {
        loop {
            let start_location = self.location.clone();
            return match self.peek_char() {
                '\0' => Ok(Token {
                    kind: TokenKind::EndOfFile,
                    location: start_location.clone(),
                    length: self.location.position - start_location.position,
                    data: TokenData::None,
                }),

                ' ' | '\t' | '\n' | '\r' => {
                    self.next_char();
                    continue;
                }

                '0'..='9' => {
                    let base = if self.peek_char() == '0' {
                        self.next_char();
                        match self.peek_char() {
                            'b' => 2,
                            'o' => 8,
                            'd' => 10,
                            'x' => 16,
                            _ => 10,
                        }
                    } else {
                        10
                    };

                    let mut int_value = 0;

                    loop {
                        let chr = self.peek_char();
                        match chr {
                            '0'..='9' | 'A'..='Z' | 'a'..='z' => {
                                let value = match chr {
                                    '0'..='9' => chr as isize - '0' as isize,
                                    'A'..='Z' => chr as isize - 'A' as isize + 10,
                                    'a'..='z' => chr as isize - 'a' as isize + 10,
                                    _ => unreachable!(),
                                };

                                if value >= base {
                                    return Err(Error {
                                        location: self.location.clone(),
                                        message: format!(
                                            "Digit '{}' is too big for base {}",
                                            chr, base
                                        ),
                                    });
                                }

                                int_value *= base;
                                int_value += value;

                                self.next_char();
                            }

                            '_' => {
                                self.next_char();
                                continue;
                            }

                            _ => break,
                        }
                    }

                    Ok(Token {
                        kind: TokenKind::Integer,
                        location: start_location.clone(),
                        length: self.location.position - start_location.position,
                        data: TokenData::Integer(int_value),
                    })
                }

                'A'..='Z' | 'a'..='z' | '_' => {
                    let mut name = String::new();
                    loop {
                        match self.peek_char() {
                            'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => name.push(self.next_char()),
                            _ => break,
                        }
                    }
                    if LEXER_KEYWORDS.contains_key(&name as &str) {
                        Ok(Token {
                            kind: LEXER_KEYWORDS[&name as &str].clone(),
                            location: start_location.clone(),
                            length: self.location.position - start_location.position,
                            data: TokenData::None,
                        })
                    } else {
                        Ok(Token {
                            kind: TokenKind::Name,
                            location: start_location.clone(),
                            length: self.location.position - start_location.position,
                            data: TokenData::String(name),
                        })
                    }
                }

                _ => {
                    let chr = self.next_char();

                    if LEXER_DOUBLE_CHARS.contains_key(&chr) {
                        if LEXER_DOUBLE_CHARS[&chr].contains_key(&self.peek_char()) {
                            let chr2 = self.next_char();
                            return Ok(Token {
                                kind: LEXER_DOUBLE_CHARS[&chr][&chr2].clone(),
                                location: start_location.clone(),
                                length: self.location.position - start_location.position,
                                data: TokenData::None,
                            });
                        }
                    }

                    if LEXER_SINGLE_CHARS.contains_key(&chr) {
                        return Ok(Token {
                            kind: LEXER_SINGLE_CHARS[&chr].clone(),
                            location: start_location.clone(),
                            length: self.location.position - start_location.position,
                            data: TokenData::None,
                        });
                    }

                    Err(Error {
                        location: start_location,
                        message: format!("Unknown character '{}'", chr),
                    })
                }
            };
        }
    }

    fn peek_token(self: &Self) -> Result<Token, Error> {
        let mut lexer = self.clone();
        return lexer.next_token();
    }

    fn peek_kind(self: &Self) -> Result<TokenKind, Error> {
        Ok(self.peek_token()?.kind)
    }
}
