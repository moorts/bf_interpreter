#![allow(dead_code)]
mod interpreter;
mod bf_utils;
mod jit_utils;

use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::env;
use crate::interpreter::{bytecode_interp, simple_interp};
use crate::bf_utils::{Program};


fn parse_from_file(path: &str) -> io::Result<Program> {
    let mut instructions = Vec::new();
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        for c in line?.chars() {
            if c == '>' || c == '<' || c == '+' || c == '-' || c == '.' || c == ',' || c == '[' || c == ']' {
                instructions.push(c);
            }
        }
    }
    Ok(Program { instructions })
}


fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let p = parse_from_file(&args[1])?;
    // simple_interp(&p);
    //println!("{:?}", p.instructions);
    // simple_interp(&p);
    bytecode_interp(&p);
    Ok(())
}
