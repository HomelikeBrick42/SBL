use crate::common::*;
use crate::execution::*;
use crate::tokenizer::*;
use crate::types::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Exit {
        location: SourceLocation,
    },
    PushFunctionPointer {
        location: SourceLocation,
        value: usize,
    },
    PushType {
        location: SourceLocation,
        value: Type,
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
    Swap {
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
    SkipProc {
        location: SourceLocation,
        position: usize,
        parameters: Vec<Type>,
        return_types: Vec<Type>,
    },
    Call {
        location: SourceLocation,
    },
    Return {
        location: SourceLocation,
    },
}

impl Op {
    pub fn get_location(self: &Op) -> SourceLocation {
        match self {
            Op::Exit { location } => location.clone(),
            Op::PushInteger { location, value: _ } => location.clone(),
            Op::PushType { location, value: _ } => location.clone(),
            Op::PushFunctionPointer { location, value: _ } => location.clone(),
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
            Op::Swap { location } => location.clone(),
            Op::Jump {
                location,
                position: _,
            } => location.clone(),
            Op::ConditonalJump {
                location,
                position: _,
            } => location.clone(),
            Op::Print { location } => location.clone(),
            Op::SkipProc {
                location,
                position: _,
                parameters: _,
                return_types: _,
            } => location.clone(),
            Op::Call { location } => location.clone(),
            Op::Return { location } => location.clone(),
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

    pub fn get_push_type_value(self: &Op) -> Type {
        if let Op::PushType { location: _, value } = self {
            value.clone()
        } else {
            unreachable!()
        }
    }

    pub fn get_push_type_value_mut(self: &mut Op) -> &mut Type {
        if let Op::PushType { location: _, value } = self {
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

    pub fn get_skip_procedure_jump_location(self: &Op) -> usize {
        if let Op::SkipProc {
            location: _,
            position,
            parameters: _,
            return_types: _,
        } = self
        {
            *position
        } else {
            unreachable!()
        }
    }

    pub fn get_skip_procedure_jump_location_mut(self: &mut Op) -> &mut usize {
        if let Op::SkipProc {
            location: _,
            position,
            parameters: _,
            return_types: _,
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
    Proc {
        jump_past_position: usize,
    },
}

pub fn compile_ops(tokenizer: &mut dyn Tokenizer, ops: &mut Vec<Op>) -> Result<(), Error> {
    let mut block_stack = Vec::new();
    let mut scopes = Vec::new();
    scopes.push(Vec::new());

    loop {
        let token = tokenizer.next_token()?;
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
                let mut found = false;
                match &name as &str {
                    "int" => {
                        ops.push(Op::PushType {
                            location: token.location.clone(),
                            value: Type::Integer,
                        });
                        found = true;
                    }

                    "bool" => {
                        ops.push(Op::PushType {
                            location: token.location.clone(),
                            value: Type::Bool,
                        });
                        found = true;
                    }

                    "type" => {
                        ops.push(Op::PushType {
                            location: token.location.clone(),
                            value: Type::Type,
                        });
                        found = true;
                    }

                    _ => (),
                }
                if !found {
                    for scope in scopes.iter().rev() {
                        for (decl_name, position) in scope.iter().rev() {
                            if &name == decl_name {
                                match &ops[*position] {
                                    Op::SkipProc {
                                        location: _,
                                        position: _,
                                        parameters: _,
                                        return_types: _,
                                    } => {
                                        ops.push(Op::PushFunctionPointer {
                                            location: token.location.clone(),
                                            value: position + 1,
                                        });
                                    }
                                    _ => unreachable!(),
                                }
                                found = true;
                                break;
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
                if !found {
                    return Err(Error {
                        location: token.location,
                        message: format!("Unable to find name '{}'", name),
                    });
                }
            }

            TokenKind::Print => ops.push(Op::Print {
                location: token.location,
            }),

            TokenKind::If => {
                ops.push(Op::Not {
                    location: token.location.clone(),
                });
                block_stack.push(BlockType::If {
                    position: ops.len(),
                });
                scopes.push(Vec::new());
                ops.push(Op::ConditonalJump {
                    location: token.location,
                    position: 0,
                });
                tokenizer.expect_token(TokenKind::OpenBrace)?;
            }

            TokenKind::While => block_stack.push(BlockType::While {
                position: ops.len(),
            }),

            TokenKind::Call => ops.push(Op::Call {
                location: token.location,
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
                        scopes.push(Vec::new());
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if !sucess {
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

            TokenKind::Swap => ops.push(Op::Swap {
                location: token.location,
            }),

            TokenKind::Proc => {
                let name_token = if tokenizer.peek_kind()? != TokenKind::OpenParenthesis {
                    Some(tokenizer.expect_token(TokenKind::Name)?)
                } else {
                    None
                };

                let mut depth: usize = 0;
                let open_paraentheis_token = tokenizer.expect_token(TokenKind::OpenParenthesis)?;
                let mut parameter_tokens = Vec::new();
                while (depth != 0 || tokenizer.peek_kind()? != TokenKind::CloseParenthesis)
                    && tokenizer.peek_kind()? != TokenKind::EndOfFile
                {
                    let token = tokenizer.next_token()?;
                    if token.kind == TokenKind::OpenParenthesis {
                        depth += 1;
                    } else if token.kind == TokenKind::CloseParenthesis {
                        depth -= 1;
                    }
                    parameter_tokens.push(token);
                }
                let close_parenthesis_token =
                    tokenizer.expect_token(TokenKind::CloseParenthesis)?;

                let mut parameter_ops = Vec::new();
                compile_ops(
                    &mut TokenArray {
                        filepath: token.location.filepath.clone(),
                        tokens: parameter_tokens,
                        position: 0,
                    },
                    &mut parameter_ops,
                )?;
                parameter_ops.push(Op::Exit {
                    location: close_parenthesis_token.location.clone(),
                });

                let parameter_stack = run_ops(&parameter_ops);
                let mut parameters = Vec::new();
                for value in parameter_stack {
                    if let Value::Type(value) = value {
                        parameters.push(value);
                    } else {
                        return Err(Error {
                            location: open_paraentheis_token.location.clone(),
                            message: format!("Expected type on stack, got {:?}", value),
                        });
                    }
                }

                let mut return_types = Vec::new();
                if tokenizer.peek_kind()? == TokenKind::RightArrow {
                    tokenizer.expect_token(TokenKind::RightArrow)?;

                    let open_paraentheis_token =
                        tokenizer.expect_token(TokenKind::OpenParenthesis)?;
                    let mut return_type_tokens = Vec::new();
                    let mut depth: usize = 0;
                    while (depth != 0 || tokenizer.peek_kind()? != TokenKind::CloseParenthesis)
                        && tokenizer.peek_kind()? != TokenKind::EndOfFile
                    {
                        let token = tokenizer.next_token()?;
                        if token.kind == TokenKind::OpenParenthesis {
                            depth += 1;
                        } else if token.kind == TokenKind::CloseParenthesis {
                            depth -= 1;
                        }
                        return_type_tokens.push(token);
                    }
                    let close_parenthesis_token =
                        tokenizer.expect_token(TokenKind::CloseParenthesis)?;

                    let mut return_type_ops = Vec::new();
                    compile_ops(
                        &mut TokenArray {
                            filepath: token.location.filepath.clone(),
                            tokens: return_type_tokens,
                            position: 0,
                        },
                        &mut return_type_ops,
                    )?;
                    return_type_ops.push(Op::Exit {
                        location: close_parenthesis_token.location.clone(),
                    });

                    let return_type_stack = run_ops(&return_type_ops);
                    for value in return_type_stack {
                        if let Value::Type(value) = value {
                            return_types.push(value);
                        } else {
                            return Err(Error {
                                location: open_paraentheis_token.location.clone(),
                                message: format!("Expected type on stack, got {:?}", value),
                            });
                        }
                    }
                }

                if let Some(name_token) = name_token {
                    tokenizer.expect_token(TokenKind::OpenBrace)?;
                    block_stack.push(BlockType::Proc {
                        jump_past_position: ops.len(),
                    });

                    scopes
                        .last_mut()
                        .unwrap()
                        .push((name_token.data.get_string(), ops.len()));
                    scopes.push(Vec::new());
                    ops.push(Op::SkipProc {
                        location: token.location,
                        position: 0,
                        parameters,
                        return_types,
                    });
                } else {
                    ops.push(Op::PushType {
                        location: token.location,
                        value: Type::Procedure {
                            parameters,
                            return_types,
                        },
                    })
                }
            }

            TokenKind::CloseBrace => {
                let block_type = block_stack.pop().unwrap();
                scopes.pop().unwrap();
                match &block_type {
                    BlockType::If { position } => {
                        if tokenizer.peek_kind()? == TokenKind::Else {
                            let else_token = tokenizer.next_token()?;
                            block_stack.push(BlockType::Else {
                                skip_position: ops.len(),
                            });
                            scopes.push(Vec::new());
                            ops.push(Op::Jump {
                                location: else_token.location.clone(),
                                position: 0,
                            });
                            tokenizer.expect_token(TokenKind::OpenBrace)?;
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

                    BlockType::Proc { jump_past_position } => {
                        ops.push(Op::Return {
                            location: token.location,
                        });
                        *ops[*jump_past_position].get_skip_procedure_jump_location_mut() =
                            ops.len();
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

            TokenKind::EqualEqual => ops.push(Op::Equal {
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
