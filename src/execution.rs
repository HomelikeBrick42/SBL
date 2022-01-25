use crate::ops::*;

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Integer(isize),
    Bool(bool),
    FunctionPointer(usize),
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

    fn function_pointer(self: Value) -> usize {
        if let Value::FunctionPointer(value) = self {
            value
        } else {
            unreachable!()
        }
    }
}

pub fn run_ops(ops: &[Op]) {
    let mut ip = 0;
    let mut stack = Vec::new();
    let mut return_stack = Vec::new();

    loop {
        match &ops[ip] {
            Op::Exit { location: _ } => break,

            Op::PushInteger { location: _, value } => stack.push(Value::Integer(*value)),

            Op::PushFunctionPointer { location: _, value } => {
                stack.push(Value::FunctionPointer(*value))
            }

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
                    Value::FunctionPointer(value) => {
                        value == stack.pop().unwrap().function_pointer()
                    }
                };
                stack.push(Value::Bool(value));
            }

            Op::NotEqual { location: _ } => {
                let value = match stack.pop().unwrap() {
                    Value::Integer(value) => value != stack.pop().unwrap().integer(),
                    Value::Bool(value) => value != stack.pop().unwrap().bool(),
                    Value::FunctionPointer(value) => {
                        value != stack.pop().unwrap().function_pointer()
                    }
                };
                stack.push(Value::Bool(value));
            }

            Op::Not { location: _ } => {
                let value = stack.pop().unwrap().bool();
                stack.push(Value::Bool(!value));
            }

            Op::Call { location: _ } => {
                let location = stack.pop().unwrap().function_pointer();
                return_stack.push(ip + 1);
                ip = location;
                continue;
            }

            Op::Return { location: _ } => {
                ip = return_stack.pop().unwrap();
                continue;
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

            Op::PrintInt { location: _ } => {
                let value = stack.pop().unwrap().integer();
                println!("{:?}", value);
            }
        }
        ip += 1;
    }
}
