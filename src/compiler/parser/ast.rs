pub trait Precedence {
    fn precedence(&self) -> u32;
}

#[derive(Debug)]
pub enum Expression {
    IntConstant(i32),
    Unary(UnaryOperator, Box<Expression>),
    Binary(BinaryOperator, Box<Expression>, Box<Expression>)
}

#[derive(Debug)]
pub enum UnaryOperator {
    Plus,
    Complement,
    Negate,
    LogicalNot
}

impl Precedence for UnaryOperator {
    fn precedence(&self) -> u32 {
        60
    }
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
    LogicalOr,
    LogicalAnd,
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual
}

impl Precedence for BinaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            BinaryOperator::LogicalOr => 5,

            BinaryOperator::LogicalAnd => 10,

            BinaryOperator::BitwiseOr => 15,

            BinaryOperator::BitwiseXor => 20,

            BinaryOperator::BitwiseAnd => 25,

            BinaryOperator::Equal|
            BinaryOperator::NotEqual => 30,

            BinaryOperator::LessThan|
            BinaryOperator::LessOrEqual |
            BinaryOperator::GreaterThan |
            BinaryOperator::GreaterOrEqual => 35,

            BinaryOperator::ShiftLeft|
            BinaryOperator::ShiftRight => 40,

            BinaryOperator::Add|
            BinaryOperator::Subtract => 45,

            BinaryOperator::Multiply |
            BinaryOperator::Divide   |
            BinaryOperator::Remainder => 50
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Return(Expression)
}

#[derive(Debug)]
pub enum FunctionDefinition {
    Function(String /* name */, Statement /* body */)
}

#[derive(Debug)]
pub enum Program {
    ProgramDefinition(FunctionDefinition)
}


/*                    GRAMMAR

<program>           ::= <function>
<function>          ::= "int" <identifier> "(" ["void"] ")" "{" <statement> "}"
<statement>         ::= "return" <exp> ";"
<exp>               ::= <factor> | <exp> <binop> <exp>
<factor>            ::= <int> | <unop> <factor> | "(" <exp> ")"
<unop>              ::= "+" | "-" | "~" | "!"
<binop>             ::= "-" | "+" | "*" | "/" | "%" |
                        "<<" | ">>" | "|" | "&" | "^" |
                        "&&" | "||" |
                        "==" | "!=" | "<" | " <=" | ">" | ">="
<identifier>        ::= ? Token::Identifier ?
<int>               ::= ? Token::IntConstant ?

*/