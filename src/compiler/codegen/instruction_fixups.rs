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
        Instruction::Binary(BinaryOperator::Add, Operand::Stack(src_idx), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Binary(BinaryOperator::Add, Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])

        },
        Instruction::Binary(BinaryOperator::Sub, Operand::Stack(src_idx), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Binary(BinaryOperator::Sub, Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Binary(BinaryOperator::And, Operand::Stack(src_idx), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Binary(BinaryOperator::And, Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Binary(BinaryOperator::Or, Operand::Stack(src_idx), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Binary(BinaryOperator::Or, Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Binary(BinaryOperator::Xor, Operand::Stack(src_idx), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::R10)),
                Instruction::Binary(BinaryOperator::Xor, Operand::Reg(Register::R10), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Binary(BinaryOperator::Shl, Operand::Stack(src_idx), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::CX)),
                Instruction::Binary(BinaryOperator::Shl, Operand::Reg(Register::CL), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Binary(BinaryOperator::Shr, Operand::Stack(src_idx), Operand::Stack(dst_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src_idx), Operand::Reg(Register::CX)),
                Instruction::Binary(BinaryOperator::Shr, Operand::Reg(Register::CL), Operand::Stack(*dst_idx))
            ])
        },
        Instruction::Idiv(Operand::Imm(c)) => {
            Some(vec![
                Instruction::Mov(Operand::Imm(*c), Operand::Reg(Register::R10)),
                Instruction::Idiv(Operand::Reg(Register::R10))
            ])
        },
        Instruction::Binary(BinaryOperator::Mul, src, Operand::Stack(stack_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*stack_idx), Operand::Reg(Register::R11)),
                Instruction::Binary(BinaryOperator::Mul, src.clone(), Operand::Reg(Register::R11)),
                Instruction::Mov(Operand::Reg(Register::R11), Operand::Stack(*stack_idx))
            ])
        },
        Instruction::Cmp(Operand::Stack(src1_idx), Operand::Stack(src2_idx)) => {
            Some(vec![
                Instruction::Mov(Operand::Stack(*src1_idx), Operand::Reg(Register::R10)),
                Instruction::Cmp(Operand::Reg(Register::R10), Operand::Stack(*src2_idx))
            ])
        },
        Instruction::Cmp(src1, Operand::Imm(src2_c)) => {
            Some(vec![
                Instruction::Mov(Operand::Imm(*src2_c), Operand::Reg(Register::R11)),
                Instruction::Cmp(src1.clone(), Operand::Reg(Register::R11))
            ])
        },
        Instruction::SetCC(cc, Operand::Reg(r)) => {
            if let Some(new_r) = match r {
                Register::AX  => Some(Register::AL),
                Register::CX  => Some(Register::CL),
                Register::DX  => Some(Register::DL),
                Register::R10 => Some(Register::R10B),
                Register::R11 => Some(Register::R11B),
                _ => None
            } {
                Some(vec![
                    Instruction::SetCC(cc.clone(), Operand::Reg(new_r))
                ])
            }
            else {
                None
            }
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



pub fn fixup_function_instructions(func_def: &mut FunctionDefinition, stack_allocation_size: usize) -> Result<(), String>
{
    match  func_def {
        FunctionDefinition::Function(func_name, instructions) => {
            fixup_function_body_instructions(instructions, stack_allocation_size)?;
        },
        _ => {
            return Err(format!("Fixup Instructions: Expected FunctionDefinion, got '{:?}'", func_def));
        }
    };

    Ok(())
}
