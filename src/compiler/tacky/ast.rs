
#[derive(Debug)]
pub enum Program {
    ProgramDefinition(FunctionDefinition)
}

#[derive(Debug)]
pub enum FunctionDefinition {
    Function(String /* name */, Vec<Instruction> /* body */)
}

#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    Unary(UnaryOperator, Val /* src */, Val /* dst */),
    Binary(BinaryOperator, Val /* src 1 */, Val /* src 2 */, Val /* dst */)
}

#[derive(Debug, Clone)]
pub enum Val {
    IntConstant(i32),
    Var(String)
}

#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder
}