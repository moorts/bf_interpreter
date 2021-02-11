use std::convert::TryInto;
use std::io::{self, Read, Write};
use crate::bf_utils::{Program, Bytecode, BfOp};

fn optimize_loop(ops: &Vec<BfOp>, loop_start: usize) -> Option<BfOp> {
    if ops.len() - loop_start == 2 {
        match ops[loop_start + 1] {
            BfOp::IncData(_) | BfOp::DecData(_) => Some(BfOp::LoopSetToZero),
            BfOp::IncPtr(c) => return Some(BfOp::LoopMovePtr(c.try_into().unwrap())),
            BfOp::DecPtr(c) => return Some(BfOp::LoopMovePtr(-(c as isize))),
            _ => return None,
        }
    } else if ops.len() - loop_start == 5 {
        // matches [->+<] and [-<+>] expressions
        match ops[loop_start + 1] {
            BfOp::DecData(1) => {
                match ops[loop_start + 3] {
                    BfOp::IncData(1) => {
                        match ops[loop_start + 2] {
                            BfOp::IncPtr(c) => {
                                match ops[loop_start + 4] {
                                    BfOp::DecPtr(d) => {
                                        if c == d {
                                            return Some(BfOp::LoopMoveData(c as isize));
                                        } else {
                                            return None;
                                        }
                                    },
                                    _ => return None,
                                }
                            },
                            BfOp::DecPtr(c) => {
                                match ops[loop_start + 4] {
                                    BfOp::IncPtr(d) => {
                                        if c == d {
                                            return Some(BfOp::LoopMoveData(-(c as isize)));
                                        } else {
                                            return None;
                                        }
                                    },
                                    _ => return None,
                                }
                            },
                            _ => return None,
                        }
                    },
                    _ => return None,
                }
            }
            _ => return None,
        } 
    } else {
        return None;
    }
}

fn translate_program(p: &Program) -> Bytecode {
    let mut ops = Vec::new();
    let mut i = 0;

    let mut open_bracket_stack = Vec::new();
    while i < p.instructions.len() {
        if p.instructions[i] == '[' {
            open_bracket_stack.push(ops.len());
            ops.push(BfOp::InvalidOp);
        } else if p.instructions[i] == ']' {
            let open_idx = open_bracket_stack.remove(open_bracket_stack.len() - 1);
            match optimize_loop(&ops, open_idx) {
                Some(op) => {
                    ops.truncate(open_idx);
                    ops.push(op);
                },
                None => {
                    ops[open_idx] = BfOp::JumpIfDataZero(ops.len() - open_idx);
                    ops.push(BfOp::JumpIfDataNotZero(ops.len() - open_idx));
                }
            }
        }else {
            let mut count = 1;
            while (i + 1) < p.instructions.len() && p.instructions[i+1] == p.instructions[i] {
                count += 1;
                i += 1;
            }
            match p.instructions[i] {
                '>' => ops.push(BfOp::IncPtr(count)),
                '<' => ops.push(BfOp::DecPtr(count)),
                '+' => ops.push(BfOp::IncData(count)),
                '-' => ops.push(BfOp::DecData(count)),
                '.' => ops.push(BfOp::WriteStdout(count)),
                ',' => ops.push(BfOp::ReadStdin(count)),
                _ => (),
            }
        }
        i += 1;
    }
    Bytecode{ ops }
}

fn compute_jumptable(p: &Program) -> Vec<usize> {
    let program_size = p.instructions.len();
    let mut jump_table = vec![0; program_size];

    let mut pc: usize = 0;

    while pc < program_size {
        match p.instructions[pc] {
            '[' => {
                let mut bracket_nesting = 1;
                let mut seek = pc; 

                while bracket_nesting > 0 && (seek + 1) < program_size {
                    seek += 1;
                    match p.instructions[seek] {
                        '[' => bracket_nesting += 1,
                        ']' => bracket_nesting -= 1,
                        _ => (),
                    }
                }
                if bracket_nesting == 0 {
                    jump_table[seek] = pc;
                    jump_table[pc] = seek;
                }
            },
            _ => (),
        }
        pc += 1;
    }
    jump_table
}

const MEMORY_SIZE: usize = 30000;

fn simple_interp(p: &Program) -> io::Result<()> {
    let mut memory: Vec<u8> = vec![0; MEMORY_SIZE];
    let mut pc: usize = 0;
    let mut dataptr: usize = 0;
    let jump_table = compute_jumptable(p);

    while pc < p.instructions.len() {
        let instruction = p.instructions[pc];
        match instruction {
            '>' => dataptr += 1,
            '<' => dataptr -= 1,
            '+' => {
                memory[dataptr] = memory[dataptr].wrapping_add(1)
            },
            '-' => {
                memory[dataptr] = memory[dataptr].wrapping_sub(1)
            },
            '.' => {
                print!("{}", memory[dataptr] as char);
                io::stdout().flush()?;
            },
            ',' => memory[dataptr] = io::stdin().bytes().next().expect("no input").ok().unwrap(),
            '[' => {
                if memory[dataptr] == 0 {
                    pc = jump_table[pc];
                }
            },
            ']' => {
                if memory[dataptr] != 0 {
                    pc = jump_table[pc];
                }
            }
            _ => (),
        };
        pc += 1;
    }

    Ok(())
}

pub fn bytecode_interp(p: &Program) {
    let mut memory: Vec<u8> = vec![0; MEMORY_SIZE];
    let b = translate_program(&p);
    // println!("{:?}", b);
    let mut pc: usize = 0;
    let mut dataptr: usize = 0;

    while pc < b.ops.len() {
        match b.ops[pc] {
            BfOp::IncPtr(c) => dataptr += c,
            BfOp::DecPtr(c) => dataptr -= c,
            BfOp::IncData(c) => {
                memory[dataptr] = memory[dataptr].wrapping_add(c.try_into().unwrap());
            },
            BfOp::DecData(c) => {
                memory[dataptr] = memory[dataptr].wrapping_sub(c.try_into().unwrap());
            },
            BfOp::WriteStdout(c) => {
                for _ in 0..c {
                    print!("{}", memory[dataptr] as char);
                }
            },
            BfOp::ReadStdin(c) => {
                for _ in 0..c {
                    memory[dataptr] = io::stdin().bytes().next().expect("no input").ok().unwrap();
                }
            }
            BfOp::JumpIfDataZero(c) => {
                if memory[dataptr] == 0 {
                    pc += c;
                }
            },
            BfOp::JumpIfDataNotZero(c) => {
                if memory[dataptr] != 0 {
                    pc -= c;
                }
            },
            BfOp::LoopSetToZero => memory[dataptr] = 0,
            BfOp::LoopMovePtr(c) => {
                while memory[dataptr] != 0 {
                    let new_dataptr = (dataptr as isize) + c;
                    dataptr = new_dataptr as usize;
                }
            },
            BfOp::LoopMoveData(c) => {
                if memory[dataptr] != 0{
                    let move_to_ptr = (dataptr as isize) + c;
                    memory[move_to_ptr as usize] = memory[move_to_ptr as usize].wrapping_add(memory[dataptr]);
                    memory[dataptr] = 0;
                }
            },
            _ => (),
        }
        pc += 1;
    }
}
