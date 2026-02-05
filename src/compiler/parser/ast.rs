#[derive(Debug)]
pub enum Expression {
    IntConstant(i32)
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
<exp>               ::= <int>
<identifier>        ::= ? Token::Identifier ?
<int>               ::= ? Token::IntConstant ?

*/