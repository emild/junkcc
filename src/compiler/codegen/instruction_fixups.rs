use std::ops::Not;

use super::ast::*;

fn fixup_instruction_operands(instruction: &Instruction) -> Option<Vec<Instruction>>
{
    match instruction {
        //Mov with two memory operands does not work
        Instruction::Mov(src, dst) if src.is_mem() && dst.is_mem() => {
            Some(vec![
                Instruction::Mov(src.clone(), Operand::Reg(Register::R10)),
                Instruction::Mov(Operand::Reg(Register::R10), dst.clone())
            ])
        }

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

        //Mul: Destination cannot be a memory operand
        Instruction::Binary(BinaryOperator::Mul, src, dst) if dst.is_mem() => {
            Some(vec![
                Instruction::Mov(dst.clone(), Operand::Reg(Register::R11)),
                Instruction::Binary(BinaryOperator::Mul, src.clone(), Operand::Reg(Register::R11)),
                Instruction::Mov(Operand::Reg(Register::R11), dst.clone())
            ])
        },

        //Generic binary operators: Can't use two memory operands
        Instruction::Binary(binop, src, dst) if src.is_mem() && dst.is_mem()=> {
            Some(vec![
                Instruction::Mov(src.clone(), Operand::Reg(Register::R10)),
                Instruction::Binary(binop.clone(), Operand::Reg(Register::R10), dst.clone())
            ])
        },

        //Idiv: Divisor cannot be immediate
        Instruction::Idiv(Operand::Imm(c)) => {
            Some(vec![
                Instruction::Mov(Operand::Imm(*c), Operand::Reg(Register::R10)),
                Instruction::Idiv(Operand::Reg(Register::R10))
            ])
        },

        //Cmp: Two mem operands are not allowed
        Instruction::Cmp(src1, src2) if src1.is_mem() && src2.is_mem() => {
            Some(vec![
                Instruction::Mov(src1.clone(), Operand::Reg(Register::R10)),
                Instruction::Cmp(Operand::Reg(Register::R10), src2.clone())
            ])
        },

        //Cmp second operand cannot be immediate
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
