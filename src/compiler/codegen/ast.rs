#[derive(Debug, Clone)]
pub enum Register {
    AL,
    AX,
    CL,
    CX,
    DL,
    DX,
    R10B,
    R10,
    R11B,
    R11
}

#[derive(Debug, Clone)]
pub enum CC { //Condition Code
    E,
    NE,
    L,
    LE,
    G,
    GE
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
    Cmp(Operand, Operand),
    Idiv(Operand),
    Cdq,
    Jmp(String),
    JmpCC(CC, String),
    SetCC(CC, Operand),
    Label(String),
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