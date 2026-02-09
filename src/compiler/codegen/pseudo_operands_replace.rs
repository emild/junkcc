use std::collections::HashMap;

use super::ast::*;

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

    pub fn get_stack_allocation_size(&self) -> i64 { -self.current_stack_index }
}


fn replace_pseudo_operands_in_function_body(instructions: &mut Vec<Instruction>) -> Result<i64, String>
{

    let mut pseudo_operand_state = PseudoOperandState::new();
    
    for it in instructions.iter_mut() {
        let replace_instruction = match it {
            Instruction::Mov(src, dst ) => {
                let new_src = pseudo_operand_state.replace_operand(&src);
                let new_dst = pseudo_operand_state.replace_operand(&dst);
                Some(Instruction::Mov(new_src, new_dst))
            },
            Instruction::Unary(unary_op, dst ) => {
                let new_dest = pseudo_operand_state.replace_operand(&dst);
                Some(Instruction::Unary(unary_op.clone(), new_dest))
            },
            _ => None
        };

        if let Some(replace_instruction) = replace_instruction {
            *it = replace_instruction;
        }
    }

    Ok(pseudo_operand_state.get_stack_allocation_size())
}

//Any pseudo operands are replaced with stack locations
//Returns the maximum stack location (i.e. the number of bytes to be allocated on the stack)
pub fn replace_pseudo_operands(program: &mut Program) -> Result<i64, String>
{
    match program {
        Program::ProgramDefinition(FunctionDefinition::Function(name, instructions )) =>
        {
            let stack_size = replace_pseudo_operands_in_function_body(instructions)?;

            Ok(stack_size)
        }
    }
}