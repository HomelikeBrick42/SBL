#![allow(dead_code)]

use std::{env::args, process::exit};

mod common;
mod execution;
mod lexer;
mod ops;
mod type_checking;
mod types;

use crate::execution::*;
use crate::lexer::*;
use crate::ops::*;
use crate::type_checking::*;

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
    let mut ops = Vec::new();

    compile_ops(&mut lexer, &mut ops).unwrap_or_else(|error| {
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

    type_check_ops(&ops).unwrap_or_else(|error| {
        eprintln!(
            "{}:{}:{}: {}",
            error.location.filepath, error.location.line, error.location.column, error.message
        );
        exit(1)
    });

    run_ops(&ops);
}
