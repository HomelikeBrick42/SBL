#![allow(dead_code)]

use std::{env::args, process::exit};

use compile::compile_ir;

use crate::lexer::Lexer;

mod common;
mod compile;
mod ir;
mod lexer;
mod token;
mod tokenizer;

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
    let mut procedures = Vec::new();

    compile_ir(&mut lexer, &mut procedures).unwrap_or_else(|error| {
        eprintln!(
            "{}:{}:{}: {}",
            error.location.filepath, error.location.line, error.location.column, error.message
        );
        exit(1)
    });

    for (procedure_index, procedure) in procedures.iter().enumerate() {
        println!("Procedure: {}", procedure_index);
        for (ir_index, ir) in procedure.iter().enumerate() {
            println!("    {} = {:?}", ir_index, ir.kind);
        }
        println!();
    }
}
