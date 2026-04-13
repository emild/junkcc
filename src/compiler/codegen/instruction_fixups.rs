use std::ops::Not;

use super::ast::*;

fn fixup_instruction_operands(instruction: &Instruction) -> Option<Vec<Instruction>>
{
    match instruction {
        Instruction::Mov(Operand::Stack(src_idx), Operand::Stack(dst_idx))=> {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Mov(Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Mov(Operand::Stack(src_idx), Operand::Data(dst_var_name)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Mov(Operand::Reg(Register::R10), Operand::Data(dst_var_name.clone()))
            ])
        },
        Instruction::Mov(Operand::Data(src_var_name), Operand::Stack(dst_idx))=> {
            Some(vec![
                Instruction::Mov(Operand::Data(src_var_name.clone()), Operand::Reg(Register::R10)),
                Instruction::Mov(Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Mov(Operand::Data(src_var_name), Operand::Data(dst_var_name)) => {
            Some(vec![
                Instruction::Mov(Operand::Data(src_var_name.clone()),  Operand::Reg(Register::R10)),
                Instruction::Mov(Operand::Reg(Register::R10), Operand::Data(dst_var_name.clone()))
            ])
        },

        Instruction::Binary(BinaryOperator::Shl, src, dst) => {
            let result = match src {
                Operand::Reg(Register::CX) => None,
                _ => Some(vec![
                        Instruction::Mov(src.clone(), Operand::Reg(Register::CX)),
                        Instruction::Binary(BinaryOperator::Shl, Operand::Reg(Register::CX), dst.clone())
                ])
            };

            result
        },

        Instruction::Binary(BinaryOperator::Shr, src, dst) => {
            let result = match src {
                Operand::Reg(Register::CX) => None,
                _ => Some(vec![
                        Instruction::Mov(src.clone(), Operand::Reg(Register::CX)),
                        Instruction::Binary(BinaryOperator::Shr, Operand::Reg(Register::CX), dst.clone())
                ])
            };

            result
        },
        Instruction::Binary(BinaryOperator::Mul, src, Operand::Stack(stack_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*stack_idx), Operand::Reg(Register::R11)),
                Instruction::Binary(BinaryOperator::Mul, src.clone(), Operand::Reg(Register::R11)),
                Instruction::Mov(Operand::Reg(Register::R11), Operand::Stack(*stack_idx))
            ])
        },
        Instruction::Binary(BinaryOperator::Mul, src, Operand::Data(glob_var_name)) => {
            Some(vec![
                Instruction::Mov(Operand::Data(glob_var_name.clone()), Operand::Reg(Register::R11)),
                Instruction::Binary(BinaryOperator::Mul, src.clone(), Operand::Reg(Register::R11)),
                Instruction::Mov(Operand::Reg(Register::R11), Operand::Data(glob_var_name.clone()))
            ])
        },
        Instruction::Binary(binop, Operand::Stack(src_idx), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Binary(binop.clone(), Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Binary(binop, Operand::Stack(src_idx), Operand::Data(dst_var_name)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Binary(binop.clone(), Operand::Reg(Register::R10), Operand::Data(dst_var_name.clone()))
            ])
        },
        Instruction::Binary(binop, Operand::Data(src_var_name), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Data(src_var_name.clone()), Operand::Reg(Register::R10)),
                Instruction::Binary(binop.clone(), Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])
        },

        Instruction::Binary(binop, Operand::Data(src_var_name),  Operand::Data(dst_var_name)) => {
            Some(vec![
                Instruction::Mov(Operand::Data(src_var_name.clone()), Operand::Reg(Register::R10)),
                Instruction::Binary(binop.clone(), Operand::Reg(Register::R10), Operand::Data(dst_var_name.clone()))
            ])
        },
        Instruction::Idiv(Operand::Imm(c)) => {
            Some(vec![
                Instruction::Mov(Operand::Imm(*c), Operand::Reg(Register::R10)),
                Instruction::Idiv(Operand::Reg(Register::R10))
            ])
        },
        Instruction::Cmp(Operand::Stack(src1_idx), Operand::Stack(src2_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src1_idx), Operand::Reg(Register::R10)),
                Instruction::Cmp(Operand::Reg(Register::R10), Operand::Stack(*src2_idx))
            ])
        },
        Instruction::Cmp(Operand::Stack(src1_idx), Operand::Data(dst_var_name)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src1_idx), Operand::Reg(Register::R10)),
                Instruction::Cmp(Operand::Reg(Register::R10), Operand::Data(dst_var_name.clone()))
            ])
        },
        Instruction::Cmp(Operand::Data(src_var_name), Operand::Stack(src2_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Data(src_var_name.clone()), Operand::Reg(Register::R10)),
                Instruction::Cmp(Operand::Reg(Register::R10), Operand::Stack(*src2_idx))
            ])
        },
        Instruction::Cmp(Operand::Data(src_var_name), Operand::Data(dst_var_name)) => {
            Some(vec![
                Instruction::Mov(Operand::Data(src_var_name.clone()), Operand::Reg(Register::R10)),
                Instruction::Cmp(Operand::Reg(Register::R10), Operand::Data(dst_var_name.clone()))
            ])
        },
        Instruction::Cmp(src1, Operand::Imm(src2_c)) => {
            Some(vec![
                Instruction::Mov(Operand::Imm(*src2_c), Operand::Reg(Register::R11)),
                Instruction::Cmp(src1.clone(), Operand::Reg(Register::R11))
            ])
        },
        _ => None
    }
}


fn fixup_function_body_instructions(instructions: &mut Vec<Instruction>, stack_allocation_size: usize) -> Result<(), String>
{
    const ALIGNMENT : usize = 16;
    let rounded_stack_allocation_size = (stack_allocation_size + ALIGNMENT - 1) & !(ALIGNMENT - 1);
    let mut new_instructions: Vec<Instruction> = vec![ Instruction::AllocateStack(rounded_stack_allocation_size) ];

    for it in instructions.drain(..) {

        if let  Some(mut replacements) = fixup_instruction_operands(&it) {
            new_instructions.append(&mut replacements);
        }
        else {
            new_instructions.push(it);
        }
    }

    instructions.append(&mut new_instructions);

    Ok(())
}



pub fn fixup_function_instructions(top_level_item: &mut TopLevel, stack_allocation_size: usize) -> Result<(), String>
{
    match  top_level_item {
        TopLevel::Function(func_name, global, instructions) => {
            fixup_function_body_instructions(instructions, stack_allocation_size)?;
        },
        TopLevel::StaticVariable(_,_,_) => {}
        //_ => {
        //    return Err(format!("Fixup Instructions: Expected FunctionDefinion, got '{:?}'", top_level_item));
        //}
    };

    Ok(())
}
