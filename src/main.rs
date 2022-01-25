#![allow(dead_code)]

use std::{array::IntoIter, collections::HashMap, env::args, process::exit};

mod common;
mod execution;
mod lexer;
mod ops;

use crate::common::*;
use crate::execution::*;
use crate::lexer::*;
use crate::ops::*;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file>", args[0]);
        exit(1)
    }

    let filepath = &args[1];

    let source = std::fs::read_to_string(filepath).unwrap_or_else(|_| {
        eprintln!("Unable to open file '{}'", filepath);
        exit(1)
    });

    let mut lexer = Lexer::new(filepath.clone(), &source as &str);

    /*
    loop {
        let token = lexer.next_token().unwrap_or_else(|error| {
            eprintln!(
                "{}:{}:{}: {}",
                error.location.filepath, error.location.line, error.location.column, error.message
            );
            exit(1)
        });
        println!("{:?}", token);
        if let TokenKind::EndOfFile = token.kind {
            break;
        }
    }
    */

    let mut ops = Vec::new();

    let jump_op_location = ops.len();
    ops.push(Op::Jump {
        location: SourceLocation {
            filepath: "buitin.sbl".to_string(),
            position: 0,
            line: 1,
            column: 1,
        },
        position: 0,
    });

    let print_int_location = ops.len();
    ops.push(Op::PrintInt {
        location: SourceLocation {
            filepath: "buitin.sbl".to_string(),
            position: 0,
            line: 1,
            column: 1,
        },
    });
    ops.push(Op::Return {
        location: SourceLocation {
            filepath: "buitin.sbl".to_string(),
            position: 0,
            line: 1,
            column: 1,
        },
    });

    *ops[jump_op_location].get_jump_location_mut() = ops.len();

    let mut functions = Vec::from([HashMap::<String, usize>::from_iter(IntoIter::new([(
        "print_int".to_string(),
        print_int_location,
    )]))]);

    compile_ops(&mut lexer, &mut ops, &mut functions).unwrap_or_else(|error| {
        eprintln!(
            "{}:{}:{}: {}",
            error.location.filepath, error.location.line, error.location.column, error.message
        );
        exit(1)
    });

    ops.push(Op::Exit {
        location: lexer
            .expect_token(TokenKind::EndOfFile)
            .unwrap_or_else(|error| {
                eprintln!(
                    "{}:{}:{}: {}",
                    error.location.filepath,
                    error.location.line,
                    error.location.column,
                    error.message
                );
                exit(1)
            })
            .location,
    });

    /*
    for (index, op) in ops.iter().enumerate() {
        println!("{} = {:?}", index, op);
    }
    */

    run_ops(&ops);
}
