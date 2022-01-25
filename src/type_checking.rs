use crate::common::*;
use crate::ops::*;
use crate::types::*;

fn expect_type_count(
    stack: &Vec<Type>,
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
    stack: &mut Vec<Type>,
    location: &SourceLocation,
    types: &[Type],
) -> Result<(), Error> {
    expect_type_count(stack, location, types.len())?;
    for (index, _) in types.iter().enumerate() {
        if stack[stack.len() - types.len() + index] != types[index] {
            return Err(Error {
                location: location.clone(),
                message: format!(
                    "Expected type {:?} for argument {}, got type {:?}",
                    types[index],
                    index + 1,
                    stack[stack.len() - types.len() + index]
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
struct Context {
    pub ip: usize,
    pub stack: Vec<Type>,
}

pub fn type_check_ops(ops: &[Op]) -> Result<(), Error> {
    let mut contexts = Vec::new();
    contexts.push(Context {
        ip: 0,
        stack: Vec::new(),
    });

    while contexts.len() > 0 {
        match &ops[contexts.last().unwrap().ip] {
            Op::Exit { location: _ } => {
                contexts.pop().unwrap();
                continue;
            }

            Op::PushInteger {
                location: _,
                value: _,
            } => {
                let context = contexts.last_mut().unwrap();
                context.stack.push(Type::Integer);
            }

            Op::AddInteger { location }
            | Op::SubtractInteger { location }
            | Op::MultiplyInteger { location }
            | Op::DivideInteger { location } => {
                let context = contexts.last_mut().unwrap();
                expect_types(
                    &mut context.stack,
                    location,
                    &[Type::Integer, Type::Integer],
                )?;
                context.stack.push(Type::Integer);
            }

            Op::Equal { location } | Op::NotEqual { location } => {
                let context = contexts.last_mut().unwrap();
                expect_type_count(&context.stack, location, 2)?;
                let typ = context.stack[context.stack.len() - 2].clone();
                expect_types(&mut context.stack, location, &[typ.clone(), typ.clone()])?;
                context.stack.push(Type::Bool);
            }

            Op::Not { location } => {
                let context = contexts.last_mut().unwrap();
                expect_types(&mut context.stack, location, &[Type::Bool])?;
                context.stack.push(Type::Bool);
            }

            Op::Jump {
                location: _,
                position,
            } => {
                let context = contexts.last_mut().unwrap();
                context.ip = *position;
                continue;
            }

            Op::ConditonalJump { location, position } => {
                expect_types(
                    &mut contexts.last_mut().unwrap().stack,
                    location,
                    &[Type::Bool],
                )?;

                let context_a = Context {
                    ip: *position,
                    stack: contexts.last().unwrap().stack.clone(),
                };
                let context_b = Context {
                    ip: contexts.last().unwrap().ip + 1,
                    stack: contexts.last().unwrap().stack.clone(),
                };

                contexts.pop().unwrap();
                contexts.push(context_a);
                contexts.push(context_b);

                continue;
            }

            Op::Print { location } => {
                let context = contexts.last_mut().unwrap();
                expect_type_count(&context.stack, location, 1)?;
                let typ = context.stack[context.stack.len() - 1].clone();
                expect_types(&mut context.stack, location, &[typ.clone()])?;
            }
        }
        contexts.last_mut().unwrap().ip += 1;
    }

    Ok(())
}