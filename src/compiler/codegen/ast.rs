#[derive(Debug)]
pub enum Operand {
    Register,
    Imm(i32)
}

#[derive(Debug)]
pub enum Instruction {
    Mov(Operand, Operand),
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