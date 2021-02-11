pub struct Program {
    pub instructions: Vec<char>,
}

#[derive(Debug)]
pub enum BfOp {
    InvalidOp,
    IncPtr(usize),
    DecPtr(usize),
    IncData(usize),
    DecData(usize),
    ReadStdin(usize),
    WriteStdout(usize),
    LoopSetToZero,
    LoopMovePtr(isize),
    LoopMoveData(isize),
    JumpIfDataZero(usize),
    JumpIfDataNotZero(usize),
}

#[derive(Debug)]
pub struct Bytecode {
    pub ops: Vec<BfOp>,
}
