#[derive(Debug, Clone)]
pub enum Register {
    AX,
    R10
}


#[derive(Debug, Clone)]
pub enum Operand {
    Imm(i32),
    Reg(Register),
    Pseudo(String),
    Stack(i64)
}


#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not
}


#[derive(Debug)]
pub enum Instruction {
    Mov(Operand, Operand),
    Unary(UnaryOperator, Operand),
    AllocateStack(i64),
    Ret
}

#[derive(Debug)]
pub enum FunctionDefinition {
    Function(String, Vec<Instruction>)
}

#[derive(Debug)]
pub enum Program {
    ProgramDefinition(FunctionDefinition)
}