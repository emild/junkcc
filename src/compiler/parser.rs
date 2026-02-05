pub mod ast;

use crate::compiler::lexer::{self, Token};

use ast::*;

fn pretty_print_expression(expr: &Expression, indent: usize)
{
    match expr {
        Expression::IntConstant(c) => {
            println!("{}Constant({})", " ".repeat(indent), c);
        }
    }
}

fn pretty_print_statement(s: &Statement, indent: usize)
{
    match s {
        Statement::Return(expr) => {
            println!("{}Return(", " ".repeat(indent));
            pretty_print_expression(&expr, indent + 4);
            println!("{})", " ".repeat(indent));
        }
    }
}

fn pretty_print_function(f: &FunctionDefinition, indent: usize)
{
    match f {
        FunctionDefinition::Function(func_name, stmt) => {
            println!("{}Function(", " ".repeat(indent));
            println!("{}name={func_name}", " ".repeat(indent + 4));
            print!("{}body=(", " ".repeat(indent + 4));
            pretty_print_statement(&stmt, indent + 4);
            println!("{})", " ".repeat(indent));
        },
        _ => ()
    }
}


fn pretty_print_program(p: &Program, indent: usize)
{
    println!("{}Program(", " ".repeat(indent));
    match p {
        Program::ProgramDefinition(f) => pretty_print_function(&f, indent + 4),
        _ => ()
    };

    println!("{})", " ".repeat(indent));
}


pub fn pretty_print_ast(p: &Program)
{
    pretty_print_program(p, 0);
}





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
            let ex = parse_expression(l)?;
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


fn parse_expression(l: &mut lexer::Lexer) -> Result<Expression, String>
{
    let t = l.get_token()?;

    let expr = match t {
        Token::EOS => { return Err(format!("Expected expression, got end of file")); },
        Token::IntConstant(c) => Expression::IntConstant(c),
        _ => { return Err(format!("Expected expression, got {:?}", t)); }
    };

    Ok(expr)
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