use super::super::parser::StaticInit;

#[derive(Debug, Clone)]
pub enum AssemblyType {
    LongWord,
    QuadWord
}

impl AssemblyType {
    pub fn size(&self) -> usize {
        match self {
            AssemblyType::LongWord => 4,
            AssemblyType::QuadWord => 8
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            AssemblyType::LongWord => 4,
            AssemblyType::QuadWord => 8
        }
    }

}

#[derive(Debug)]
pub enum AssemblySymbolInfo {
    ObjEntry(AssemblyType /* assembly_type */,  bool /* is_static */),
    FuncEntry(bool /* is_defined */)
}


#[derive(Debug, Clone)]
pub enum Register {
    AX,
    BX,
    CX,
    DX,
    DI,
    SI,
    R8,
    R9,
    R10,
    R11,
    SP
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
    Imm(i64),
    Reg(Register),
    Pseudo(String),
    Stack(i64),
    Data(String)
}

impl Operand
{
    pub fn is_mem(&self) -> bool
    {
        use Operand::*;
        match self {
            Stack(_) |
            Data(_) => true,
            Pseudo(_) => { panic!("Unresolved pseudo operand: '{:?}'", self); }
            Reg(_) |
            Imm(_) => false
        }
    }

    pub fn is_imm(&self) -> bool
    {
        use Operand::*;
        match self {
            Stack(_)    |
            Data(_)     |
            Reg(_)  => false,
            Pseudo(_) => { panic!("Unresolved pseudo operand: '{:?}'", self); },
            Imm(_) => true
        }
    }

}





#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Mov(AssemblyType, Operand /* src */, Operand /* dst */),
    Movsx(Operand /* src */, Operand /* dst */),
    Unary(UnaryOperator, AssemblyType, Operand),
    Binary(BinaryOperator, AssemblyType, Operand, Operand),
    Cmp(AssemblyType, Operand, Operand),
    Idiv(AssemblyType, Operand),
    Cdq(AssemblyType),
    Jmp(String),
    JmpCC(CC, String),
    SetCC(CC, Operand),
    Label(String),
    Push(Operand),
    Call(String),
    Ret
}

#[derive(Debug)]
pub enum TopLevel {
    Function(String /* name */, bool /* global */, Vec<Instruction> /* body */),
    StaticVariable(String /* name */, bool /* global */, usize /* alignment */, StaticInit /* init */)
}

#[derive(Debug)]
pub enum Program {
    ProgramDefinition(Vec<TopLevel>)
}