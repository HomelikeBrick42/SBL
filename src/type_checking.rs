use crate::common::*;
use crate::ops::*;
use crate::types::*;

fn expect_type_count(
    stack: &Vec<(Type, SourceLocation)>,
    location: &SourceLocation,
    count: usize,
) -> Result<(), Error> {
    if stack.len() < count {
        Err(Error {
            location: location.clone(),
            message: format!(
                "Expected at least {} items on the stack, got {}",
                count,
                stack.len()
            ),
        })
    } else {
        Ok(())
    }
}

fn expect_types(
    stack: &mut Vec<(Type, SourceLocation)>,
    location: &SourceLocation,
    types: &[Type],
) -> Result<(), Error> {
    expect_type_count(stack, location, types.len())?;
    for (index, _) in types.iter().enumerate() {
        if stack[stack.len() - types.len() + index].0 != types[index] {
            let bad_type_location = &stack[stack.len() - types.len() + index].1;
            return Err(Error {
                location: location.clone(),
                message: format!(
                    "Expected type {:?} for argument {}, got type {:?}; Incorrect type came from here: {}:{}:{}",
                    types[index],
                    index + 1,
                    stack[stack.len() - types.len() + index].0,
					bad_type_location.filepath,
					bad_type_location.line,
					bad_type_location.column,
                ),
            });
        }
    }
    for _ in types {
        stack.pop();
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum JumpType {
    Jump,
    Fallthrough,
    Both,
}

#[derive(Debug, Clone, PartialEq)]
struct Context {
    pub ip: usize,
    pub stack: Vec<(Type, SourceLocation)>,
    pub condtional_jump_list: Vec<(usize, JumpType)>,
    pub procedure_info: Option<Type>,
}

pub fn type_check_ops(ops: &[Op]) -> Result<(), Error> {
    let mut contexts = Vec::new();
    contexts.push(Context {
        ip: 0,
        stack: Vec::new(),
        condtional_jump_list: Vec::new(),
        procedure_info: Option::None,
    });

    while contexts.len() > 0 {
        let context = contexts.last_mut().unwrap();
        match &ops[context.ip] {
            Op::Exit { location: _ } => {
                if context.stack.len() > 0 {
                    return Err(Error {
                        location: context.stack.pop().unwrap().1,
                        message: format!("Unexpected types on the stack at the end of the program"),
                    });
                }

                contexts.pop().unwrap();
                continue;
            }

            Op::PushFunctionPointer { location, value } => {
                let (parameters, return_types) = if let Op::SkipProc {
                    location: _,
                    position: _,
                    parameters,
                    return_types,
                } = &ops[value - 1]
                {
                    (parameters, return_types)
                } else {
                    unreachable!()
                };
                context.stack.push((
                    Type::Procedure {
                        parameters: parameters.clone(),
                        return_types: return_types.clone(),
                    },
                    location.clone(),
                ));
            }

            Op::PushInteger { location, value: _ } => {
                context.stack.push((Type::Integer, location.clone()));
            }

            Op::AddInteger { location }
            | Op::SubtractInteger { location }
            | Op::MultiplyInteger { location }
            | Op::DivideInteger { location } => {
                expect_types(
                    &mut context.stack,
                    location,
                    &[Type::Integer, Type::Integer],
                )?;
                context.stack.push((Type::Integer, location.clone()));
            }

            Op::LessThanInteger { location }
            | Op::GreaterThanInteger { location }
            | Op::LessThanEqualInteger { location }
            | Op::GreaterThanEqualInteger { location } => {
                expect_types(
                    &mut context.stack,
                    location,
                    &[Type::Integer, Type::Integer],
                )?;
                context.stack.push((Type::Bool, location.clone()));
            }

            Op::Equal { location } | Op::NotEqual { location } => {
                expect_type_count(&context.stack, location, 2)?;
                let typ = context.stack[context.stack.len() - 2].clone().0;
                expect_types(&mut context.stack, location, &[typ.clone(), typ.clone()])?;
                context.stack.push((Type::Bool, location.clone()));
            }

            Op::Not { location } => {
                expect_types(&mut context.stack, location, &[Type::Bool])?;
                context.stack.push((Type::Bool, location.clone()));
            }

            Op::Dup { location } => {
                expect_type_count(&context.stack, location, 1)?;
                let typ = context.stack.last().unwrap().clone().0;
                context.stack.push((typ, location.clone()));
            }

            Op::Drop { location } => {
                expect_type_count(&context.stack, location, 1)?;
                context.stack.pop().unwrap();
            }

            Op::Swap { location } => {
                expect_type_count(&context.stack, location, 2)?;
                let a = context.stack.pop().unwrap().0;
                let b = context.stack.pop().unwrap().0;
                context.stack.push((a, location.clone()));
                context.stack.push((b, location.clone()));
            }

            Op::Jump {
                location: _,
                position,
            } => {
                context.ip = *position;
                continue;
            }

            Op::ConditonalJump { location, position } => {
                expect_types(&mut context.stack, location, &[Type::Bool])?;

                let pos = context
                    .condtional_jump_list
                    .iter()
                    .position(|(pos, _)| *pos == context.ip);

                if let Some(pos) = pos {
                    let (_, jump_type) = &mut context.condtional_jump_list[pos];
                    match jump_type {
                        JumpType::Jump => context.ip += 1,
                        JumpType::Fallthrough => context.ip = *position,
						JumpType::Both => return Err(Error {
							location: location.clone(),
							message: format!("Internal Compiler Error: Infinite loop detected in type checker; what do we do here?")
						}),
                    }
                    *jump_type = JumpType::Both;
                } else {
                    let mut new_list = context.condtional_jump_list.clone();
                    new_list.push((context.ip, JumpType::Fallthrough));
                    let new_context = Context {
                        ip: context.ip + 1,
                        stack: context.stack.clone(),
                        condtional_jump_list: new_list,
                        procedure_info: context.procedure_info.clone(),
                    };

                    context
                        .condtional_jump_list
                        .push((context.ip, JumpType::Jump));
                    context.ip = *position;

                    contexts.push(new_context);
                }

                continue;
            }

            Op::Print { location } => {
                expect_type_count(&context.stack, location, 1)?;
                let typ = context.stack[context.stack.len() - 1].clone().0;
                expect_types(&mut context.stack, location, &[typ.clone()])?;
            }

            Op::SkipProc {
                location,
                position,
                parameters,
                return_types,
            } => {
                let procedure_type = Type::Procedure {
                    parameters: parameters.clone(),
                    return_types: return_types.clone(),
                };

                let mut stack = Vec::new();
                for parameter in parameters {
                    stack.push((parameter.clone(), location.clone()));
                }

                let new_context = Context {
                    ip: context.ip + 1,
                    stack,
                    condtional_jump_list: Vec::new(),
                    procedure_info: Option::Some(procedure_type),
                };

                context.ip = *position;

                contexts.push(new_context);

                continue;
            }

            Op::Call { location } => {
                expect_type_count(&context.stack, location, 1)?;
                let (typ, typ_location) = context.stack.pop().unwrap();
                let (parameters, return_types) = match &typ {
                    Type::Procedure {
                        parameters,
                        return_types,
                    } => (parameters, return_types),

                    _ => {
                        return Err(Error {
                            location: typ_location,
                            message: format!("Expected procedure type, but got type {:?}", typ),
                        })
                    }
                };
                expect_types(&mut context.stack, location, &parameters)?;
                for typ in return_types {
                    context.stack.push((typ.clone(), location.clone()));
                }
            }

            Op::Return { location } => {
                let return_types = match &context.procedure_info.as_ref().unwrap() {
                    Type::Procedure {
                        parameters: _,
                        return_types,
                    } => return_types,
                    _ => unreachable!(),
                };

                expect_types(&mut context.stack, location, return_types)?;
                if context.stack.len() > 0 {
                    return Err(Error {
                        location: context.stack.pop().unwrap().1.clone(),
                        message: format!(
                            "Unexpected types on the stack at the end of the procedure"
                        ),
                    });
                }

                contexts.pop().unwrap();
                continue;
            }
        }

        contexts.last_mut().unwrap().ip += 1;
    }

    Ok(())
}
