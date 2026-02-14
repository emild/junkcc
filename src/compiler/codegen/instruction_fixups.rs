use std::mem::replace;

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
        }
        _ => None
    }
}


fn fixup_function_body_instructions(instructions: &mut Vec<Instruction>, stack_allocation_size: i64) -> Result<(), String>
{
    let mut new_instructions: Vec<Instruction> = vec![ Instruction::AllocateStack(stack_allocation_size) ];

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



fn fixup_function_instructions(func_def: &mut FunctionDefinition, stack_allocation_size: i64) -> Result<(), String>
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


pub fn fixup_instructions(program: &mut Program, stack_allocation_size: i64) -> Result<(), String>
{
    if stack_allocation_size < 0 {
        return Err(format!("Fixup instructions: Invalid stack allocation size: {}", stack_allocation_size));
    }

    match program {
        Program::ProgramDefinition(func_def) => {
            fixup_function_instructions(func_def, stack_allocation_size)?;
            Ok(())
        },

        _ => {
            return Err(format!("Fixup instructions: Expected program definiton, got {:?}", program));
        }
    }
}