use std::collections::HashMap;

use super::ast::*;
use super::fixup_function_instructions;

struct PseudoOperandState<'a> {
    pseudo_op_table: HashMap<String, i64>,
    current_stack_index: i64,
    symbol_table: &'a HashMap<String, AssemblySymbolInfo>
}

impl<'a> PseudoOperandState<'a> {
    pub fn new(symbol_table: &'a HashMap<String, AssemblySymbolInfo>) -> PseudoOperandState<'a> {
        PseudoOperandState {
            pseudo_op_table: HashMap::new(),
            current_stack_index: 0,
            symbol_table
        }
    }

    pub fn replace_operand(&mut self, operand: &Operand, ass_type: &AssemblyType) -> Operand
    {
        match operand {
            Operand::Pseudo(var_name) => {
                if !self.pseudo_op_table.contains_key(var_name) {
                    match self.symbol_table.get(var_name) {
                        Some(AssemblySymbolInfo::ObjEntry(_ass_type, true)) =>  {
                            return Operand::Data(var_name.clone());
                        },
                        Some(AssemblySymbolInfo::FuncEntry(_)) => {
                            //TODO: Remove once function pointers are implemented??
                            panic!("Symbol '{var_name}' found in symbol table, but it is not a variable");
                        },
                        _ => {}
                    };
                }
                let stack_index = self.pseudo_op_table.entry(var_name.clone()).or_insert_with(
                    ||
                    {
                        //align stack index Longword to 4 bytes and Quadword to 8 bytes
                        self.current_stack_index += (ass_type.size() as i64);
                        let ass_type_align  = ass_type.alignment();
                        self.current_stack_index = ((self.current_stack_index + (ass_type_align as i64)) / (ass_type_align as i64)) * (ass_type_align as i64);

                        -self.current_stack_index
                    });
                Operand::Stack(*stack_index)
            },
            _ => operand.clone(),
        }
    }

    pub fn get_stack_allocation_size(&self) -> usize { self.current_stack_index as usize }
}

fn replace_pseudo_operands_in_function_body(instructions: &mut Vec<Instruction>, assembly_symbol_table: &HashMap<String, AssemblySymbolInfo>) -> Result<usize, String>
{

    let mut pseudo_operand_state = PseudoOperandState::new(assembly_symbol_table);

    for it in instructions.iter_mut() {
        let replace_instruction = match it {
            Instruction::Mov(ass_type, src, dst ) => {
                let new_src = pseudo_operand_state.replace_operand(&src, ass_type);
                let new_dst = pseudo_operand_state.replace_operand(&dst, ass_type);
                Some(Instruction::Mov(ass_type.clone(), new_src, new_dst))
            },
            Instruction::Movsx(src, dst) => {
                let new_src = pseudo_operand_state.replace_operand(&src, &AssemblyType::LongWord);
                let new_dst = pseudo_operand_state.replace_operand(&dst, &AssemblyType::QuadWord);
                Some(Instruction::Movsx(new_src.clone(), new_dst.clone()))
            }
            Instruction::Push(src) => {
                let new_src = pseudo_operand_state.replace_operand(&src, &AssemblyType::QuadWord);
                Some(Instruction::Push(new_src))
            },
            Instruction::Unary(unary_op, ass_type, dst ) => {
                let new_dest = pseudo_operand_state.replace_operand(&dst, ass_type);
                Some(Instruction::Unary(unary_op.clone(), ass_type.clone(), new_dest))
            },
            Instruction::Binary(binary_op, ass_type, src, dst) => {
                let new_src = pseudo_operand_state.replace_operand(&src, ass_type);
                let new_dst = pseudo_operand_state.replace_operand(&dst, ass_type);
                Some(Instruction::Binary(binary_op.clone(), ass_type.clone(), new_src, new_dst))
            },
            Instruction::Idiv(ass_type, divisor) => {
                let new_divisor = pseudo_operand_state.replace_operand(&divisor, ass_type);
                Some(Instruction::Idiv(ass_type.clone(), new_divisor))
            },
            Instruction::Cmp(ass_type, src1, src2) => {
                let new_src1 = pseudo_operand_state.replace_operand(&src1, ass_type);
                let new_src2 = pseudo_operand_state.replace_operand(&src2, ass_type);
                Some(Instruction::Cmp(ass_type.clone(), new_src1, new_src2))
            },
            Instruction::SetCC(cc, dst ) => {
                let new_dst = pseudo_operand_state.replace_operand(&dst, &AssemblyType::LongWord);
                Some(Instruction::SetCC(cc.clone(), new_dst))
            },
            Instruction::Call(_)    |
            Instruction::Cdq(_)     |
            Instruction::Jmp(_)     |
            Instruction::JmpCC(_,_) |
            Instruction::Label(_)   |
            Instruction::Ret    => None
            //_ => None
        };

        if let Some(replace_instruction) = replace_instruction {
            *it = replace_instruction;
        }
    }

    Ok(pseudo_operand_state.get_stack_allocation_size())
}

//Any pseudo operands are replaced with stack locations
pub fn replace_pseudo_operands(program: &mut Program, symbol_table: &HashMap<String, AssemblySymbolInfo>) -> Result<(), String>
{
    match program {
        Program::ProgramDefinition(top_level_items) => {
            for top_level_item in top_level_items {
                if let TopLevel::Function(_, _, instructions ) = top_level_item {
                    let stack_allocation_size = replace_pseudo_operands_in_function_body(instructions, symbol_table)?;
                    fixup_function_instructions(top_level_item, stack_allocation_size)?;
                }
            }
            Ok(())
        }
    }
}