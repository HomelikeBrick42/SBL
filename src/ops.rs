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
    LessThanInteger {
        location: SourceLocation,
    },
    GreaterThanInteger {
        location: SourceLocation,
    },
    LessThanEqualInteger {
        location: SourceLocation,
    },
    GreaterThanEqualInteger {
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
    Dup {
        location: SourceLocation,
    },
    Drop {
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
    Print {
        location: SourceLocation,
    },
}

impl Op {
    pub fn get_location(self: &Op) -> SourceLocation {
        match self {
            Op::Exit { location } => location.clone(),
            Op::PushInteger { location, value: _ } => location.clone(),
            Op::AddInteger { location } => location.clone(),
            Op::SubtractInteger { location } => location.clone(),
            Op::MultiplyInteger { location } => location.clone(),
            Op::DivideInteger { location } => location.clone(),
            Op::LessThanInteger { location } => location.clone(),
            Op::GreaterThanInteger { location } => location.clone(),
            Op::LessThanEqualInteger { location } => location.clone(),
            Op::GreaterThanEqualInteger { location } => location.clone(),
            Op::Equal { location } => location.clone(),
            Op::NotEqual { location } => location.clone(),
            Op::Not { location } => location.clone(),
            Op::Dup { location } => location.clone(),
            Op::Drop { location } => location.clone(),
            Op::Jump {
                location,
                position: _,
            } => location.clone(),
            Op::ConditonalJump {
                location,
                position: _,
            } => location.clone(),
            Op::Print { location } => location.clone(),
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

enum BlockType {
    If {
        position: usize,
    },
    Else {
        skip_position: usize,
    },
    While {
        position: usize,
    },
    WhileBody {
        begin_position: usize,
        end_jump_position: usize,
    },
}

pub fn compile_ops(lexer: &mut Lexer, ops: &mut Vec<Op>) -> Result<(), Error> {
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
                let _name = token.data.get_string();
                unimplemented!()
            }

            TokenKind::PrintInt => ops.push(Op::Print {
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
            }

            TokenKind::While => block_stack.push(BlockType::While {
                position: ops.len(),
            }),

            TokenKind::OpenBrace => {
                let sucess = if let Some(block) = block_stack.last() {
                    if let BlockType::While { position } = block {
                        let position = *position;
                        block_stack.pop().unwrap();
                        ops.push(Op::Not {
                            location: token.location.clone(),
                        });
                        block_stack.push(BlockType::WhileBody {
                            begin_position: position,
                            end_jump_position: ops.len(),
                        });
                        ops.push(Op::ConditonalJump {
                            location: token.location.clone(),
                            position: 0,
                        });
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !sucess {
                    // TODO: Move this to a function because its also used in the `_` arm
                    return Err(Error {
                        location: token.location.clone(),
                        message: format!("Unexpected token '{:?}'", token.kind),
                    });
                }
            }

            TokenKind::Dup => ops.push(Op::Dup {
                location: token.location,
            }),

            TokenKind::Drop => ops.push(Op::Drop {
                location: token.location,
            }),

            TokenKind::CloseBrace => {
                let block_type = block_stack.pop().unwrap();
                match &block_type {
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
                        }

                        *ops[*position].get_condtional_jump_location_mut() = ops.len();
                    }

                    BlockType::Else { skip_position } => {
                        *ops[*skip_position].get_jump_location_mut() = ops.len();
                    }

                    BlockType::While { position: _ } => {
                        // TODO: Move this to a function because its also used in the `_` arm
                        return Err(Error {
                            location: token.location.clone(),
                            message: format!("Unexpected token '{:?}'", token.kind),
                        });
                    }

                    BlockType::WhileBody {
                        begin_position,
                        end_jump_position,
                    } => {
                        ops.push(Op::Jump {
                            location: token.location,
                            position: *begin_position,
                        });

                        *ops[*end_jump_position].get_condtional_jump_location_mut() = ops.len();
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

            TokenKind::LessThan => ops.push(Op::LessThanInteger {
                location: token.location,
            }),

            TokenKind::GreaterThan => ops.push(Op::GreaterThanInteger {
                location: token.location,
            }),

            TokenKind::LessThanEqual => ops.push(Op::LessThanEqualInteger {
                location: token.location,
            }),

            TokenKind::GreaterThanEqual => ops.push(Op::GreaterThanEqualInteger {
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
