
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
    Binary(BinaryOperator, Val /* src 1 */, Val /* src 2 */, Val /* dst */),
    Copy(Val /* src */, Val /* dst */),
    Jump(String /* Target label */),
    JumpIfNotZero(Val /* Value to test */, String /* Target label */),
    JumpIfZero(Val /* Value to test */, String /* Targer label */),
    Label(String)
}

#[derive(Debug, Clone)]
pub enum Val {
    IntConstant(i32),
    Var(String)
}

#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Plus,
    LogicalNot
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    ShiftLeft,
    ShiftRight,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual
}