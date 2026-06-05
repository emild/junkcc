use super::super::parser::StaticInit;

#[derive(Debug, Clone, PartialEq)]
pub enum AssemblyType {
    LongWord,
    QuadWord,
    Double
}

impl AssemblyType {
    pub fn size(&self) -> usize {
        match self {
            AssemblyType::LongWord => 4,
            AssemblyType::QuadWord => 8,
            AssemblyType::Double   => 8
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            AssemblyType::LongWord => 4,
            AssemblyType::QuadWord => 8,
            AssemblyType::Double   => 8
        }
    }

}

#[derive(Debug)]
pub enum AssemblySymbolInfo {
    ObjEntry(AssemblyType /* assembly_type */,  bool /* is_static */, bool /* is_const */),
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
    SP,

    XMM0,
    XMM1,
    XMM2,
    XMM3,
    XMM4,
    XMM5,
    XMM6,
    XMM7,
    XMM8,
    XMM9,
    XMM10,
    XMM11,
    XMM12,
    XMM13,
    XMM14,
    XMM15
}

#[derive(Debug, Clone)]
pub enum CC { //Condition Code
    E,
    NE,

    /* signed comparisons */
    L,
    LE,
    G,
    GE,

    /* unsigned comparisons */
    B,
    BE,
    A,
    AE
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
    DivDouble
}


#[derive(Debug)]
pub enum Instruction {
    Mov(AssemblyType, Operand /* src */, Operand /* dst */),
    Movsx(Operand /* src */, Operand /* dst */),
    MovZeroExtend(Operand /* src */, Operand /* dst */),
    Unary(UnaryOperator, AssemblyType, Operand),
    Binary(BinaryOperator, AssemblyType, Operand, Operand),
    Cmp(AssemblyType, Operand, Operand),
    Idiv(AssemblyType, Operand),
    Div(AssemblyType, Operand),
    Shl(AssemblyType, Operand /* shift count */, Operand /* target */),
    Shra(AssemblyType, Operand /* shift count */, Operand /* target */),
    Shrl(AssemblyType, Operand /* shift count */, Operand /* target */),
    Cdq(AssemblyType),
    Jmp(String),
    JmpCC(CC, String),
    SetCC(CC, Operand),
    Label(String),
    Push(Operand),
    Call(String),
    Ret,
    Cvttsd2si(AssemblyType /* dst_type */, Operand /* src */, Operand /* dst */),
    Cvtsi2sd(AssemblyType /* src_type */, Operand /* src */, Operand /* dst */)
}

#[derive(Debug)]
pub enum TopLevel {
    Function(String /* name */, bool /* global */, Vec<Instruction> /* body */),
    StaticVariable(String /* name */, bool /* global */, usize /* alignment */, StaticInit /* init */),
    StaticConstant(String /* name */, usize /* alignment */, StaticInit /* init */)
}

#[derive(Debug)]
pub enum Program {
    ProgramDefinition(Vec<TopLevel>)
}