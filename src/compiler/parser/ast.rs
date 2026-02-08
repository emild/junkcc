#[derive(Debug)]
pub enum Expression {
    IntConstant(i32),
    Unary(UnaryOperator, Box<Expression>)
}

#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate
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
<exp>               ::= <int> | <unop> <exp> | "(" <exp> ")"
<unop>              ::= "-" | "~"
<identifier>        ::= ? Token::Identifier ?
<int>               ::= ? Token::IntConstant ?

*/