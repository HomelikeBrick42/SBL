use crate::{
    common::Error,
    ir::{IRKind, IR},
    token::TokenKind,
    tokenizer::Tokenizer,
};

#[derive(Debug, Clone, PartialEq)]
enum Scope {
    While {
        position: usize,
    },
    WhileBody {
        while_position: usize,
        conditional_jump_position: usize,
    },
    If {
        conditional_jump_position: usize,
    },
    Else {
        end_of_then_jump_position: usize,
    },
    Proc {
        id: usize,
    },
    Const {
        start_position: usize,
    },
    Block,
    ConstantEval,
    Global,
}

#[derive(Debug, Clone, PartialEq)]
enum Decl {
    Proc { id: usize },
    Const { ir: Vec<IR> },
}

pub fn compile_ir(
    tokenizer: &mut dyn Tokenizer,
    procedures: &mut Vec<Vec<IR>>,
) -> Result<(), Error> {
    let mut scopes: Vec<(Vec<(String, Decl)>, Scope)> = Vec::new();
    scopes.push((Vec::new(), Scope::Global));
    procedures.push(Vec::new());

    'main_loop: loop {
        let mut current_procedure = None;
        for (_, scope) in scopes.iter().rev() {
            match scope {
                Scope::Proc { id } => {
                    current_procedure = Some(*id);
                    break;
                }

                Scope::Global => {
                    current_procedure = Some(0);
                    break;
                }

                _ => {}
            };
        }
        let current_procedure = current_procedure.unwrap();

        let token = tokenizer.next_token()?;
        match token.kind {
            TokenKind::EndOfFile => break,

            TokenKind::Integer => {
                let value = token.data.get_integer();
                procedures[current_procedure].push(IR {
                    location: token.location,
                    kind: IRKind::PushInt { value },
                });
            }

            TokenKind::Name => {
                let name = token.data.get_string();
                for (decls, _) in scopes.iter().rev() {
                    for (decl_name, decl) in decls.iter().rev() {
                        if decl_name == &name {
                            match decl {
                                Decl::Proc { id } => procedures[current_procedure].push(IR {
                                    location: token.location.clone(),
                                    kind: IRKind::PushProc { id: *id },
                                }),
                                Decl::Const { ir: _ } => todo!(),
                            }
                            continue 'main_loop;
                        }
                    }
                }
            }

            TokenKind::Print => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Print,
            }),

            TokenKind::If => {
                tokenizer.expect_token(TokenKind::OpenBrace)?;
                scopes.push((
                    Vec::new(),
                    Scope::If {
                        conditional_jump_position: procedures[current_procedure].len(),
                    },
                ));
                procedures[current_procedure].push(IR {
                    location: token.location,
                    kind: IRKind::JumpFalse {
                        relative_position: 0,
                    },
                });
            }

            TokenKind::While => {
                scopes.push((
                    Vec::new(),
                    Scope::While {
                        position: procedures[current_procedure].len(),
                    },
                ));
            }

            TokenKind::Const => todo!(),

            TokenKind::Proc => {
                let name = if tokenizer.peek_kind()? != TokenKind::OpenParenthesis {
                    Some(tokenizer.expect_token(TokenKind::Name)?)
                } else {
                    None
                };

                tokenizer.expect_token(TokenKind::OpenParenthesis)?;
                tokenizer.expect_token(TokenKind::CloseParenthesis)?;

                tokenizer.expect_token(TokenKind::OpenBrace)?;

                if let Some(name_token) = name {
                    let name = name_token.data.get_string();
                    scopes.last_mut().unwrap().0.push((
                        name,
                        Decl::Proc {
                            id: procedures.len(),
                        },
                    ));
                    scopes.push((
                        Vec::new(),
                        Scope::Proc {
                            id: procedures.len(),
                        },
                    ));
                    procedures.push(Vec::new());
                } else {
                    todo!()
                }
            }

            TokenKind::Call => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Call,
            }),

            TokenKind::Dup => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Dup,
            }),

            TokenKind::Drop => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Drop,
            }),

            TokenKind::Swap => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Swap,
            }),

            TokenKind::Memory => todo!(),

            TokenKind::OpenParenthesis => todo!(),
            TokenKind::CloseParenthesis => todo!(),

            TokenKind::OpenBrace => match &scopes.last().unwrap().1 {
                &Scope::While { position } => {
                    scopes.pop().unwrap();
                    scopes.push((
                        Vec::new(),
                        Scope::WhileBody {
                            while_position: position,
                            conditional_jump_position: procedures[current_procedure].len(),
                        },
                    ));
                    procedures[current_procedure].push(IR {
                        location: token.location,
                        kind: IRKind::JumpFalse {
                            relative_position: 0,
                        },
                    });
                }

                _ => {
                    scopes.push((Vec::new(), Scope::Block));
                }
            },

            TokenKind::CloseBrace => {
                let scope = scopes.pop().unwrap().1;
                match scope {
                    Scope::WhileBody {
                        while_position,
                        conditional_jump_position,
                    } => {
                        let current_pos = procedures[current_procedure].len();
                        procedures[current_procedure].push(IR {
                            location: token.location,
                            kind: IRKind::Jump {
                                relative_position: while_position as isize - current_pos as isize,
                            },
                        });

                        let current_pos = procedures[current_procedure].len();
                        let ir = &mut procedures[current_procedure][conditional_jump_position];
                        if let IRKind::JumpFalse { relative_position } = &mut ir.kind {
                            *relative_position =
                                current_pos as isize - conditional_jump_position as isize;
                        } else {
                            unreachable!()
                        }
                    }

                    Scope::If {
                        conditional_jump_position,
                    } => {
                        if tokenizer.peek_kind()? == TokenKind::Else {
                            tokenizer.expect_token(TokenKind::Else)?;
                            tokenizer.expect_token(TokenKind::OpenBrace)?;
                            scopes.push((
                                Vec::new(),
                                Scope::Else {
                                    end_of_then_jump_position: procedures[current_procedure].len(),
                                },
                            ));
                            procedures[current_procedure].push(IR {
                                location: token.location,
                                kind: IRKind::Jump {
                                    relative_position: 0,
                                },
                            });
                        }

                        let current_pos = procedures[current_procedure].len();
                        let ir = &mut procedures[current_procedure][conditional_jump_position];
                        if let IRKind::JumpFalse { relative_position } = &mut ir.kind {
                            *relative_position =
                                current_pos as isize - conditional_jump_position as isize;
                        } else {
                            unreachable!()
                        }
                    }

                    Scope::Else {
                        end_of_then_jump_position,
                    } => {
                        let current_pos = procedures[current_procedure].len();
                        let ir = &mut procedures[current_procedure][end_of_then_jump_position];
                        if let IRKind::Jump { relative_position } = &mut ir.kind {
                            *relative_position =
                                current_pos as isize - end_of_then_jump_position as isize;
                        } else {
                            unreachable!()
                        }
                    }

                    Scope::Proc { id } => {
                        procedures[id].push(IR {
                            location: token.location,
                            kind: IRKind::Return,
                        });
                    }

                    Scope::Block => {}

                    _ => {
                        return Err(Error {
                            location: token.location,
                            message: format!("Unexpected token '{:?}'", token.kind),
                        });
                    }
                }
            }

            TokenKind::Not => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Not,
            }),

            TokenKind::Plus => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Add,
            }),

            TokenKind::Minus => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Sub,
            }),

            TokenKind::Asterisk => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Mul,
            }),

            TokenKind::Slash => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Div,
            }),

            TokenKind::LessThan => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::LessThan,
            }),

            TokenKind::GreaterThan => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::GreaterThan,
            }),

            TokenKind::LessThanEqual => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::LessThanEqual,
            }),

            TokenKind::GreaterThanEqual => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::GreaterThanEqual,
            }),

            TokenKind::EqualEqual => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::Equal,
            }),

            TokenKind::NotEqual => procedures[current_procedure].push(IR {
                location: token.location,
                kind: IRKind::NotEqual,
            }),

            _ => {
                return Err(Error {
                    location: token.location,
                    message: format!("Unexpected token '{:?}'", token.kind),
                });
            }
        }
    }

    assert_eq!(tokenizer.peek_kind()?, TokenKind::EndOfFile);
    procedures[0].push(IR {
        location: tokenizer.next_token()?.location,
        kind: IRKind::Exit,
    });

    Ok(())
}
