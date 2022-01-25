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
}

pub fn type_check_ops(ops: &[Op]) -> Result<(), Error> {
    let mut contexts = Vec::new();
    contexts.push(Context {
        ip: 0,
        stack: Vec::new(),
        condtional_jump_list: Vec::new(),
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
                let typ = context.stack[context.stack.len() - 1].clone().0;
                expect_types(&mut context.stack, location, &[typ.clone()])?;
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
        }
        contexts.last_mut().unwrap().ip += 1;
    }

    Ok(())
}
