use std::collections::HashMap;

use super::ast::*;
use super::fixup_function_instructions;

struct PseudoOperandState {
    pseudo_op_table: HashMap<String, i64>,
    current_stack_index: i64
}

impl PseudoOperandState {
    pub fn new() -> PseudoOperandState {
        PseudoOperandState {
            pseudo_op_table: HashMap::new(),
            current_stack_index: 0
        }
    }

    pub fn replace_operand(&mut self, operand: &Operand) -> Operand
    {
        match operand {
            Operand::Pseudo(var_name) => {
                let stack_index = self.pseudo_op_table.entry(var_name.clone()).or_insert_with(
                    ||
                    {
                        self.current_stack_index -= 4;
                        self.current_stack_index
                    });
                Operand::Stack(*stack_index)
            }
            _ => operand.clone(),
        }
    }

    pub fn get_stack_allocation_size(&self) -> usize { -self.current_stack_index as usize }
}


fn replace_pseudo_operands_in_function_body(instructions: &mut Vec<Instruction>) -> Result<usize, String>
{

    let mut pseudo_operand_state = PseudoOperandState::new();

    for it in instructions.iter_mut() {
        let replace_instruction = match it {
            Instruction::Mov(src, dst ) => {
                let new_src = pseudo_operand_state.replace_operand(&src);
                let new_dst = pseudo_operand_state.replace_operand(&dst);
                Some(Instruction::Mov(new_src, new_dst))
            },
            Instruction::Push(src) => {
                let new_src = pseudo_operand_state.replace_operand(&src);
                Some(Instruction::Push(new_src))
            },
            Instruction::Unary(unary_op, dst ) => {
                let new_dest = pseudo_operand_state.replace_operand(&dst);
                Some(Instruction::Unary(unary_op.clone(), new_dest))
            },
            Instruction::Binary(binary_op, src, dst) => {
                let new_src = pseudo_operand_state.replace_operand(&src);
                let new_dst = pseudo_operand_state.replace_operand(&dst);
                Some(Instruction::Binary(binary_op.clone(), new_src, new_dst))
            },
            Instruction::Idiv(divisor) => {
                let new_divisor = pseudo_operand_state.replace_operand(&divisor);
                Some(Instruction::Idiv(new_divisor))
            },
            Instruction::Cmp(src1, src2) => {
                let new_src1 = pseudo_operand_state.replace_operand(&src1);
                let new_src2 = pseudo_operand_state.replace_operand(&src2);
                Some(Instruction::Cmp(new_src1, new_src2))
            },
            Instruction::SetCC(cc, dst ) => {
                let new_dst = pseudo_operand_state.replace_operand(&dst);
                Some(Instruction::SetCC(cc.clone(), new_dst))
            }
            _ => None
        };

        if let Some(replace_instruction) = replace_instruction {
            *it = replace_instruction;
        }
    }

    Ok(pseudo_operand_state.get_stack_allocation_size())
}

//Any pseudo operands are replaced with stack locations
pub fn replace_pseudo_operands(program: &mut Program) -> Result<(), String>
{
    match program {
        Program::ProgramDefinition(func_defs) => {
            for func_def in func_defs {
                let FunctionDefinition::Function(_, instructions ) = func_def;
                let stack_allocation_size = replace_pseudo_operands_in_function_body(instructions)?;
                fixup_function_instructions(func_def, stack_allocation_size)?;
            }
            Ok(())
        }
    }
}