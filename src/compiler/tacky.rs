use std::sync::atomic::{AtomicUsize, Ordering};

pub mod ast;
mod pretty_print;

use ast::*;
use super::parser;


static TMP_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);

fn make_temp_name() -> String
{
    let index = TMP_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let temp_name = format!("tmp.{}", index);

    temp_name
}

fn emit_tacky_unary_operator(unop: &parser::ast::UnaryOperator) -> Result<UnaryOperator, String>
{
    match unop {
        parser::ast::UnaryOperator::Plus => Ok(UnaryOperator::Plus),
        parser::ast::UnaryOperator::Complement => Ok(UnaryOperator::Complement),
        parser::ast::UnaryOperator::Negate => Ok(UnaryOperator::Negate),
        _ => { return Err(format!("TACKY Conversion: Expected unary operator, got '{:?}'", unop)); }
    }
}


fn emit_tacky_binary_operator(binop: &parser::ast::BinaryOperator) -> Result<BinaryOperator, String>
{
    match binop {
        parser::ast::BinaryOperator::Add => Ok(BinaryOperator::Add),
        parser::ast::BinaryOperator::Subtract => Ok(BinaryOperator::Subtract),
        parser::ast::BinaryOperator::Multiply => Ok(BinaryOperator::Multiply),
        parser::ast::BinaryOperator::Divide => Ok(BinaryOperator::Divide),
        parser::ast::BinaryOperator::Remainder => Ok(BinaryOperator::Remainder),
        _ => { return Err(format!("TACKY Conversion: Expected binary operator, got '{:?}'", binop)); }
    }
}


fn emit_tacky_expression(expr: &parser::ast::Expression, instructions: &mut Vec<Instruction>) -> Result<Val, String>
{
    let val = match expr {
        parser::ast::Expression::IntConstant(c) => {
            Val::IntConstant(*c)
        },
        parser::ast::Expression::Unary(unop, inner_expression) => {
            let src = emit_tacky_expression(&inner_expression, instructions)?;
            let dst_name = make_temp_name();
            let dst = Val::Var(dst_name);
            let tacky_un_op = emit_tacky_unary_operator(unop)?;
            instructions.push(Instruction::Unary(tacky_un_op, src, dst.clone()));

            dst
        },
        parser::ast::Expression::Binary(binop, expr1, expr2 ) => {
            let src1 = emit_tacky_expression(expr1, instructions)?;
            let src2 = emit_tacky_expression(expr2, instructions)?;
            let dst_name = make_temp_name();
            let dst = Val::Var(dst_name);
            let tacky_bin_op = emit_tacky_binary_operator(binop)?;
            instructions.push(Instruction::Binary(tacky_bin_op, src1, src2, dst.clone()));

            dst
        }

        _ => { return Err(format!("TACKY Conversion: expected expression, got '{:?}'", expr)); }
    };

    Ok(val)
}


fn emit_tacky_statement(stmnt: &parser::ast::Statement) -> Result<Vec<Instruction>, String>
{
    match stmnt {
        parser::ast::Statement::Return(expr) => {
            let mut instructions = vec![];
            let val = emit_tacky_expression(&expr, &mut instructions)?;
            instructions.push(Instruction::Return(val));

            Ok(instructions)
        },

        _ => { return Err(format!("TACKY Conversion: expected statement, got '{:?}'", stmnt))}
    }
}


fn emit_tacky_function_definition(func_def: &parser::ast::FunctionDefinition) -> Result<FunctionDefinition, String>
{
    match func_def {
        parser::ast::FunctionDefinition::Function(name, stmnt) => {
            let instructions = emit_tacky_statement(&stmnt)?;
            Ok(FunctionDefinition::Function(name.clone(), instructions))
        },
        _ => { return Err(format!("TACKY Conversion: expected function definition, got '{:?}'", *func_def)); }
    }
}


pub fn emit_tacky_program(program: &parser::ast::Program) -> Result<Program, String>
{
    match program {
        parser::ast::Program::ProgramDefinition(func_def) => {
            let tacky_func_def = emit_tacky_function_definition(&func_def)?;
            Ok(Program::ProgramDefinition(tacky_func_def))
        },
        _ => { return Err(format!("Tacky conversion: expected ProgramDefinition, got '{:?}'", program)); }
    }
}



pub use self::emit_tacky_program as generate_tacky_ast;
pub use pretty_print::pretty_print_tacky_ast;