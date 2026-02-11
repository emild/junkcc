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
<unop>              ::= "+" | "-" | "~"
<binop>             ::= "-" | "+" | "*" | "/" | "%"
<identifier>        ::= ? Token::Identifier ?
<int>               ::= ? Token::IntConstant ?

*/