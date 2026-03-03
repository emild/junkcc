use log::{info, trace, warn, error};
pub mod ast;
mod pretty_print;
mod semantic_analyzer;

use crate::compiler::lexer::{self, Token};
use ast::*;


fn parse_function(l: &mut lexer::Lexer) -> Result<FunctionDefinition, String>
{
    let mut t  = l.get_token()?;

    //return type
    match t {
        Token::EOS => { return Err(format!("Expected return type, got end of file")); },
        _ => {
            if !is_type(&t) {
                return Err(format!("Expected return type, got {:?}", t));
            }
        }
    }

    //name
    t = l.get_token()?;
    let func_name = match t {
        Token::EOS => { return Err(format!("Expected function name, got end of file")); },
        Token::Identifier(id) => id,
        _ => { return Err(format!("Expected function name, got {:?}", t)); }
    };

    // (
    t = l.get_token()?;
    let _ = match t {
        Token::EOS => { return Err(format!("Expected '(', got end of file")); },
        Token::OpenParenthesis => (),
        _ => { return Err(format!("Expected '(', got {:?}", t)); }
    };


    //void or )
    t = l.get_token()?;
    let is_void = match t {
        Token::EOS => { return Err(format!("Expected arg type or '))', got end of file")); },
        Token::KwVoid => true,
        Token::CloseParenthesis => false,
        _ => { return Err(format!("Expected arg type or '))', got {:?}", t)); }
    };

    //)
    if is_void {
        t = l.get_token()?;
        let _ = match t {
            Token::EOS => { return Err(format!("Expected '))', got end of file")); },
            Token::CloseParenthesis => (),
            _ => { return Err(format!("Expected '))', got {:?}", t)); }
        };
    }

    //{
    t = l.get_token()?;
    let _ = match t {
        Token::EOS => { return Err(format!("Expected '{{', got end of file")); },
        Token::OpenBrace => (),
        _ => { return Err(format!("Expected '{{', got {:?}", t)); }
    };

    let mut block = vec![];
    loop {
        if let Ok(Token::CloseBrace) = l.peek_token() {
            break;
        }

        let block_item = parse_block_item(l)?;
        block.push(block_item);
    }

    // }
    t = l.get_token()?;
    match t {
        Token::EOS => { return Err(format!("Expected '}}', got end of file")); },
        Token::CloseBrace => (),
        _ => { return Err(format!("Expected '}}', got {:?}", t)); }
    };

    Ok(FunctionDefinition::Function(func_name, block))
}


fn is_type(t: &Token) -> bool
{
    match t {
        Token::KwInt => true,
        _ => false
    }
}


fn parse_block_item(l: &mut lexer::Lexer) -> Result<BlockItem, String>
{
    let t = l.peek_token()?;

    if is_type(&t) {
        let decl = parse_declaration(l)?;
        Ok(BlockItem::D(decl))
    }
    else {
        let stmnt = parse_statement(l)?;
        Ok(BlockItem::S(stmnt))
    }
}


fn parse_declaration(l: &mut lexer::Lexer) -> Result<Declaration, String>
{
    let t = l.get_token()?;

    match t {
        Token::EOS => { return Err(format!("Expected declaration, got end of file")); },
        _ => {
            if !is_type(&t) {
                return Err(format!("Expected type, got '{:?}'", t));
            }
        }
    }

    let t = l.get_token()?;
    let var_name = match t {
        Token::EOS => { return Err(format!("Expected ID, got end of file")); },
        Token::Identifier(id) => id,
        _ => { return Err(format!("Expected ID, got '{:?}'", t)); }
    };

    let t = l.peek_token()?;
    let initial_value = match t {
        Token::EqualSign => {
            l.get_token()?; //consume the '='
            let expr = parse_expression(l, 0)?;
            Some(expr)
        },
        _ => {
            None
        }
    };

    let t = l.get_token()?;
    match t {
        Token::Semicolon => { return Ok(Declaration::Declarant(var_name, initial_value)); }
        _ => { return Err(format!("Missing ';' at end of declaration (got '{:?}')", t)); }
    }
}


fn parse_statement(l: &mut lexer::Lexer) -> Result<Statement, String>
{
    let mut t = l.peek_token()?;
    let stmnt = match t {
        Token::EOS => { return Err(format!("Unexpected end of file")); },
        Token::KwReturn => {
            l.get_token()?; //Consume the return keyword
            let ex = parse_expression(l, 0)?;
            Statement::Return(ex)
        },
        Token::Semicolon => {
            Statement::Null
        },
        _ => {
            let ex = parse_expression(l, 0)?;
            Statement::Expr(ex)
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


fn parse_identifier(l: &mut lexer::Lexer) -> Result<String, String>
{
    let t = l.get_token()?;
    match t {
        Token::EOS => Err(format!("Expected identifier, got end of file")),
        Token::Identifier(id) => Ok(id),
        _ => Err(format!("Expected identifier, got '{:?}'", t))
    }
}


fn parse_unary_operator(l: &mut lexer::Lexer) -> Result<UnaryOperator, String>
{
    let t = l.get_token()?;

    let op = match t {
        Token::Plus            => UnaryOperator::Plus,
        Token::Minus           => UnaryOperator::Negate,
        Token::Tilde           => UnaryOperator::Complement,
        Token::ExclamationMark => UnaryOperator::LogicalNot,
        _ => { return Err(format!("Expected unary operator, got '{:?}'", t)); }
    };

    Ok(op)
}

fn parse_increment(l: &mut lexer::Lexer, pre: bool) -> Result<UnaryOperator, String>
{
    let t = l.get_token()?;

    match t {
        Token::Increment => {
            if pre {
                Ok(UnaryOperator::PreIncrement)
            }
            else {
                Ok(UnaryOperator::PostIncrement)
            }
        }
        _ => { return Err(format!("Expected '++', got '{:?}'", t)); }
    }
}


fn parse_decrement(l: &mut lexer::Lexer, pre: bool) -> Result<UnaryOperator, String>
{
    let t = l.get_token()?;

    match t {
        Token::Decrement => {
            if pre {
                Ok(UnaryOperator::PreDecrement)
            }
            else {
                Ok(UnaryOperator::PostDecrement)
            }
        }
        _ => { return Err(format!("Expected '--', got '{:?}'", t)); }
    }
}


fn convert_to_binary_operator(t: &Token) -> Option<BinaryOperator>
{
    let op = match t {
        Token::Plus              => BinaryOperator::Add,
        Token::Minus             => BinaryOperator::Subtract,
        Token::Asterisk          => BinaryOperator::Multiply,
        Token::Slash             => BinaryOperator::Divide,
        Token::Percent           => BinaryOperator::Remainder,
        Token::Ampersand         => BinaryOperator::BitwiseAnd,
        Token::VerticalBar       => BinaryOperator::BitwiseOr,
        Token::Caret             => BinaryOperator::BitwiseXor,
        Token::ShiftLeft         => BinaryOperator::ShiftLeft,
        Token::ShiftRight        => BinaryOperator::ShiftRight,
        Token::LogicalOr         => BinaryOperator::LogicalOr,
        Token::LogicalAnd        => BinaryOperator::LogicalAnd,
        Token::EqualTo           => BinaryOperator::Equal,
        Token::NotEqualTo        => BinaryOperator::NotEqual,
        Token::OpenAngleBracket  => BinaryOperator::LessThan,
        Token::LessOrEqual       => BinaryOperator::LessOrEqual,
        Token::CloseAngleBracket => BinaryOperator::GreaterThan,
        Token::GreaterOrEqual    => BinaryOperator::GreaterOrEqual,
        Token::EqualSign         => BinaryOperator::Assign,
        Token::AddAssign         => BinaryOperator::AddAssign,
        Token::SubAssign         => BinaryOperator::SubtractAssign,
        Token::MulAssign         => BinaryOperator::MultiplyAssign,
        Token::DivAssign         => BinaryOperator::DivideAssign,
        Token::ModAssign         => BinaryOperator::RemainderAssign,
        Token::BitwiseOrAssign   => BinaryOperator::BitwiseOrAssign,
        Token::BitwiseAndAssign  => BinaryOperator::BitwiseAndAssign,
        Token::BitwiseXorAssign  => BinaryOperator::BitwiseXorAssign,
        Token::ShiftLeftAssign   => BinaryOperator::ShiftLeftAssign,
        Token::ShiftRightAssign  => BinaryOperator::ShiftRightAssign,

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

    let mut expr = match t {
        Token::EOS => { return Err(format!("Expected factor, got end of file")); },
        Token::IntConstant(_) => {
            let c = parse_int_constant(l)?;
            Expression::IntConstant(c)
        },
        Token::Identifier(_) => {
            let var_name = parse_identifier(l)?;
            Expression::Var(var_name)
        }
        Token::Plus  |
        Token::Minus |
        Token::Tilde |
        Token::ExclamationMark => {
            let unary_operator = parse_unary_operator(l)?;
            let inner_expression = parse_factor(l)?;
            Expression::Unary(unary_operator, Box::new(inner_expression))
        },
        Token::Increment => {
            parse_increment(l, true)?;
            let inner_expression = parse_factor(l)?;
            Expression::PreIncrement(Box::new(inner_expression))
        },
        Token::Decrement => {
            parse_decrement(l, true)?;
            let inner_expression = parse_factor(l)?;
            Expression::PreDecrement(Box::new(inner_expression))
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

    let postfix_expr = loop {
        let next_t = l.peek_token()?;
        match next_t {
            Token::Increment => {
                parse_increment(l, false)?;
                expr = Expression::PostIncrement(Box::new(expr));
            },
            Token::Decrement => {
                parse_decrement(l, false)?;
                expr = Expression::PostDecrement(Box::new(expr));
            }

            _ => { break expr; }
        }
    };


    Ok(postfix_expr)

}




fn parse_expression(l: &mut lexer::Lexer, min_precedence: u32) -> Result<Expression, String>
{
    let mut left = parse_factor(l)?;
    let mut t = l.peek_token()?;

    loop {
        if let Some(binary_operator) = convert_to_binary_operator(&t) {
            let curr_precedence = binary_operator.precedence();
            if curr_precedence >= min_precedence {
                match binary_operator {
                BinaryOperator::Assign => {
                    parse_binary_operator(l)?;
                    let right = parse_expression(l, curr_precedence)?;
                    left = Expression::Assignment(Box::new(left), Box::new(right))
                },
                BinaryOperator::AddAssign |
                BinaryOperator::SubtractAssign |
                BinaryOperator::MultiplyAssign |
                BinaryOperator::DivideAssign |
                BinaryOperator::RemainderAssign |
                BinaryOperator::BitwiseAndAssign |
                BinaryOperator::BitwiseOrAssign |
                BinaryOperator::BitwiseXorAssign |
                BinaryOperator::ShiftLeftAssign |
                BinaryOperator::ShiftRightAssign
                => {
                    parse_binary_operator(l)?;
                    let right = parse_expression(l, curr_precedence)?;
                    left = Expression::CompoundAssignment(binary_operator, Box::new(left), Box::new(right))
                },
                _ => {
                    let binary_operator = parse_binary_operator(l)?;
                    let right = parse_expression(l, curr_precedence + 1)?;
                    left = Expression::Binary(binary_operator, Box::new(left), Box::new(right));
                }
            }
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

pub use semantic_analyzer::resolve_program;