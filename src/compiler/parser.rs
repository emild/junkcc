pub mod ast;
mod pretty_print;

use crate::compiler::lexer::{self, Token};
use ast::*;


fn parse_function(l: &mut lexer::Lexer) -> Result<FunctionDefinition, String>
{
    let mut t  = l.get_token()?;

    //return type
    let _ = match t {
        Token::EOS => { return Err(format!("Expected return type, got end of file")); },
        Token::KwInt => (),
        _ => { return Err(format!("Expected return type, got {:?}", t)); }
    };

    //name
    t = l.get_token()?;
    let func_name = match t {
        Token::EOS => { return Err(format!("Expected function name, got end of file")); },
        Token::Identifier(id) => id,
        _ => { return Err(format!("Expected function name, got {:?}", t)); }
    };

    //(
    t = l.get_token()?;
    let _ = match t {
        Token::EOS => { return Err(format!("Expected '(', got end of file")); },
        Token::OpenParenthesis => (),
        _ => { return Err(format!("Expected '(', got {:?}", t)); }
    };


    //void or )
    t = l.get_token()?;
    let is_void = match t {
        Token::EOS => { return Err(format!("Expected arg type or ')', got end of file")); },
        Token::KwVoid => true,
        Token::CloseParenthesis => false,
        _ => { return Err(format!("Expected arg type or ')', got {:?}", t)); }
    };

    //)
    if is_void {
        t = l.get_token()?;
        let _ = match t {
            Token::EOS => { return Err(format!("Expected ')', got end of file")); },
            Token::CloseParenthesis => (),
            _ => { return Err(format!("Expected ')', got {:?}", t)); }
        };
    }

    //{
    t = l.get_token()?;
    let _ = match t {
        Token::EOS => { return Err(format!("Expected '{{', got end of file")); },
        Token::OpenBrace => (),
        _ => { return Err(format!("Expected '{{', got {:?}", t)); }
    };

    let stmnt = parse_statement(l)?;

    //}
    t = l.get_token()?;
    let _ = match t {
        Token::EOS => { return Err(format!("Expected '}}', got end of file")); },
        Token::CloseBrace => (),
        _ => { return Err(format!("Expected '}}', got {:?}", t)); }
    };

    Ok(FunctionDefinition::Function(func_name, stmnt))
}


fn parse_statement(l: &mut lexer::Lexer) -> Result<Statement, String>
{
    let mut t = l.get_token()?;
    let stmnt = match t {
        Token::EOS => { return Err(format!("Unexpected end of file")); },
        Token::KwReturn => {
            let ex = parse_expression(l, 0)?;
            Statement::Return(ex)
        },
        _ => {
            return Err(format!("Expected statement, got {:?}", t));
        }
    };

    t = l.get_token()?;
    match t {
        Token::EOS => { return Err(format!("Expected ';', got end of file")); },
        Token::Semicolon => {},
        _ => { return Err(format!("Expected ';', got {:?}", t)); }
    };

    Ok(stmnt)
}

fn parse_int_constant(l: &mut lexer::Lexer) -> Result<i32, String>
{
    let t = l.get_token()?;
    match t {
        Token::EOS => Err(format!("Expected int constant, got end of file")),
        Token::IntConstant(c) => Ok(c),
        _ => Err(format!("Expected int constant, got {:?}", t))
    }
}

fn parse_unary_operator(l: &mut lexer::Lexer) -> Result<UnaryOperator, String>
{
    let t = l.get_token()?;

    let op = match t {
        Token::Plus  => UnaryOperator::Plus,
        Token::Minus => UnaryOperator::Negate,
        Token::Tilde => UnaryOperator::Complement,
        _ => { return Err(format!("Expected unary operator, got '{:?}'", t)); }
    };

    Ok(op)
}

fn convert_to_binary_operator(t: &Token) -> Option<BinaryOperator>
{
    let op = match t {
        Token::Plus        => BinaryOperator::Add,
        Token::Minus       => BinaryOperator::Subtract,
        Token::Asterisk    => BinaryOperator::Multiply,
        Token::Slash       => BinaryOperator::Divide,
        Token::Percent     => BinaryOperator::Remainder,
        Token::Ampersand   => BinaryOperator::BitwiseAnd,
        Token::VerticalBar => BinaryOperator::BitwiseOr,
        Token::Caret       => BinaryOperator::BitwiseXor,
        Token::ShiftLeft   => BinaryOperator::ShiftLeft,
        Token::ShiftRight  => BinaryOperator::ShiftRight,
        _ => { return None; }
    };

    Some(op)
}


fn parse_binary_operator(l: &mut lexer::Lexer) -> Result<BinaryOperator, String>
{
    let t = l.get_token()?;

    if let Some(op) = convert_to_binary_operator(&t) {
        return Ok(op);
    }

    return Err(format!("Expected binary operator, got '{:?}'", t));
}


fn parse_factor(l: &mut lexer::Lexer) -> Result<Expression, String>
{
    let t = l.peek_token()?;

    let expr = match t {
        Token::EOS => { return Err(format!("Expected factor, got end of file")); },
        Token::IntConstant(_) => {
            let c = parse_int_constant(l)?;
            Expression::IntConstant(c)
        },
        Token::Plus  |
        Token::Minus |
        Token::Tilde => {
            let unary_operator = parse_unary_operator(l)?;
            let inner_expression = parse_factor(l)?;
            Expression::Unary(unary_operator, Box::new(inner_expression))
        },
        Token::OpenParenthesis => {
            let mut t = l.get_token()?;
            match t {
                Token::OpenParenthesis => (),
                _ => { return Err(format!("Expected '(' got '{:?}'", t)); }
            };

            let inner_expression = parse_expression(l, 0)?;

            t = l.get_token()?;
            match t {
                Token::CloseParenthesis => (),
                _ => { return Err(format!("Expected ')' got '{:?}'", t)); }
            };

            inner_expression
        },
        _ => { return Err(format!("Malformed factor, got {:?}", t)); }
    };

    Ok(expr)

}



fn parse_expression(l: &mut lexer::Lexer, min_precedence: u32) -> Result<Expression, String>
{
    let mut left = parse_factor(l)?;
    let mut t = l.peek_token()?;

    loop {
        if let Some(binary_operator) = convert_to_binary_operator(&t) {
            let curr_precedence = binary_operator.precedence();
            if curr_precedence >= min_precedence {
                let binary_operator = parse_binary_operator(l)?;
                let right = parse_expression(l, curr_precedence + 1)?;
                left = Expression::Binary(binary_operator, Box::new(left), Box::new(right));
                t = l.peek_token()?;

                continue;
            }
        }

        break;
    }
    Ok(left)
}


pub fn parse_program(l: &mut lexer::Lexer) -> Result<Program, String>
{
    let f = parse_function(l)?;

    let t = l.get_token()?;

    match t {
        Token::EOS => Ok(Program::ProgramDefinition(f)),
        _ => Err(format!("Trailing garbage: {:?}", t))
    }
}

pub use pretty_print::pretty_print_ast;
