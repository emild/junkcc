pub mod ast;
use ast::*;

use super::parser;


fn pretty_print_operand(op: &Operand)
{
    match op {
        Operand::Register => {
            print!("REG");
        },
        Operand::Imm(c) => {
            print!("IMM({})", c);
        }
        _ => {}
    };
}


fn pretty_print_instructions(instructions: &Vec<Instruction>, indent: usize)
{
    for ins in instructions {
        match ins {
            Instruction::Mov(src, dest ) => {
                print!("{}mov src=", " ".repeat(indent));
                pretty_print_operand(&src);
                print!(", dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Ret => {
                println!("{}ret", " ".repeat(indent))
            },
            _ => {}
        };
    }
}

fn pretty_print_function(f: &FunctionDefinition, indent: usize)
{
    match f {
        FunctionDefinition::Function(func_name, instructions) => {
            println!("{}Function(", " ".repeat(indent));
            println!("{}name={func_name}", " ".repeat(indent + 4));
            println!("{}body=(", " ".repeat(indent + 4));
            pretty_print_instructions(instructions, indent + 8);
            println!("{})", " ".repeat(indent + 4));
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


pub fn pretty_print_ast(program: &Program)
{
    pretty_print_program(&program, 0);
}



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