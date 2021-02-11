#![allow(dead_code)]
mod interpreter;
mod bf_utils;

use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use crate::interpreter::{bytecode_interp};
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
    let p = parse_from_file("./mandel.bf")?;
    // simple_interp(&p);
    //println!("{:?}", p.instructions);
    bytecode_interp(&p);
    Ok(())
}
