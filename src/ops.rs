use std::collections::HashMap;

use crate::common::*;
use crate::lexer::*;

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
