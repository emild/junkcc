use std::collections::HashMap;

pub trait Precedence {
    fn precedence(&self) -> u32;
}

#[derive(Debug, Clone)]
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


#[derive(Debug, Clone)]
pub enum Label {
    Goto(String),
    Case(Expression), //Expression must be constant
    Default
}


#[derive(Debug)]
pub enum Statement {
    Stmnt(Option<Vec<Label>> /* labels */, UnlabeledStatement)
}

#[derive(Debug)]
pub enum ForInit {
    InitDecl(Declaration),
    InitExp(Option<Expression>)
}

#[derive(Debug, PartialEq, Clone)]
pub enum BreakType {
    Loop,
    Switch
}


#[derive(Debug)]
pub enum UnlabeledStatement {
    Return(Expression),
    Goto(String),
    If(Expression, Box<Statement> /* then */, Option<Box<Statement>> /* else */),
    Break(Option<BreakType>, Option<String> /* loop/switch label */),
    Continue(Option<String> /* loop label */),
    While(Expression /* condition */, Box<Statement> /* body */, Option<String> /* loop label */),
    DoWhile(Box<Statement> /* body */, Expression /* condition */, Option<String> /* loop label */),
    For(ForInit, Option<Expression> /* condition */, Option<Expression> /* post */, Box<Statement> /* body */, Option<String> /* loop label */),
    Switch(Expression, Box<Statement> /* body */, Option<String> /* switch label */, Vec<Label> /* case and default labels */, HashMap<i32, String>, /* case constants and global labels */ Option<String> /* default_label */ ),
    Compound(Block),
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
pub enum Block {
    Blk(Vec<BlockItem>)
}

#[derive(Debug)]
pub enum FunctionDefinition {
    Function(String /* name */, Block /* body */)
}

#[derive(Debug)]
pub enum Program {
    ProgramDefinition(FunctionDefinition)
}


/*                    GRAMMAR

<program>           ::= <function>
<function>          ::= "int" <identifier> "(" ["void"] ")" block
<block_item>        ::= <statement>|<declaration>
<declaration>       ::= "int" <identifier> [ "=" <exp> ] ";"
<label>             ::= <id> ":" | "case" <exp> ":" | "default" ":"
<block>             ::= "{" [<block_item> *] "}"
<statement>         ::= [<label> *] <unlbld_statement>
<unlbld_statement>  ::= ";" |
                        <exp> ";" |
                        "return" <exp> ";" |
                        "goto" <id> ";"   |
                        "break" ";" |
                        "continue" ";"  |
                        "if" "(" <exp> ")" <statement> ["else" <statement> ] |
                        "while" "(" <exp> ")" <statement> |
                        "do" <statement> "while" "(" <exp> ")" |
                        "for" "(" for_init ";" [<exp>] ";" [<exp>] ")" <statement>  |
                        "switch" "(" <exp ")" <statement>
                        <block>
<for_init>          ::= <declaration> | [<exp>]
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
