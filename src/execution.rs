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

            Op::PushFunctionPointer { location: _, value } => {
                stack.push(Value::FunctionPointer(*value))
            }

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

            Op::LessThanInteger { location: _ } => {
                let b = stack.pop().unwrap().integer();
                let a = stack.pop().unwrap().integer();
                stack.push(Value::Bool(a < b));
            }

            Op::GreaterThanInteger { location: _ } => {
                let b = stack.pop().unwrap().integer();
                let a = stack.pop().unwrap().integer();
                stack.push(Value::Bool(a > b));
            }

            Op::LessThanEqualInteger { location: _ } => {
                let b = stack.pop().unwrap().integer();
                let a = stack.pop().unwrap().integer();
                stack.push(Value::Bool(a <= b));
            }

            Op::GreaterThanEqualInteger { location: _ } => {
                let b = stack.pop().unwrap().integer();
                let a = stack.pop().unwrap().integer();
                stack.push(Value::Bool(a >= b));
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

            Op::Dup { location: _ } => {
                let value = stack.last().unwrap().clone();
                stack.push(value);
            }

            Op::Drop { location: _ } => {
                stack.pop().unwrap();
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
                    Value::FunctionPointer(value) => println!("{}", value),
                };
            }

            Op::SkipProc {
                location: _,
                position,
                parameters: _,
                return_types: _,
            } => {
                ip = *position;
                continue;
            }

            Op::Call { location: _ } => {
                return_stack.push(ip + 1);
                let position = stack.pop().unwrap().function_pointer();
                ip = position;
                continue;
            }

            Op::Return { location: _ } => {
                ip = return_stack.pop().unwrap();
                continue;
            }
        }
        ip += 1;
    }
}
