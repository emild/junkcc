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


fn generate_code_for_tacky_ret_instruction(
    ret_val: &tacky::ast::Val,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let ret_val_src = convert_tacky_value_to_operand(&ret_val)?;
    instructions.push(Instruction::Mov(ret_val_src, Operand::Reg(Register::AX)));
    instructions.push(Instruction::Ret);

    Ok(())
}


fn generate_code_for_tacky_unary_instruction(
    tacky_unary_op: &tacky::ast::UnaryOperator,
    src: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let unary_op_src = convert_tacky_value_to_operand(src)?;
    let unary_op_dst = convert_tacky_value_to_operand(dst)?;
    instructions.push(Instruction::Mov(unary_op_src, unary_op_dst.clone()));
    let unary_op_instruction = match tacky_unary_op {
        tacky::ast::UnaryOperator::Complement   => Instruction::Unary(UnaryOperator::Not, unary_op_dst),
        tacky::ast::UnaryOperator::Negate       => Instruction::Unary(UnaryOperator::Neg, unary_op_dst),
        tacky::ast::UnaryOperator::Plus         => { return Ok(()); }
    };
    instructions.push(unary_op_instruction);

    Ok(())
}



fn generate_code_for_remainder_instruction(
    src1: &Operand,
    src2: &Operand,
    dst: &Operand,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let mut div_instructions = vec![
        Instruction::Mov(src1.clone(), Operand::Reg(Register::AX)),
        Instruction::Cdq,
        Instruction::Idiv(src2.clone()),
        Instruction::Mov(Operand::Reg(Register::DX), dst.clone())
    ];

    instructions.append(&mut div_instructions);

    Ok(())
}


fn generate_code_for_divide_instruction(
    src1: &Operand,
    src2: &Operand,
    dst: &Operand,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let mut div_instructions = vec![
        Instruction::Mov(src1.clone(), Operand::Reg(Register::AX)),
        Instruction::Cdq,
        Instruction::Idiv(src2.clone()),
        Instruction::Mov(Operand::Reg(Register::AX), dst.clone())
    ];

    instructions.append(&mut div_instructions);

    Ok(())
}



//add, sub, and mul
fn generate_code_for_binary_instruction(
    bin_op: &BinaryOperator,
    src1: &Operand,
    src2: &Operand,
    dst: &Operand,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    instructions.push(Instruction::Mov(src1.clone(), dst.clone()));
    instructions.push(Instruction::Binary(bin_op.clone(), src2.clone(), dst.clone()));

    Ok(())
}


fn generate_code_for_tacky_binary_instruction(
    bin_op: &tacky::ast::BinaryOperator,
    src1: &tacky::ast::Val,
    src2: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1 = convert_tacky_value_to_operand(src1)?;
    let src2 = convert_tacky_value_to_operand(src2)?;
    let dst  = convert_tacky_value_to_operand(dst)?;

    let result = match bin_op {
        tacky::ast::BinaryOperator::Add => generate_code_for_binary_instruction(&BinaryOperator::Add, &src1, &src2, &dst, instructions)?,
        tacky::ast::BinaryOperator::Subtract => generate_code_for_binary_instruction(&BinaryOperator::Sub, &src1, &src2, &dst, instructions)?,
        tacky::ast::BinaryOperator::Multiply => generate_code_for_binary_instruction(&BinaryOperator::Mul, &src1, &src2, &dst, instructions)?,
        tacky::ast::BinaryOperator::Divide => generate_code_for_divide_instruction(&src1, &src2, &dst, instructions)?,
        tacky::ast::BinaryOperator::Remainder => generate_code_for_remainder_instruction(&src1, &src2, &dst, instructions)?
    };

    Ok(result)
}


fn generate_code_for_tacky_instructions(tacky_instructions: &Vec<tacky::ast::Instruction>) -> Result<Vec<Instruction>, String>
{
    let mut instructions = vec![];

    for tacky_inst in tacky_instructions {
        match tacky_inst {
            tacky::ast::Instruction::Return(ret_val) => {
                generate_code_for_tacky_ret_instruction(ret_val, &mut instructions)?;
            },
            tacky::ast::Instruction::Unary(tacky_unary_op, src, dst) => {
                generate_code_for_tacky_unary_instruction(tacky_unary_op, &src, &dst, &mut instructions)?;
            },
            tacky::ast::Instruction::Binary(tacky_binary_op, src1, src2, dst) => {
                 generate_code_for_tacky_binary_instruction(tacky_binary_op, &src1, &src2, &dst, &mut instructions)?;
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