pub mod ast;
mod pretty_print;

use ast::*;
use super::parser;


fn generate_code_for_return(expression: &parser::ast::Expression) -> Result<Vec<Instruction>, String>
{
    let mut instructions = vec![];

    let mut expr_instructions = match expression {
        parser::ast::Expression::IntConstant(c) => {
            vec![
                Instruction::Mov(Operand::Imm(*c), Operand::Register)
            ]
        },
        _ => {
            return Err(format!("Expected expression, got {:?}", expression));
        }
    };

    instructions.append(&mut expr_instructions);
    instructions.push(Instruction::Ret);

    Ok(instructions)
}


fn generate_code_for_statement(statement: &parser::ast::Statement) -> Result<Vec<Instruction>, String>
{
    let mut instructions = vec![];

    let mut new_instructions = match statement {
        parser::ast::Statement::Return(expr) => {
            let ret_instructions = generate_code_for_return(expr)?;

            ret_instructions
        }

        _ => {
            return Err(format!("Expected simple statement, got {:?}", statement));
        }
    };

    instructions.append(&mut new_instructions);

    Ok(instructions)
}


fn generate_code_for_function_definition(func_def: &parser::ast::FunctionDefinition) -> Result<FunctionDefinition, String>
{
    let (func_name, statement) = match func_def {
        parser::ast::FunctionDefinition::Function(f_name, stmnt) => {
            (f_name, stmnt)
        },
        _ => {
            return Err(format!("Expected function definitions, got {:?}", func_def));
        }
    };

    let instructions = generate_code_for_statement(&statement)?;

    Ok(FunctionDefinition::Function(func_name.clone(), instructions))
}

pub fn generate_code(program: &parser::ast::Program) -> Result<ast::Program, String>
{
    let func_def = match program {
        parser::ast::Program::ProgramDefinition(func) => {
            let fd = generate_code_for_function_definition(&func)?;
            fd
        },
        _ => {
            return Err(format!("Expected program definition, got {:?}", program));
        }
    };

    Ok(Program::ProgramDefinition(func_def))
}

pub use pretty_print::pretty_print_ast;
