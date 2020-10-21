//! Entry point of the program.
mod error;
mod eval;
mod identifier;
mod label;
mod merge;
mod operation;
mod parser;
mod position;
mod program;
mod stack;
mod stdlib;
mod term;
mod transformations;
mod typecheck;
mod types;

use crate::program::Program;
use std::io::{self, Read};
use crate::parser::lexer::Lexer;

extern crate either;

fn main() {
        // let mut buf = String::new();
        // let content = std::io::stdin().read_to_string(&mut buf);
        // let mut lexer = Lexer::new(&buf);
        // while let Some(next) = lexer.next() {
        //     println!("Next: {:?}", next);
        // }
        // println!("Done");
    match Program::new_from_stdin() {
        Ok(mut p) => match p.eval() {
            Ok(t) => println!("Done: {:?}", t),
            Err(err) => p.report(err),
        },
        Err(msg) => eprintln!("Error when reading the source: {}", msg),
    };
}
