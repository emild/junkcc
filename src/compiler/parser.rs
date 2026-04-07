use std::collections::HashMap;

use log::{info, trace, warn, error};
pub mod ast;
mod pretty_print;
mod semantic_analyzer;


use crate::compiler::lexer::{self, Token};
use ast::*;

fn is_type(t: &Token) -> bool
{
    match t {
        Token::KwInt => true,
        _ => false
    }
}


fn parse_param_list(l: &mut lexer::Lexer) -> Result<Vec<String>, String>
{
 //void or )
    let mut param_list = vec![];
    let t = l.peek_token()?;
    let is_void = match t {
        Token::EOS => { return Err(format!("Expected parameter type or ')', got end of file")); },
        Token::KwVoid => { l.get_token()?; true},
        Token::CloseParenthesis => true,
        _ => false
    };

    if is_void {
        return Ok(param_list);
    }

    loop {

        let mut t = l.peek_token()?;
        match t {
            Token::EOS => { return Err(format!("Expected parameter type or ')', got end of file")); },
            _ => {
                if !is_type(&t) {
                    return Err(format!("Expected parameter type, got '{:?}'", t));
                }
                l.get_token()?;
            }
        };


        t = l.peek_token()?;
        let param_name = match t {
            Token::EOS => { return Err(format!("Expected parameter name, got end of file")); },
            Token::Identifier(param_name) => { l.get_token()?; param_name },
            _ => { return Err(format!("Expected parameter name, got '{:?}'", t)); }
        };


        t = l.peek_token()?;
        match t {
            Token::EOS => { return Err(format!("Expected ',' or ')', got end of file")); },
            Token::Comma => { l.get_token()?; param_list.push(param_name); continue; },
            Token::CloseParenthesis => { param_list.push(param_name); break; }
            _ => { return Err(format!("Expected ',' or ')', got '{:?}'", t)); }
        };
    }

    Ok(param_list)

}


fn parse_function_declaration(l: &mut lexer::Lexer) -> Result<FunctionDeclaration, String>
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
    check_open_paren(l)?;

    let param_list = parse_param_list(l)?;

    check_close_paren(l)?;

    t = l.peek_token()?;

    match t {
        Token::Semicolon => {
            l.get_token()?;
            Ok(FunctionDeclaration::Declarant(func_name, param_list, None))
        },
        Token::OpenBrace => {
            let block = parse_block(l)?;
            Ok(FunctionDeclaration::Declarant(func_name, param_list, Some(block)))
        },
        Token::EOS => { return Err(format!("Unexpected end of file at the end of function declaration")); },
        _ => { return Err(format!("Expected ';' or '{{', got '{:?}", t)); }
    }

}


fn parse_variable_declaration(l: &mut lexer::Lexer) -> Result<VariableDeclaration, String>
{
    let t = l.get_token()?;

    match t {
        Token::EOS => { return Err(format!("Expected variable declaration, got end of file")); },
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
        Token::Semicolon => { return Ok(VariableDeclaration::Declarant(var_name, initial_value)); }
        _ => { return Err(format!("Missing ';' at end of declaration (got '{:?}')", t)); }
    }
}


fn parse_declaration(l: &mut lexer::Lexer) -> Result<Declaration, String>
{
   let mut t = l.get_token()?;

    let dtype = match t {
        Token::EOS => { return Err(format!("Expected type, got end of file")); },
        _ => {
            if !is_type(&t) {
                return Err(format!("Expected type, got '{:?}'", t));
            }
            t
        }
    };

    t = l.get_token()?;
    let dname = match t {
        Token::EOS => { return Err(format!("Expected ID, got end of file")); },
        Token::Identifier(id) => id,
        _ => { return Err(format!("Expected ID, got '{:?}'", t)); }
    };

    t = l.peek_token()?;
    match t {
        Token::OpenParenthesis => {
            l.putback_token(Token::Identifier(dname))?;
            l.putback_token(dtype)?;
            let func_dec = parse_function_declaration(l)?;
            Ok(Declaration::FunDecl(func_dec))
        },
        Token::EqualSign |
        Token::Semicolon => {
            l.putback_token(Token::Identifier(dname))?;
            l.putback_token(dtype)?;
            let var_dec = parse_variable_declaration(l)?;
            Ok(Declaration::VarDecl(var_dec))
        },
        _ => {
            return Err(format!("Expected declaration, got '{:?}'", t));
        }
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


fn parse_block(l: &mut lexer::Lexer) -> Result<Block, String>
{
    //{
    let mut t = l.get_token()?;
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

    Ok(Block::Blk(block))
}


fn check_semicolon(l: &mut lexer::Lexer) -> Result<(), String>
{
    let t = l.get_token()?;
    match t {
        Token::EOS => { return Err(format!("Expected ';', got end of file")); },
        Token::Semicolon => {},
        _ => { return Err(format!("Expected ';', got {:?}", t)); }
    };

    Ok(())
}

fn check_open_paren(l: &mut lexer::Lexer) -> Result<(), String>
{
    let t = l.get_token()?;
    match t {
        Token::EOS => { return Err(format!("Expected '(', got end of file")); },
        Token::OpenParenthesis => {},
        _ => { return Err(format!("Expected '(', got {:?}", t)); }
    };

    Ok(())
}


fn check_close_paren(l: &mut lexer::Lexer) -> Result<(), String>
{
    let t = l.get_token()?;
    match t {
        Token::EOS => { return Err(format!("Expected ')', got end of file")); },
        Token::CloseParenthesis => {},
        _ => { return Err(format!("Expected ')', got {:?}", t)); }
    };

    Ok(())
}


fn parse_statement(l: &mut lexer::Lexer) -> Result<Statement, String>
{
    let mut labels = vec![];

    loop {
        let t = l.get_token()?;
        match t {
            Token::KwDefault => {
                let colon = l.get_token()?;
                if colon != Token::Colon {
                    return Err(format!("Error: expected ':' after default, got '{:?}'", colon));
                }

                trace!("FOUND DEFAULT: LABEL");

                labels.push(Label::Default);
            },
            Token::Identifier(label) => {
                let colon = l.get_token()?;
                if colon != Token::Colon {
                    l.putback_token(colon)?;
                    l.putback_token(Token::Identifier(label))?;
                    break;
                }

                trace!("FOUND GOTO LABEL: '{}'", label);

                labels.push(Label::Goto(label));
            },
            Token::KwCase => {
                let expr = parse_expression(l, 0)?;
                trace!("Found CASE LABEL");
                labels.push(Label::Case(expr));
                let colon = l.get_token()?;
                if colon != Token::Colon {
                    return Err(format!("Expected ':', got '{:?}'", colon));
                }
            },
            _ => {
                l.putback_token(t)?;
                break;
            }
        }
    }

    let unlabeled_statement = parse_unlabeled_statement(l)?;
    if labels.is_empty() {
        Ok(Statement::Stmnt(None, unlabeled_statement))
    }
    else {
        Ok(Statement::Stmnt(Some(labels), unlabeled_statement))
    }

}


fn parse_condition(l: &mut lexer::Lexer) -> Result<Expression, String>
{
    check_open_paren(l)?;
    let cond = parse_expression(l, 0)?;
    check_close_paren(l)?;

    Ok(cond)
}


fn parse_for_init(l: &mut lexer::Lexer) -> Result<ForInit, String>
{
    let t = l.peek_token()?;

    let for_init = match t {
        Token::Semicolon => {
            check_semicolon(l)?;
            ForInit::InitExp(None)
        },
        _ => {
            if is_type(&t) {
                let decl = parse_variable_declaration(l)?;
                ForInit::InitDecl(decl)
            }
            else {
                let expr = parse_expression(l, 0)?;
                check_semicolon(l)?;
                ForInit::InitExp(Some(expr))
            }
        }
    };

    Ok(for_init)
}


fn parse_unlabeled_statement(l: &mut lexer::Lexer) -> Result<UnlabeledStatement, String>
{
    let t = l.peek_token()?;

    let stmnt = match t {
        Token::EOS => { return Err(format!("Unexpected end of file")); },
        Token::KwReturn => {
            l.get_token()?; //Consume the return keyword
            let ex = parse_expression(l, 0)?;
            check_semicolon(l)?;
            UnlabeledStatement::Return(ex)
        },
        Token::KwGoto => {
            l.get_token()?; //Consume the goto keyword
            let t = l.get_token()?;
            let label = match t {
                Token::Identifier(label) => label,
                _ => { return Err(format!("Expected label, got {:?}", t)); }
            };
            check_semicolon(l)?;

            UnlabeledStatement::Goto(label)
        },
        Token::KwIf => {
            l.get_token()?; //Consume the if keyword
            let cond = parse_condition(l)?;
            let then_stmnt = parse_statement(l)?;
            let else_stmnt = match l.peek_token() {
                Err(e) => { return Err(e); }
                Ok(Token::EOS) => { return Err(format!("Error: Unexpected end of file after if")); },
                Ok(Token::KwElse) => {
                    l.get_token()?; //Consume the else token
                    let else_stmnt = parse_statement(l)?;
                    Some(Box::new(else_stmnt))
                },
                _ => None
            };
            UnlabeledStatement::If(cond, Box::new(then_stmnt), else_stmnt)
        },
        Token::OpenBrace => {
            let block = parse_block(l)?;
            UnlabeledStatement::Compound(block)
        },
        Token::KwBreak => {
            l.get_token()?; //Consume the break keyword
            check_semicolon(l)?;
            UnlabeledStatement::Break(None, None)
        },
        Token::KwContinue => {
            l.get_token()?; //Consume the continue keyword
            check_semicolon(l)?;
            UnlabeledStatement::Continue(None)
        },
        Token::KwWhile => {
            l.get_token()?; //Consume the while keyword
            let cond = parse_condition(l)?;
            let body = parse_statement(l)?;
            UnlabeledStatement::While(cond, Box::new(body), None)
        },
        Token::KwDo => {
            l.get_token()?; //Consume the do keyword
            let body = parse_statement(l)?;
            let t = l.get_token()?;
            if t != Token::KwWhile {
                return Err(format!("Expected 'while', got '{:?}'", t));
            }
            let cond = parse_expression(l, 0)?;
            check_semicolon(l)?;
            UnlabeledStatement::DoWhile(Box::new(body), cond, None)
        },
        Token::KwFor => {
            l.get_token()?; //Consume the for keyword
            check_open_paren(l)?;
            let for_init = parse_for_init(l)?;
            let mut t = l.peek_token()?;
            let mut cond = None;
            if t != Token::Semicolon {
                let expr = parse_expression(l, 0)?;
                cond = Some(expr);
            }
            check_semicolon(l)?;
            t = l.peek_token()?;
            let mut post = None;
            if t != Token::CloseParenthesis {
                let expr = parse_expression(l, 0)?;
                post = Some(expr);
            }
            check_close_paren(l)?;
            let body = parse_statement(l)?;
            UnlabeledStatement::For(for_init, cond, post, Box::new(body), None)
        },
        Token::KwSwitch => {
            l.get_token()?; //Consume the switch keyword
            check_open_paren(l)?;
            let cond = parse_expression(l, 0)?;
            check_close_paren(l)?;
            let body = parse_statement(l)?;
            UnlabeledStatement::Switch(cond, Box::new(body), None, HashMap::new(), None)
        }
        Token::Semicolon => {
            check_semicolon(l)?;
            UnlabeledStatement::Null
        },
        _ => {
            let ex = parse_expression(l, 0)?;
            check_semicolon(l)?;
            UnlabeledStatement::Expr(ex)
        }
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
        Token::QuestionMark      => BinaryOperator::ConditionalMiddle,

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



fn parse_function_argument_list(l: &mut lexer::Lexer) -> Result<Vec<Expression>, String>
{
    let mut func_args = vec![];

    let t = l.peek_token()?;
    if t == Token::CloseParenthesis {
        return Ok(func_args);
    }


    loop {

        let func_arg = parse_expression(l, 0)?;

        let t = l.peek_token()?;
        match t {
            Token::CloseParenthesis => {
                func_args.push(func_arg);
                break;
            },
            Token::Comma => {
                l.get_token()?;
                func_args.push(func_arg);
                continue;
            },
            _ => { return Err(format!("Expected ',' or ')', got: '{:?}'", t)); }
        }
    }

    Ok(func_args)
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
            let name = parse_identifier(l)?;
            let t = l.peek_token()?;
            match t {
                Token::OpenParenthesis => {
                    check_open_paren(l)?;
                    let func_args = parse_function_argument_list(l)?;
                    check_close_paren(l)?;

                    Expression::FunctionCall(name, func_args)
                },
                _ => Expression::Var(name)
            }
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
        _ => { return Err(format!("Malformed factor, got '{:?}'", t)); }
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


fn parse_conditional_middle(l: &mut lexer::Lexer) -> Result<Expression, String>
{
    let mut t = l.get_token()?; //consume '?'
    if t != Token::QuestionMark {
        return Err(format!("Failed to read '?'"));
    }

    let true_exp = parse_expression(l, 0)?;
    t = l.get_token()?;
    if t != Token::Colon {
        return Err(format!("Expected ':' after true_expression operand of conditional operator"));
    }

    Ok(true_exp)
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
                    BinaryOperator::ConditionalMiddle => {
                        let true_exp = parse_conditional_middle(l)?;
                        let false_exp = parse_expression(l, curr_precedence)?;
                        left = Expression::Conditional(Box::new(left), Box::new(true_exp), Box::new(false_exp));
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
    let mut funcs = vec![];

    loop {
        let t = l.peek_token()?;
        match t {
            Token::EOS => { break; },
            _ => {
                let f = parse_function_declaration(l)?;
                funcs.push(f);
            }
        }
    }

    Ok(Program::ProgramDefinition(funcs))
}

pub use pretty_print::pretty_print_ast;
pub use semantic_analyzer::semantic_analysis;
pub use semantic_analyzer::Type;
