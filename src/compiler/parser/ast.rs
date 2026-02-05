pub enum Expression {
    IntConstant(i32)
}

pub enum Statement {
    Return(Expression)
}

pub enum FunctionDefinition {
    Function(String /* name */, Statement /* body */)
}

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