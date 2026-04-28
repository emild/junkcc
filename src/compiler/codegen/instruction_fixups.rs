use std::ops::Not;

use super::ast::*;

fn fixup_instruction_operands(instruction: &Instruction) -> Option<Vec<Instruction>>
{
    match instruction {

        //Movq from immediate to memory only uses the lower 32 bits from the immediate and sign-extends the result
        //If we need to store an actual 64 bit constant, store it to a register first
        Instruction::Mov(AssemblyType::QuadWord, Operand::Imm(c), dst) if dst.is_mem() => {
            if *c >=  i64::from(i32::MIN) && *c < i64::from(i32::MAX) {
                None
            }
            else {
                Some(vec![
                    Instruction::Mov(AssemblyType::QuadWord, Operand::Imm(*c), Operand::Reg(Register::R10)),
                    Instruction::Mov(AssemblyType::QuadWord, Operand::Reg(Register::R10), dst.clone())
                ])
            }
        },

        //Mov with immediate source if the immediate value value exceeds the range of a 32 bit integer
        Instruction::Mov(AssemblyType::LongWord, Operand::Imm(c), dst) => {
            if *c >=  i64::from(i32::MIN) && *c < i64::from(i32::MAX) {
                None
            }
            else {
                Some(vec![
                    Instruction::Mov(AssemblyType::LongWord, Operand::Imm(*c & 0xFFFFFFFF), dst.clone())
                ])
            }
        },

        //Mov with two memory operands does not work
        Instruction::Mov(ass_type, src, dst) if src.is_mem() && dst.is_mem() => {
            Some(vec![
                Instruction::Mov(ass_type.clone(), src.clone(), Operand::Reg(Register::R10)),
                Instruction::Mov(ass_type.clone(), Operand::Reg(Register::R10), dst.clone())
            ])
        },

        //Movsx does not accept immediate as source and memory as destination
        Instruction::Movsx(Operand::Imm(src_c), dst) if dst.is_mem() => {
            Some(vec![
                Instruction::Mov(AssemblyType::LongWord, Operand::Imm(*src_c), Operand::Reg(Register::R10)),
                Instruction::Movsx(Operand::Reg(Register::R10), Operand::Reg(Register::R11)),
                Instruction::Mov(AssemblyType::QuadWord, Operand::Reg(Register::R11), dst.clone())
            ])
        }

        //Movsx doesn't accept immediate as source
        Instruction::Movsx(Operand::Imm(src_c), Operand::Reg(dest_reg)) => {
            Some(vec![
                Instruction::Mov(AssemblyType::LongWord, Operand::Imm(*src_c), Operand::Reg(Register::R10)),
                Instruction::Movsx(Operand::Reg(Register::R10), Operand::Reg(dest_reg.clone()))
            ])
        },

        //Movsx does not accept memory as destination
        Instruction::Movsx(src, dst) if dst.is_mem() => {
            Some(vec![
                Instruction::Movsx(src.clone(), Operand::Reg(Register::R11)),
                Instruction::Mov(AssemblyType::QuadWord, Operand::Reg(Register::R11), dst.clone())
            ])
        }

        //Shift count must be in CL
        Instruction::Binary(binop, ass_type, src, dst) if *binop == BinaryOperator::Shl || *binop == BinaryOperator::Shr => {
            let result = match src {
                Operand::Reg(Register::CX) => None,
                _ => Some(vec![
                    Instruction::Mov(AssemblyType::LongWord, src.clone(), Operand::Reg(Register::CX)),
                    Instruction::Binary(binop.clone(), ass_type.clone(), Operand::Reg(Register::CX), dst.clone())
                ])
            };

            result
        },

        //Mul: Destination cannot be a memory operand
        Instruction::Binary(BinaryOperator::Mul, ass_type, src, dst) if dst.is_mem() => {
            Some(vec![
                Instruction::Mov(ass_type.clone(), dst.clone(), Operand::Reg(Register::R11)),
                Instruction::Binary(BinaryOperator::Mul, ass_type.clone(), src.clone(), Operand::Reg(Register::R11)),
                Instruction::Mov(ass_type.clone(), Operand::Reg(Register::R11), dst.clone())
            ])
        },


        //For quadword (64 bit) versions of Add, Sub, Mul, And, Or, Xor, if the source operand is immediate
        //only its lower 32 bits are considered. The operand is sign-extended to 64 bits
        //If we need an operand beyond the 32 bit range, we must store it into a register first
        Instruction::Binary(binop, AssemblyType::QuadWord, Operand::Imm(src_c), dst) if  *binop != BinaryOperator::Shl && *binop != BinaryOperator::Shr => {
            if *src_c >=  i64::from(i32::MIN) && *src_c < i64::from(i32::MAX) {
                None
            }
            else {
                Some(vec![
                    Instruction::Mov(AssemblyType::QuadWord, Operand::Imm(*src_c), Operand::Reg(Register::R10)),
                    Instruction::Binary(binop.clone(), AssemblyType::QuadWord, Operand::Reg(Register::R10), dst.clone())
                ])
            }
        },


        //Generic binary operators: Can't use two memory operands
        Instruction::Binary(binop, ass_type, src, dst) if src.is_mem() && dst.is_mem()=> {
            Some(vec![
                Instruction::Mov(ass_type.clone(), src.clone(), Operand::Reg(Register::R10)),
                Instruction::Binary(binop.clone(), ass_type.clone(), Operand::Reg(Register::R10), dst.clone())
            ])
        },

        //Idiv: Divisor cannot be immediate
        Instruction::Idiv(ass_type, Operand::Imm(c)) => {
            Some(vec![
                Instruction::Mov(ass_type.clone(), Operand::Imm(*c), Operand::Reg(Register::R10)),
                Instruction::Idiv(ass_type.clone(), Operand::Reg(Register::R10))
            ])
        },


        //Cmp: Quad versions: If the first operand is immediate, only its lower 32 bits are considered, and
        //the operand is sign-extended to 64 bits. If the actual value of the immediate cannot be represented on
        //32 bits, we must copy the immendiate to a register first
        //Cmp: Two mem operands are not allowed
        Instruction::Cmp(AssemblyType::QuadWord, Operand::Imm(src_c1), src2) => {
            if *src_c1 >=  i64::from(i32::MIN) && *src_c1 < i64::from(i32::MAX) {
                None
            }
            else {
                Some(vec![
                    Instruction::Mov(AssemblyType::QuadWord, Operand::Imm(*src_c1), Operand::Reg(Register::R10)),
                    Instruction::Cmp(AssemblyType::QuadWord, Operand::Reg(Register::R10), src2.clone())
                ])
            }
        },

        //Cmp: Two mem operands are not allowed
        Instruction::Cmp(ass_type, src1, src2) if src1.is_mem() && src2.is_mem() => {
            Some(vec![
                Instruction::Mov(ass_type.clone(), src1.clone(), Operand::Reg(Register::R10)),
                Instruction::Cmp(ass_type.clone(), Operand::Reg(Register::R10), src2.clone())
            ])
        },

        //Cmp: second operand cannot be immediate.
        Instruction::Cmp(ass_type, src1, Operand::Imm(src2_c)) => {
            Some(vec![
                Instruction::Mov(ass_type.clone(), Operand::Imm(*src2_c), Operand::Reg(Register::R11)),
                Instruction::Cmp(ass_type.clone(), src1.clone(), Operand::Reg(Register::R11))
            ])
        },

        //Push: For Immediate operands, only the lower 32 bits are considered and the operand is sign-extended to 64 bits
        //If the immediate is not representable on 32 bits we must copy it to a register first
        Instruction::Push(Operand::Imm(c)) => {
            if *c >=  i64::from(i32::MIN) && *c < i64::from(i32::MAX) {
                None
            }
            else {
                Some(vec![
                    Instruction::Mov(AssemblyType::QuadWord, Operand::Imm(*c), Operand::Reg(Register::R10)),
                    Instruction::Push(Operand::Reg(Register::R10))
                ])
            }
        }
        _ => None
    }
}



fn fixup_function_body_instructions(instructions: &mut Vec<Instruction>, stack_allocation_size: usize) -> Result<(), String>
{
    const ALIGNMENT : usize = 16;
    let rounded_stack_allocation_size = (stack_allocation_size + ALIGNMENT - 1) & !(ALIGNMENT - 1);
    let mut new_instructions: Vec<Instruction> = vec![
        Instruction::Binary(
            BinaryOperator::Sub,
            AssemblyType::QuadWord,
            Operand::Imm(rounded_stack_allocation_size as i64),
            Operand::Reg(Register::SP))
        ];

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
        TopLevel::StaticVariable(_,_,_,_) => {}
        //_ => {
        //    return Err(format!("Fixup Instructions: Expected FunctionDefinion, got '{:?}'", top_level_item));
        //}
    };

    Ok(())
}
