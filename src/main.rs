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

    Integer(isize),
    Name(String),

    Call,

    Plus,
    Minus,
    Asterisk,
    Slash,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub location: SourceLocation,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct Lexer {
    source: Vec<char>,
    location: SourceLocation,
}

lazy_static::lazy_static! {
    static ref LEXER_SINGLE_CHARS: HashMap<char, TokenKind> =
        HashMap::from_iter(IntoIter::new([
            ('+', TokenKind::Plus),
            ('-', TokenKind::Minus),
            ('*', TokenKind::Asterisk),
            ('/', TokenKind::Slash),
        ]));

    static ref LEXER_KEYWORDS: HashMap<&'static str, TokenKind> =
        HashMap::from_iter(IntoIter::new([
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

    pub fn next_token(self: &mut Lexer) -> Result<Token, Error> {
        loop {
            let start_location = self.location.clone();
            return match self.peek_char() {
                '\0' => Ok(Token {
                    kind: TokenKind::EndOfFile,
                    location: start_location.clone(),
                    length: self.location.position - start_location.position,
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
                        kind: TokenKind::Integer(int_value),
                        location: start_location.clone(),
                        length: self.location.position - start_location.position,
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
                        })
                    } else {
                        Ok(Token {
                            kind: TokenKind::Name(name),
                            location: start_location.clone(),
                            length: self.location.position - start_location.position,
                        })
                    }
                }

                _ => {
                    let chr = self.next_char();
                    if LEXER_SINGLE_CHARS.contains_key(&chr) {
                        Ok(Token {
                            kind: LEXER_SINGLE_CHARS[&chr].clone(),
                            location: start_location.clone(),
                            length: self.location.position - start_location.position,
                        })
                    } else {
                        Err(Error {
                            location: start_location,
                            message: format!("Unknown character '{}'", chr),
                        })
                    }
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
    Exit,
    PushInteger { value: isize },
    PushFunctionPointer { value: usize },
    AddInteger,
    SubtractInteger,
    MultiplyInteger,
    DivideInteger,
    Call,
    Return,
    Jump { location: usize },
    PrintInt,
}

pub fn compile_ops(
    lexer: &mut Lexer,
    ops: &mut Vec<Op>,
    functions: &mut HashMap<String, usize>,
) -> Result<(), Error> {
    loop {
        let token = lexer.next_token()?;
        match &token.kind {
            TokenKind::EndOfFile => break,

            TokenKind::Integer(integer) => ops.push(Op::PushInteger { value: *integer }),

            TokenKind::Name(name) => {
                if functions.contains_key(name) {
                    ops.push(Op::PushFunctionPointer {
                        value: functions[name],
                    })
                } else {
                    return Err(Error {
                        location: token.location.clone(),
                        message: format!("Unknown name '{}'", name),
                    });
                }
            }

            TokenKind::Call => ops.push(Op::Call),

            TokenKind::Plus => ops.push(Op::AddInteger),
            TokenKind::Minus => ops.push(Op::SubtractInteger),
            TokenKind::Asterisk => ops.push(Op::MultiplyInteger),
            TokenKind::Slash => ops.push(Op::DivideInteger),

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
    FunctionPointer(usize),
}

pub fn run_ops(ops: &[Op]) {
    let mut ip = 0;
    let mut stack = Vec::new();
    let mut return_stack = Vec::new();

    loop {
        match ops[ip] {
            Op::Exit => break,

            Op::PushInteger { value } => stack.push(Value::Integer(value)),

            Op::PushFunctionPointer { value } => stack.push(Value::FunctionPointer(value)),

            Op::AddInteger => {
                let b = match stack.pop().unwrap() {
                    Value::Integer(value) => value,
                    _ => unreachable!(),
                };
                let a = match stack.pop().unwrap() {
                    Value::Integer(value) => value,
                    _ => unreachable!(),
                };
                stack.push(Value::Integer(a + b));
            }

            Op::SubtractInteger => {
                let b = match stack.pop().unwrap() {
                    Value::Integer(value) => value,
                    _ => unreachable!(),
                };
                let a = match stack.pop().unwrap() {
                    Value::Integer(value) => value,
                    _ => unreachable!(),
                };
                stack.push(Value::Integer(a - b));
            }

            Op::MultiplyInteger => {
                let b = match stack.pop().unwrap() {
                    Value::Integer(value) => value,
                    _ => unreachable!(),
                };
                let a = match stack.pop().unwrap() {
                    Value::Integer(value) => value,
                    _ => unreachable!(),
                };
                stack.push(Value::Integer(a * b));
            }

            Op::DivideInteger => {
                let b = match stack.pop().unwrap() {
                    Value::Integer(value) => value,
                    _ => unreachable!(),
                };
                let a = match stack.pop().unwrap() {
                    Value::Integer(value) => value,
                    _ => unreachable!(),
                };
                stack.push(Value::Integer(a / b));
            }

            Op::Call => {
                let location = stack.pop().unwrap();
                return_stack.push(ip + 1);
                match location {
                    Value::FunctionPointer(location) => ip = location,
                    _ => unreachable!(),
                }
                continue;
            }

            Op::Return => {
                ip = return_stack.pop().unwrap();
                continue;
            }

            Op::Jump { location } => {
                ip = location;
                continue;
            }

            Op::PrintInt => {
                let value = stack.pop().unwrap();
                match value {
                    Value::Integer(value) => println!("{:?}", value),
                    _ => unreachable!(),
                }
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
    ops.push(Op::Jump { location: 0 });

    let print_int_location = ops.len();
    ops.push(Op::PrintInt);
    ops.push(Op::Return);

    {
        let ops_len = ops.len();
        if let Op::Jump { location } = &mut ops[jump_op_location] {
            *location = ops_len
        } else {
            unreachable!()
        }
    }

    let mut functions = HashMap::<String, usize>::from_iter(IntoIter::new([(
        "print_int".to_string(),
        print_int_location,
    )]));

    compile_ops(&mut lexer, &mut ops, &mut functions).unwrap_or_else(|error| {
        eprintln!(
            "{}:{}:{}: {}",
            error.location.filepath, error.location.line, error.location.column, error.message
        );
        exit(1)
    });

    ops.push(Op::Exit);

    /*
    for (index, op) in ops.iter().enumerate() {
        println!("{} = {:?}", index, op);
    }
    */

    run_ops(&ops);
}
