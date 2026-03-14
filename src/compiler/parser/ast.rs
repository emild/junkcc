pub trait Precedence {
    fn precedence(&self) -> u32;
}

#[derive(Debug)]
pub enum Expression {
    IntConstant(i32),
    Var(String),
    Unary(UnaryOperator, Box<Expression>),
    PreIncrement(Box<Expression>),
    PreDecrement(Box<Expression>),
    PostIncrement(Box<Expression>),
    PostDecrement(Box<Expression>),
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),
    Assignment(Box<Expression>, Box<Expression>),
    CompoundAssignment(BinaryOperator, Box<Expression>, Box<Expression>),
    Conditional(Box<Expression> /* condition */, Box<Expression> /* true */, Box<Expression> /* false */)
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Plus,
    Complement,
    Negate,
    LogicalNot,
    PreIncrement,
    PreDecrement,
    PostIncrement,
    PostDecrement
}

impl Precedence for UnaryOperator {
    fn precedence(&self) -> u32 {
        60
    }
}


#[derive(Debug, PartialEq, Clone)]
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
    GreaterOrEqual,
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    RemainderAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    ShiftLeftAssign,
    ShiftRightAssign,
    ConditionalMiddle //Not a real binary operator
}

impl Precedence for BinaryOperator {
    fn precedence(&self) -> u32 {
        match self {
            BinaryOperator::Assign |
            BinaryOperator::AddAssign |
            BinaryOperator::SubtractAssign |
            BinaryOperator::MultiplyAssign |
            BinaryOperator::DivideAssign |
            BinaryOperator::RemainderAssign |
            BinaryOperator::BitwiseAndAssign |
            BinaryOperator::BitwiseOrAssign |
            BinaryOperator::BitwiseXorAssign |
            BinaryOperator::ShiftLeftAssign |
            BinaryOperator::ShiftRightAssign => 1,

            BinaryOperator::ConditionalMiddle => 3,

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
    Stmnt(Option<Vec<String>> /* labels */, UnlabeledStatement)
}

#[derive(Debug)]
pub enum UnlabeledStatement {
    Return(Expression),
    Goto(String),
    If(Expression, Box<Statement> /* then */, Option<Box<Statement>> /* else */),
    Expr(Expression),
    Null
}

#[derive(Debug)]
pub enum Declaration {
    Declarant(String /* id */, Option<Expression> /* initializer */)
}


#[derive(Debug)]
pub enum BlockItem {
    D(Declaration),
    S(Statement)
}

#[derive(Debug)]
pub enum FunctionDefinition {
    Function(String /* name */, Vec<BlockItem> /* body */)
}

#[derive(Debug)]
pub enum Program {
    ProgramDefinition(FunctionDefinition)
}


/*                    GRAMMAR

<program>           ::= <function>
<function>          ::= "int" <identifier> "(" ["void"] ")" "{" { <block_item> } "}"
<block_item>        ::= <statement>|<declaration>
<declaration>       ::= "int" <identifier> [ "=" <exp> ] ";"
<label>             ::= <id> ":"
<statement>         ::= [<label> *] <unlbld_statement>
<unlbld_statement>  ::= "return" <exp> ";" | <exp> ";" |
                        ";" | if <exp> <statement> ["else" <statement> ] |
                        "goto" <id> ";"
<exp>               ::= <factor> | <exp> <binop> <exp> | <exp> "?" <exp> ":" <exp>
<factor>            ::= <int> | <identifier> | <unop> <factor> | "(" <exp> ")" |
                        <inc_dec> <factor> | <factor> <inc_dec>
<unop>              ::= "+"  | "-" | "~" | "!"
<inc_dec>           ::= "++" | "--"
<binop>             ::= "-"  | "+" | "*" | "/" | "%" |
                        "<<" | ">>" | "|"  | "&"  | "^"  |
                        "&&" | "||" |
                        "==" | "!=" | "<"  | "<=" | ">"  | ">=" |
                        "="  | "+=" | "-=" | "*=" | "/=" | "%=" |
                        "|=" | "&=" |
                        "<<="| ">>="

<identifier>        ::= ? Token::Identifier ?
<int>               ::= ? Token::IntConstant ?

*/
