#[derive(Debug, Clone)]
pub enum Register {
    AX,
    CL,
    CX,
    DX,
    R10,
    R11
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

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Or,
    And,
    Xor,
    Shl,
    Shr
}


#[derive(Debug)]
pub enum Instruction {
    Mov(Operand, Operand),
    Unary(UnaryOperator, Operand),
    Binary(BinaryOperator, Operand, Operand),
    Idiv(Operand),
    Cdq,
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