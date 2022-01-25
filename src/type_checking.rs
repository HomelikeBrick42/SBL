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

pub fn type_check_ops(ops: &[Op]) -> Result<(), Error> {
    let mut ip = 0;
    let mut stack = Vec::new();

    loop {
        match &ops[ip] {
            Op::Exit { location: _ } => break,

            Op::PushInteger {
                location: _,
                value: _,
            } => stack.push(Type::Integer),

            Op::AddInteger { location }
            | Op::SubtractInteger { location }
            | Op::MultiplyInteger { location }
            | Op::DivideInteger { location } => {
                expect_types(&mut stack, location, &[Type::Integer, Type::Integer])?;
                stack.push(Type::Integer);
            }

            Op::Equal { location } | Op::NotEqual { location } => {
                expect_type_count(&stack, location, 2)?;
                let typ = stack[stack.len() - 2].clone();
                expect_types(&mut stack, location, &[typ.clone(), typ.clone()])?;
                stack.push(Type::Bool);
            }

            Op::Not { location } => {
                expect_types(&mut stack, location, &[Type::Bool])?;
                stack.push(Type::Bool);
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
                position: _,
            } => unimplemented!(),

            Op::Print { location } => {
                expect_type_count(&stack, location, 1)?;
                let typ = stack[stack.len() - 1].clone();
                expect_types(&mut stack, location, &[typ.clone()])?;
            }
        }
        ip += 1;
    }

    Ok(())
}
