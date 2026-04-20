use std::collections::HashMap;

pub trait Precedence {
    fn precedence(&self) -> u32;
}


#[derive(Debug, Clone)]
pub enum TypedExpression {
    TypedExp(Option<Type>, Expression)
}


#[derive(Debug, Clone)]
pub enum Expression {
    Constant(Const),
    Var(String),
    Cast(Type /* target_type */, Box<TypedExpression>),
    Unary(UnaryOperator, Box<TypedExpression>),
    PreIncrement(Box<TypedExpression>),
    PreDecrement(Box<TypedExpression>),
    PostIncrement(Box<TypedExpression>),
    PostDecrement(Box<TypedExpression>),
    Binary(BinaryOperator, Box<TypedExpression>, Box<TypedExpression>),
    Assignment(Box<TypedExpression>, Box<TypedExpression>),
    CompoundAssignment(BinaryOperator, Box<TypedExpression>, Box<TypedExpression>),
    Conditional(Box<TypedExpression> /* condition */, Box<TypedExpression> /* true */, Box<TypedExpression> /* false */),
    FunctionCall(String /* func_name */, Vec<TypedExpression> /* args */)
}



pub fn typex_get_type(typed_expr: &TypedExpression) -> Type
{
    let TypedExpression::TypedExp(typ, _expr) = typed_expr;

    typ.clone().unwrap()
}


pub fn typex_set_type(expr: Expression, typ: Type) -> TypedExpression
{
    TypedExpression::TypedExp(Some(typ), expr)
}


pub fn typex_init(expr: Expression) -> TypedExpression
{
    TypedExpression::TypedExp(None, expr)
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Const {
    ConstInt(i32),
    ConstLong(i64)
}

impl Const {
    pub fn to_typex(&self) -> TypedExpression
    {
        let typ = match self {
            Const::ConstInt(_)  => Type::Int,
            Const::ConstLong(_) => Type::Long
        };
        typex_set_type(Expression::Constant(self.clone()), typ)
    }

    pub fn to_i64(&self) -> i64
    {
        match self {
            Const::ConstInt(c)  => i64::from(*c),
            Const::ConstLong(c) => *c
        }
    }

    pub fn is_false(&self) -> bool
    {
        match self {
            Const::ConstInt(0) |
            Const::ConstLong(0) => true,

            _ => false
        }
    }

    pub fn is_true(&self) -> bool
    {
        !self.is_false()
    }

    pub fn convert_to(&self, typ: &Type) -> Const
    {
        match (typ, self) {
            (Type::Int,  Const::ConstInt(_)) |
            (Type::Long, Const::ConstLong(_))  => self.clone(),

            (Type::Int, Const::ConstLong(c)) => Const::ConstInt((*c & 0xFFFFFFFF) as i32),
            (Type::Long, Const::ConstInt(c)) => Const::ConstLong(i64::from(*c)),
            (Type::FuncType(_, _), _) => { panic!("Cannot convert anything to a function"); }
        }
    }

    pub fn complement(&self) -> Const
    {
        match self {
            Const::ConstInt(c)  => Const::ConstInt(!c),
            Const::ConstLong(c) => Const::ConstLong(!c)
        }
    }

    pub fn logical_not(&self) -> Const
    {
        match self {
            Const::ConstInt(c)  => Const::ConstInt((*c == 0)  as i32),
            Const::ConstLong(c) => Const::ConstLong((*c == 0) as i64)
        }
    }

    pub fn unary_minus(&self) -> Const
    {
        match self {
            Const::ConstInt(c)  => Const::ConstInt(-*c),
            Const::ConstLong(c) => Const::ConstLong(-*c)
        }
    }

    pub fn unary_plus(&self) -> Const
    {
        match self {
            Const::ConstInt(c)  => Const::ConstInt(*c),
            Const::ConstLong(c) => Const::ConstLong(*c)
        }
    }



    pub fn add(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a + b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a + b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstLong(i64::from(*a) + b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a + i64::from(*b))
        }
    }

    pub fn sub(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a - b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a - b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstLong(i64::from(*a) - b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a - i64::from(*b))
        }
    }

    pub fn mul(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a * b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a * b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstLong(i64::from(*a) * b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a * i64::from(*b))
        }
    }

    pub fn div(&self, other: &Const) -> Const
    {
        assert!(other.is_true());
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a / b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a / b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstLong(i64::from(*a) / b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a / i64::from(*b))
        }
    }

    pub fn modulo(&self, other: &Const) -> Const
    {
        assert!(other.is_true());
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a % b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a % b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstLong(i64::from(*a) % b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a % i64::from(*b))
        }
    }

    pub fn bin_and(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a & b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a & b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstLong(i64::from(*a) & b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a & i64::from(*b))
        }
    }


    pub fn bin_or(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a | b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a | b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstLong(i64::from(*a) | b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a | i64::from(*b))
        }
    }


    pub fn bin_xor(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a ^ b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a ^ b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstLong(i64::from(*a) ^ b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a ^ i64::from(*b))
        }
    }


    pub fn left_shift(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a << b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a << b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstInt(a << b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a << b)
        }
    }

    pub fn right_shift(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt(a >> b),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstLong(a >> b),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstInt(a >> b),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstLong(a >> b)
        }
    }


    pub fn logical_and(&self, other: &Const) -> Const
    {
        if self.is_false() || other.is_false() {
            Const::ConstInt(0)
        }
        else {
            Const::ConstInt(1)
        }
    }


    pub fn logical_or(&self, other: &Const) -> Const
    {
        if self.is_true() || other.is_true() {
            Const::ConstInt(1)
        }
        else {
            Const::ConstInt(0)
        }
    }

    pub fn lt(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt((a < b) as i32),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstInt((a < b) as i32),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstInt((i64::from(*a) < *b) as i32),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstInt((*a < i64::from(*b)) as i32)
        }
    }

    pub fn gt(&self, other: &Const) -> Const
    {
        other.lt(self)
    }

    pub fn le(&self, other: &Const) -> Const
    {
        other.lt(self).logical_not()
    }

    pub fn ge(&self, other: &Const) -> Const
    {
        self.lt(other).logical_not()
    }

    pub fn eq(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt((a == b) as i32),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstInt((a == b) as i32),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstInt((i64::from(*a) == *b) as i32),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstInt((*a == i64::from(*b)) as i32)
        }
    }

    pub fn ne(&self, other: &Const) -> Const
    {
        match (self, other) {
            (Const::ConstInt(a), Const::ConstInt(b)) => Const::ConstInt((a != b) as i32),
            (Const::ConstLong(a), Const::ConstLong(b)) => Const::ConstInt((a != b) as i32),
            (Const::ConstInt(a), Const::ConstLong(b)) => Const::ConstInt((i64::from(*a) != *b) as i32),
            (Const::ConstLong(a), Const::ConstInt(b)) => Const::ConstInt((*a != i64::from(*b)) as i32)
        }
    }


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
    Case(TypedExpression), //Expression must be constant
    Default,
    ResolvedCase(String)
}


#[derive(Debug)]
pub enum Statement {
    Stmnt(Option<Vec<Label>> /* labels */, UnlabeledStatement)
}

#[derive(Debug)]
pub enum ForInit {
    InitDecl(VariableDeclaration),
    InitExp(Option<TypedExpression>)
}

#[derive(Debug, PartialEq, Clone)]
pub enum BreakType {
    Loop,
    Switch
}


#[derive(Debug)]
pub enum UnlabeledStatement {
    Return(TypedExpression),
    Goto(String),
    If(TypedExpression, Box<Statement> /* then */, Option<Box<Statement>> /* else */),
    Break(Option<BreakType>, Option<String> /* loop/switch label */),
    Continue(Option<String> /* loop label */),
    While(TypedExpression /* condition */, Box<Statement> /* body */, Option<String> /* loop label */),
    DoWhile(Box<Statement> /* body */, TypedExpression /* condition */, Option<String> /* loop label */),
    For(ForInit, Option<TypedExpression> /* condition */, Option<TypedExpression> /* post */, Box<Statement> /* body */, Option<String> /* loop label */),
    Switch(TypedExpression, Box<Statement> /* body */, Option<String> /* switch label */, HashMap<Const, String>, /* case constants and global labels */ Option<String> /* default_label */ ),
    Compound(Block),
    Expr(TypedExpression),
    Null
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Long,
    FuncType(Vec<Type> /* param_types */, Box<Type> /* ret_type */ )
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageClass {
    Static,
    Extern
}


#[derive(Debug)]
pub enum VariableDeclaration {
    Declarant(
        String /* var_name */,
        Option<TypedExpression> /* initializer */,
        Type /* var_type */,
        Option<StorageClass>
    )
}


#[derive(Debug)]
pub enum FunctionDeclaration {
    Declarant(
        String /* func_name */,
        Vec<String> /* args */,
        Option<Block> /* body */,
        Type /* return_type */,
        Option<StorageClass>
    )
}


#[derive(Debug)]
pub enum Declaration {
    VarDecl(VariableDeclaration),
    FunDecl(FunctionDeclaration)
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
pub enum Program {
    ProgramDefinition(Vec<Declaration>)
}


/*                    GRAMMAR

<program>               ::= [ <function-declaration> ]*
<declaration>           ::= <variable-declaration> | <function-declaration>
<variable-declaration>  ::= <specifier>+ <identifier> [ "=" <exp> ] ";"
<function-declaration>  ::= <specifier>+ <identifier> "(" [<param-list>] )" ( <block> | ";" )
<specifier>             ::= <type-specifier> | "static" | "extern"
<type-specifier>        ::= "int" | "long"
<param-list>            ::= "" |
                            "void" |
                            <type-specifier> <identifier> ["," <type-specifier> <identifier>"]*
<block>                 ::= "{" [<block_item> *] "}"
<block_item>            ::= <statement>|<declaration>
<label>                 ::= <id> ":" | "case" <exp> ":" | "default" ":"
<statement>             ::= [<label> *] <unlbld_statement>
<unlbld_statement>      ::= ";" |
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
<for_init>              ::= <variable-declaration> | [<exp>]
<exp>                   ::= <factor> | <exp> <binop> <exp> | <exp> "?" <exp> ":" <exp>
<factor>                ::= <const> | <identifier> |
                            "(" <type-specifier>+ ")" <factor>
                            <unop> <factor> | "(" <exp> ")" |
                            <inc_dec> <factor> | <factor> <inc_dec> |
                            <identifier> "(" [ <argument-list > ] ")"
<argument-list>         ::= <exp> ["," <exp>]*
<unop>                  ::= "+"  | "-" | "~" | "!"
<inc_dec>               ::= "++" | "--"
<binop>                 ::= "-"  | "+" | "*" | "/" | "%" |
                            "<<" | ">>" | "|"  | "&"  | "^"  |
                            "&&" | "||" |
                            "==" | "!=" | "<"  | "<=" | ">"  | ">=" |
                            "="  | "+=" | "-=" | "*=" | "/=" | "%=" |
                            "|=" | "&=" |
                            "<<="| ">>="
<const>                 ::= <int> | <long>
<identifier>            ::= ? Token::Identifier ?
<int>                   ::= ? Token::IntConstant ?
<long>                  ::= ? Token::LongConstant ?

*/
