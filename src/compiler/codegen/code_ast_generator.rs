use super::super::tacky;
use super::ast::*;

fn convert_tacky_value_to_operand(val: &tacky::ast::Val) -> Result<Operand, String>
{
    match val {
        tacky::ast::Val::IntConstant(c) => Ok(Operand::Imm(*c)),
        tacky::ast::Val::Var(var_name) => Ok(Operand::Pseudo(var_name.clone())),
        _ => { return Err(format!("Tacky value to operand conversion error: cannot convert '{:?}'", val)); }

    }
}

fn generate_code_for_tacky_instructions(tacky_instructions: &Vec<tacky::ast::Instruction>) -> Result<Vec<Instruction>, String>
{
    let mut instructions = vec![];

    for tacky_inst in tacky_instructions {
        match tacky_inst {
            tacky::ast::Instruction::Return(ret_val) => {
                let ret_val_src = convert_tacky_value_to_operand(&ret_val)?;
                instructions.push(Instruction::Mov(ret_val_src, Operand::Reg(Register::AX)));
                instructions.push(Instruction::Ret);
            },
            tacky::ast::Instruction::Unary(tacky_unary_op, src, dst) => {
                let unary_op_src = convert_tacky_value_to_operand(&src)?;
                let unary_op_dst = convert_tacky_value_to_operand(&dst)?;
                instructions.push(Instruction::Mov(unary_op_src, unary_op_dst.clone()));
                let unary_op_instruction = match tacky_unary_op {
                    tacky::ast::UnaryOperator::Complement   => Instruction::Unary(UnaryOperator::Not, unary_op_dst),
                    tacky::ast::UnaryOperator::Negate       => Instruction::Unary(UnaryOperator::Neg, unary_op_dst),
                    _ => { return Err(format!("Tacky instruction conversion error: cannot convert '{:?}'", tacky_unary_op)); }
                };
                instructions.push(unary_op_instruction);
            },
            _ => { panic!("Invalid TACKY Instruction: {:?}", tacky_inst); }
        };
    }

    Ok(instructions)
}


fn generate_code_for_function_definition(func_def: &tacky::ast::FunctionDefinition) -> Result<FunctionDefinition, String>
{
    let (func_name, tacky_instructions) = match func_def {
        tacky::ast::FunctionDefinition::Function(f_name, tacky_instructions) => {
            (f_name, tacky_instructions)
        },
        _ => {
            return Err(format!("Expected function definitions, got {:?}", func_def));
        }
    };

    let instructions = generate_code_for_tacky_instructions(&tacky_instructions)?;

    Ok(FunctionDefinition::Function(func_name.clone(), instructions))
}

pub fn generate_code(program: &tacky::ast::Program) -> Result<Program, String>
{
    let func_def = match program {
        tacky::ast::Program::ProgramDefinition(func) => {
            let fd = generate_code_for_function_definition(&func)?;
            fd
        },
        _ => {
            return Err(format!("Expected program definition, got {:?}", program));
        }
    };

    Ok(Program::ProgramDefinition(func_def))
}