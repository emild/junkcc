use super::parser::ast::Type;
use super::parser::ast::Const;
use super::parser::StaticInit;

#[derive(Debug)]
pub enum Program {
    ProgramDefinition(Vec<TopLevel>)
}

#[derive(Debug)]
pub enum TopLevel {
    Function(String /* name */, bool /* global */, Vec<String> /* parameters */, Vec<Instruction> /* body */),
    StaticVariable(String /* name */, bool /* global */, Type, StaticInit /* init */)
}



#[derive(Debug)]
pub enum Instruction {
    Return(Val),
    SignExtend(Val /* src */, Val /* dst */),
    Truncate(Val /* src */, Val /* dst */),
    Unary(UnaryOperator, Val /* src */, Val /* dst */),
    Binary(BinaryOperator, Val /* src 1 */, Val /* src 2 */, Val /* dst */),
    Copy(Val /* src */, Val /* dst */),
    Jump(String /* Target label */),
    JumpIfNotZero(Val /* Value to test */, String /* Target label */),
    JumpIfZero(Val /* Value to test */, String /* Targer label */),
    Label(String),
    FuncCall(String /* name */, Vec<Val> /* parameters */, Val /* return value */)
}

#[derive(Debug, Clone)]
pub enum Val {
    Constant(Const),
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