use std::{array::IntoIter, collections::HashMap, env::args, process::exit};

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub filepath: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub location: SourceLocation,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    EndOfFile,

    Integer,
    Name,

    Proc,
    Call,
    Return,
    If,
    Else,

    OpenBrace,
    CloseBrace,

    Not,

    Plus,
    Minus,
    Asterisk,
    Slash,

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

#[derive(Debug, Clone)]
pub struct Lexer {
    source: Vec<char>,
    location: SourceLocation,
}

lazy_static::lazy_static! {
    static ref LEXER_SINGLE_CHARS: HashMap<char, TokenKind> =
        HashMap::from_iter(IntoIter::new([
            ('{', TokenKind::OpenBrace),
            ('}', TokenKind::CloseBrace),

            ('!', TokenKind::Not),

            ('+', TokenKind::Plus),
            ('-', TokenKind::Minus),
            ('*', TokenKind::Asterisk),
            ('/', TokenKind::Slash),

            ('=', TokenKind::Equal),
        ]));

    static ref LEXER_DOUBLE_CHARS: HashMap<char, HashMap<char, TokenKind>> =
        HashMap::from_iter(IntoIter::new([
            ('!',  HashMap::from_iter(IntoIter::new([('=', TokenKind::NotEqual)]))),
        ]));

    static ref LEXER_KEYWORDS: HashMap<&'static str, TokenKind> =
        HashMap::from_iter(IntoIter::new([
            ("proc", TokenKind::Proc),
            ("call", TokenKind::Call),
            ("return", TokenKind::Return),
            ("if", TokenKind::If),
            ("else", TokenKind::Else),
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

    pub fn next_token(self: &mut Lexer) -> Result<Token, Error> {
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

    pub fn peek_token(self: &Lexer) -> Result<Token, Error> {
        let mut lexer = self.clone();
        return lexer.next_token();
    }

    pub fn peek_kind(self: &Lexer) -> Result<TokenKind, Error> {
        Ok(self.peek_token()?.kind)
    }

    pub fn expect_token(self: &mut Lexer, kind: TokenKind) -> Result<Token, Error> {
        let actual_kind = self.peek_kind()?;
        if actual_kind != kind {
            Err(Error {
                location: self.location.clone(),
                message: format!(
                    "Unexpected token '{:?}', expected '{:?}'",
                    actual_kind, kind
                ),
            })
        } else {
            self.next_token()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Exit {
        location: SourceLocation,
    },
    PushInteger {
        location: SourceLocation,
        value: isize,
    },
    PushFunctionPointer {
        location: SourceLocation,
        value: usize,
    },
    AddInteger {
        location: SourceLocation,
    },
    SubtractInteger {
        location: SourceLocation,
    },
    MultiplyInteger {
        location: SourceLocation,
    },
    DivideInteger {
        location: SourceLocation,
    },
    Equal {
        location: SourceLocation,
    },
    NotEqual {
        location: SourceLocation,
    },
    Not {
        location: SourceLocation,
    },
    Call {
        location: SourceLocation,
    },
    Return {
        location: SourceLocation,
    },
    Jump {
        location: SourceLocation,
        position: usize,
    },
    ConditonalJump {
        location: SourceLocation,
        position: usize,
    },
    PrintInt {
        location: SourceLocation,
    },
}

impl Op {
    pub fn get_location(self: &Op) -> SourceLocation {
        match self {
            Op::Exit { location } => location.clone(),
            Op::PushInteger { location, value: _ } => location.clone(),
            Op::PushFunctionPointer { location, value: _ } => location.clone(),
            Op::AddInteger { location } => location.clone(),
            Op::SubtractInteger { location } => location.clone(),
            Op::MultiplyInteger { location } => location.clone(),
            Op::DivideInteger { location } => location.clone(),
            Op::Equal { location } => location.clone(),
            Op::NotEqual { location } => location.clone(),
            Op::Not { location } => location.clone(),
            Op::Call { location } => location.clone(),
            Op::Return { location } => location.clone(),
            Op::Jump {
                location,
                position: _,
            } => location.clone(),
            Op::ConditonalJump {
                location,
                position: _,
            } => location.clone(),
            Op::PrintInt { location } => location.clone(),
        }
    }

    pub fn get_push_integer_value(self: &Op) -> isize {
        if let Op::PushInteger { location: _, value } = self {
            *value
        } else {
            unreachable!()
        }
    }

    pub fn get_push_integer_value_mut(self: &mut Op) -> &mut isize {
        if let Op::PushInteger { location: _, value } = self {
            value
        } else {
            unreachable!()
        }
    }

    pub fn get_push_function_pointer_value(self: &Op) -> usize {
        if let Op::PushFunctionPointer { location: _, value } = self {
            *value
        } else {
            unreachable!()
        }
    }

    pub fn get_push_function_pointer_value_mut(self: &mut Op) -> &mut usize {
        if let Op::PushFunctionPointer { location: _, value } = self {
            value
        } else {
            unreachable!()
        }
    }

    pub fn get_jump_location(self: &Op) -> usize {
        if let Op::Jump {
            location: _,
            position,
        } = self
        {
            *position
        } else {
            unreachable!()
        }
    }

    pub fn get_jump_location_mut(self: &mut Op) -> &mut usize {
        if let Op::Jump {
            location: _,
            position,
        } = self
        {
            position
        } else {
            unreachable!()
        }
    }

    pub fn get_condtional_jump_location(self: &Op) -> usize {
        if let Op::ConditonalJump {
            location: _,
            position,
        } = self
        {
            *position
        } else {
            unreachable!()
        }
    }

    pub fn get_condtional_jump_location_mut(self: &mut Op) -> &mut usize {
        if let Op::ConditonalJump {
            location: _,
            position,
        } = self
        {
            position
        } else {
            unreachable!()
        }
    }
}

pub enum BlockType {
    Function { position: usize },
    If { position: usize },
    Else { skip_position: usize },
}

pub fn compile_ops(
    lexer: &mut Lexer,
    ops: &mut Vec<Op>,
    functions: &mut Vec<HashMap<String, usize>>,
) -> Result<(), Error> {
    let mut block_stack = Vec::new();

    fn get_function_location(
        functions: &mut Vec<HashMap<String, usize>>,
        name: &String,
        location: SourceLocation,
    ) -> Result<usize, Error> {
        for scope in functions.iter().rev() {
            if scope.contains_key(name) {
                return Ok(scope[name]);
            }
        }
        Err(Error {
            location: location,
            message: format!("Unknown name '{}'", name),
        })
    }

    loop {
        let token = lexer.next_token()?;
        match &token.kind {
            TokenKind::EndOfFile => break,

            TokenKind::Integer => {
                let integer = token.data.get_integer();
                ops.push(Op::PushInteger {
                    location: token.location,
                    value: integer,
                })
            }

            TokenKind::Name => {
                let name = token.data.get_string();
                ops.push(Op::PushFunctionPointer {
                    location: token.location.clone(),
                    value: get_function_location(functions, &name, token.location)?,
                });
            }

            TokenKind::Proc => {
                let name_token = lexer.expect_token(TokenKind::Name)?;
                let name = name_token.data.get_string();

                block_stack.push(BlockType::Function {
                    position: ops.len(),
                });
                ops.push(Op::Jump {
                    location: token.location,
                    position: 0,
                });

                let scope = functions.last_mut().unwrap();
                scope.insert(name, ops.len());

                lexer.expect_token(TokenKind::OpenBrace)?;
                functions.push(HashMap::new());
            }

            TokenKind::Call => ops.push(Op::Call {
                location: token.location,
            }),

            TokenKind::Return => ops.push(Op::Return {
                location: token.location,
            }),

            TokenKind::If => {
                ops.push(Op::Not {
                    location: token.location.clone(),
                });
                block_stack.push(BlockType::If {
                    position: ops.len(),
                });
                ops.push(Op::ConditonalJump {
                    location: token.location,
                    position: 0,
                });
                lexer.expect_token(TokenKind::OpenBrace)?;
                functions.push(HashMap::new());
            }

            TokenKind::CloseBrace => {
                let block_type = block_stack.pop().unwrap();
                functions.pop().unwrap();
                match &block_type {
                    BlockType::Function { position } => {
                        ops.push(Op::Return {
                            location: token.location,
                        });
                        *ops[*position].get_jump_location_mut() = ops.len();
                    }

                    BlockType::If { position } => {
                        if lexer.peek_kind()? == TokenKind::Else {
                            let else_token = lexer.next_token()?;
                            block_stack.push(BlockType::Else {
                                skip_position: ops.len(),
                            });
                            ops.push(Op::Jump {
                                location: else_token.location.clone(),
                                position: 0,
                            });
                            lexer.expect_token(TokenKind::OpenBrace)?;
                            functions.push(HashMap::new());
                        }

                        *ops[*position].get_condtional_jump_location_mut() = ops.len();
                    }

                    BlockType::Else { skip_position } => {
                        *ops[*skip_position].get_jump_location_mut() = ops.len();
                    }
                }
            }

            TokenKind::Plus => ops.push(Op::AddInteger {
                location: token.location,
            }),

            TokenKind::Minus => ops.push(Op::SubtractInteger {
                location: token.location,
            }),

            TokenKind::Asterisk => ops.push(Op::MultiplyInteger {
                location: token.location,
            }),

            TokenKind::Slash => ops.push(Op::DivideInteger {
                location: token.location,
            }),

            TokenKind::Equal => ops.push(Op::Equal {
                location: token.location,
            }),

            TokenKind::NotEqual => ops.push(Op::NotEqual {
                location: token.location,
            }),

            TokenKind::Not => ops.push(Op::Not {
                location: token.location,
            }),

            _ => {
                return Err(Error {
                    location: token.location.clone(),
                    message: format!("Unexpected token '{:?}'", token.kind),
                });
            }
        };
    }
    Ok(())
}

pub enum Value {
    Integer(isize),
    Bool(bool),
    FunctionPointer(usize),
}

impl Value {
    pub fn integer(self: Value) -> isize {
        if let Value::Integer(value) = self {
            value
        } else {
            unreachable!()
        }
    }

    pub fn bool(self: Value) -> bool {
        if let Value::Bool(value) = self {
            value
        } else {
            unreachable!()
        }
    }

    pub fn function_pointer(self: Value) -> usize {
        if let Value::FunctionPointer(value) = self {
            value
        } else {
            unreachable!()
        }
    }
}

pub fn run_ops(ops: &[Op]) {
    let mut ip = 0;
    let mut stack = Vec::new();
    let mut return_stack = Vec::new();

    loop {
        match &ops[ip] {
            Op::Exit { location: _ } => break,

            Op::PushInteger { location: _, value } => stack.push(Value::Integer(*value)),

            Op::PushFunctionPointer { location: _, value } => {
                stack.push(Value::FunctionPointer(*value))
            }

            Op::AddInteger { location: _ } => {
                let b = stack.pop().unwrap().integer();
                let a = stack.pop().unwrap().integer();
                stack.push(Value::Integer(a + b));
            }

            Op::SubtractInteger { location: _ } => {
                let b = stack.pop().unwrap().integer();
                let a = stack.pop().unwrap().integer();
                stack.push(Value::Integer(a - b));
            }

            Op::MultiplyInteger { location: _ } => {
                let b = stack.pop().unwrap().integer();
                let a = stack.pop().unwrap().integer();
                stack.push(Value::Integer(a * b));
            }

            Op::DivideInteger { location: _ } => {
                let b = stack.pop().unwrap().integer();
                let a = stack.pop().unwrap().integer();
                stack.push(Value::Integer(a / b));
            }

            Op::Equal { location: _ } => {
                let value = match stack.pop().unwrap() {
                    Value::Integer(value) => value == stack.pop().unwrap().integer(),
                    Value::Bool(value) => value == stack.pop().unwrap().bool(),
                    Value::FunctionPointer(value) => {
                        value == stack.pop().unwrap().function_pointer()
                    }
                };
                stack.push(Value::Bool(value));
            }

            Op::NotEqual { location: _ } => {
                let value = match stack.pop().unwrap() {
                    Value::Integer(value) => value != stack.pop().unwrap().integer(),
                    Value::Bool(value) => value != stack.pop().unwrap().bool(),
                    Value::FunctionPointer(value) => {
                        value != stack.pop().unwrap().function_pointer()
                    }
                };
                stack.push(Value::Bool(value));
            }

            Op::Not { location: _ } => {
                let value = stack.pop().unwrap().bool();
                stack.push(Value::Bool(!value));
            }

            Op::Call { location: _ } => {
                let location = stack.pop().unwrap().function_pointer();
                return_stack.push(ip + 1);
                ip = location;
                continue;
            }

            Op::Return { location: _ } => {
                ip = return_stack.pop().unwrap();
                continue;
            }

            Op::Jump {
                location: _,
                position,
            } => {
                ip = *position;
                continue;
            }

            Op::ConditonalJump {
                location: _,
                position,
            } => {
                let condition = stack.pop().unwrap().bool();
                if condition {
                    ip = *position;
                    continue;
                }
            }

            Op::PrintInt { location: _ } => {
                let value = stack.pop().unwrap().integer();
                println!("{:?}", value);
            }
        }
        ip += 1;
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        exit(1)
    }

    let filepath = &args[1];

    let source = std::fs::read_to_string(filepath).unwrap_or_else(|_| {
        eprintln!("Unable to open file '{}'", filepath);
        exit(1)
    });

    let mut lexer = Lexer::new(filepath.clone(), &source as &str);

    /*
    loop {
        let token = lexer.next_token().unwrap_or_else(|error| {
            eprintln!(
                "{}:{}:{}: {}",
                error.location.filepath, error.location.line, error.location.column, error.message
            );
            exit(1)
        });
        println!("{:?}", token);
        if let TokenKind::EndOfFile = token.kind {
            break;
        }
    }
    */

    let mut ops = Vec::new();

    let jump_op_location = ops.len();
    ops.push(Op::Jump {
        location: SourceLocation {
            filepath: "buitin.sbl".to_string(),
            position: 0,
            line: 1,
            column: 1,
        },
        position: 0,
    });

    let print_int_location = ops.len();
    ops.push(Op::PrintInt {
        location: SourceLocation {
            filepath: "buitin.sbl".to_string(),
            position: 0,
            line: 1,
            column: 1,
        },
    });
    ops.push(Op::Return {
        location: SourceLocation {
            filepath: "buitin.sbl".to_string(),
            position: 0,
            line: 1,
            column: 1,
        },
    });

    *ops[jump_op_location].get_jump_location_mut() = ops.len();

    let mut functions = Vec::from([HashMap::<String, usize>::from_iter(IntoIter::new([(
        "print_int".to_string(),
        print_int_location,
    )]))]);

    compile_ops(&mut lexer, &mut ops, &mut functions).unwrap_or_else(|error| {
        eprintln!(
            "{}:{}:{}: {}",
            error.location.filepath, error.location.line, error.location.column, error.message
        );
        exit(1)
    });

    ops.push(Op::Exit {
        location: lexer
            .expect_token(TokenKind::EndOfFile)
            .unwrap_or_else(|error| {
                eprintln!(
                    "{}:{}:{}: {}",
                    error.location.filepath,
                    error.location.line,
                    error.location.column,
                    error.message
                );
                exit(1)
            })
            .location,
    });

    /*
    for (index, op) in ops.iter().enumerate() {
        println!("{} = {:?}", index, op);
    }
    */

    run_ops(&ops);
}
