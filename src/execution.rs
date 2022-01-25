use crate::ops::*;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Integer(isize),
    Bool(bool),
}

impl Value {
    fn integer(self: Value) -> isize {
        if let Value::Integer(value) = self {
            value
        } else {
            unreachable!()
        }
    }

    fn bool(self: Value) -> bool {
        if let Value::Bool(value) = self {
            value
        } else {
            unreachable!()
        }
    }
}

pub fn run_ops(ops: &[Op]) {
    let mut ip = 0;
    let mut stack = Vec::new();

    loop {
        match &ops[ip] {
            Op::Exit { location: _ } => break,

            Op::PushInteger { location: _, value } => stack.push(Value::Integer(*value)),

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
                };
                stack.push(Value::Bool(value));
            }

            Op::NotEqual { location: _ } => {
                let value = match stack.pop().unwrap() {
                    Value::Integer(value) => value != stack.pop().unwrap().integer(),
                    Value::Bool(value) => value != stack.pop().unwrap().bool(),
                };
                stack.push(Value::Bool(value));
            }

            Op::Not { location: _ } => {
                let value = stack.pop().unwrap().bool();
                stack.push(Value::Bool(!value));
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

            Op::Print { location: _ } => {
                match stack.pop().unwrap() {
                    Value::Integer(value) => println!("{}", value),
                    Value::Bool(value) => println!("{}", value),
                };
            }
        }
        ip += 1;
    }
}
