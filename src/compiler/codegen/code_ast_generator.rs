use std::cmp::min;
use std::collections::HashMap;



use super::super::tacky;
use super::super::parser::ast::Type;
use super::super::parser::SymbolInfo;
use super::super::parser::IdentifierAttrs;
use super::ast::*;


fn convert_type_to_assembly_type(typ: &Type) -> AssemblyType
{
    match typ {
        Type::Int  => AssemblyType::LongWord,
        Type::Long => AssemblyType::QuadWord,
        _ => { panic!("code_ast_generator: Attempt to call convert_type_to_assembly_type() on non scalar type: '{:?}'", typ); }
    }
}


fn get_symbol_assembly_type(sym: &String, symbol_table: &HashMap<String, SymbolInfo>) -> AssemblyType
{
    if let Some(sym_info) = symbol_table.get(sym) {
        convert_type_to_assembly_type(&sym_info.typ)
    }
    else {
        panic!("code_ast_generator: Symbol '{}' not found in [frontend] symbol table", sym);
    }
}

fn get_operand_assembly_type(oprnd: &Operand, symbol_table: &HashMap<String, SymbolInfo>) -> AssemblyType
{
    match oprnd {
        Operand::Pseudo(sym) => get_symbol_assembly_type(sym, symbol_table),
        Operand::Imm(_) => AssemblyType::QuadWord,
        _ => { panic!("code_ast_generator: Attempt to get assembly type for non tacky derived operand '{:?}'", oprnd); }
    }
}



fn convert_tacky_value_to_operand(val: &tacky::ast::Val) -> Result<Operand, String>
{
    match val {
        tacky::ast::Val::Constant(c) => Ok(Operand::Imm(c.to_i64())),
        tacky::ast::Val::Var(var_name) => Ok(Operand::Pseudo(var_name.clone())),
        _ => { return Err(format!("Tacky value to operand conversion error: cannot convert '{:?}'", val)); }
    }
}


fn symbol_info_to_assembly_symbol_info(sym_info: &SymbolInfo) -> AssemblySymbolInfo
{
    if sym_info.typ.is_func() {
        if let IdentifierAttrs::FuncAttr(defined, _global ) = sym_info.attrs {
            AssemblySymbolInfo::FuncEntry(defined)
        }
        else {
            panic!("symbol_info_to_assembly_symbol_info(): Corrupt symbolInfo: type '{}' is function, but attribute '{:?}' is not FuncAttr", sym_info.typ.to_string(), sym_info.attrs);
        }
    }
    else {
        let is_static =  match sym_info.attrs {
            IdentifierAttrs::LocalAttr => false,
            IdentifierAttrs::StaticAttr(_,_) => true,
            IdentifierAttrs::FuncAttr(_,_) => { panic!("symbol_info_to_assembly_symbol_info(): Corrrupt SymbolInfo: type '{}' is NOT function, but attribute '{:?} is FuncAttr", sym_info.typ.to_string(), sym_info.attrs); }
        };

        let ass_typ = convert_type_to_assembly_type(&sym_info.typ);
        AssemblySymbolInfo::ObjEntry(ass_typ, is_static)
    }
}


fn generate_code_for_tacky_ret_instruction(
    ret_val: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let ret_val_src = convert_tacky_value_to_operand(&ret_val)?;
    let ass_type = get_operand_assembly_type(&ret_val_src, symbol_table);
    instructions.push(Instruction::Mov(ass_type.clone(), ret_val_src, Operand::Reg(Register::AX)));
    instructions.push(Instruction::Ret);

    Ok(())
}


fn generate_code_for_tacky_unary_instruction(
    tacky_unary_op: &tacky::ast::UnaryOperator,
    src: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let unary_op_src = convert_tacky_value_to_operand(src)?;
    let unary_op_dst = convert_tacky_value_to_operand(dst)?;
    let src_ass_type = get_operand_assembly_type(&unary_op_src, symbol_table);
    let dst_ass_type = get_operand_assembly_type(&unary_op_dst, symbol_table);

    let mut unary_op_instructions = match tacky_unary_op {
        tacky::ast::UnaryOperator::Complement =>
            vec![
                Instruction::Mov(src_ass_type.clone(), unary_op_src, unary_op_dst.clone()),
                Instruction::Unary(UnaryOperator::Not, src_ass_type.clone(), unary_op_dst)
            ],
        tacky::ast::UnaryOperator::Negate =>
            vec![
                Instruction::Mov(src_ass_type.clone(), unary_op_src, unary_op_dst.clone()),
                Instruction::Unary(UnaryOperator::Neg, src_ass_type.clone(), unary_op_dst)
            ],
        tacky::ast::UnaryOperator::Plus =>
            vec![
               Instruction::Mov(src_ass_type.clone(), unary_op_src, unary_op_dst.clone()),
            ],
        tacky::ast::UnaryOperator::LogicalNot =>
            vec![
                Instruction::Mov(dst_ass_type.clone(), Operand::Imm(0), unary_op_dst.clone()),
                Instruction::Cmp(src_ass_type.clone(), unary_op_src, Operand::Imm(0)),
                Instruction::SetCC(CC::E, unary_op_dst.clone())
            ]
         //   _ => { panic!("codegen::generate_code_for_unary_instruction: Unimplemented Unary Operand: {:?}", tacky_unary_op); }
    };

    instructions.append(&mut unary_op_instructions);

    Ok(())
}



fn generate_code_for_remainder_instruction(
    src1: &Operand,
    src2: &Operand,
    dst: &Operand,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1_ass_type = get_operand_assembly_type(src1, symbol_table);

    let mut div_instructions = vec![
        Instruction::Mov(src1_ass_type.clone(), src1.clone(), Operand::Reg(Register::AX)),
        Instruction::Cdq(src1_ass_type.clone()),
        Instruction::Idiv(src1_ass_type.clone(), src2.clone()),
        Instruction::Mov(src1_ass_type.clone(), Operand::Reg(Register::DX), dst.clone())
    ];

    instructions.append(&mut div_instructions);

    Ok(())
}


fn generate_code_for_divide_instruction(
    src1: &Operand,
    src2: &Operand,
    dst: &Operand,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1_ass_type = get_operand_assembly_type(src1, symbol_table);
    let mut div_instructions = vec![
        Instruction::Mov(src1_ass_type.clone(), src1.clone(), Operand::Reg(Register::AX)),
        Instruction::Cdq(src1_ass_type.clone()),
        Instruction::Idiv(src1_ass_type.clone(), src2.clone()),
        Instruction::Mov(src1_ass_type.clone(), Operand::Reg(Register::AX), dst.clone())
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
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1_ass_type = get_operand_assembly_type(src1, symbol_table);
    instructions.push(Instruction::Mov(src1_ass_type.clone(), src1.clone(), dst.clone()));
    instructions.push(Instruction::Binary(bin_op.clone(), src1_ass_type.clone(), src2.clone(), dst.clone()));

    Ok(())
}

fn generate_code_for_condition(
    cc: &CC,
    src1: &Operand,
    src2: &Operand,
    dst: &Operand,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1_ass_type = get_operand_assembly_type(src1, symbol_table);
    let dst_ass_type = get_operand_assembly_type(dst, symbol_table);
    instructions.push(Instruction::Mov(dst_ass_type.clone(), Operand::Imm(0), dst.clone()));
    instructions.push(Instruction::Cmp(src1_ass_type.clone(), src2.clone(), src1.clone()));
    instructions.push(Instruction::SetCC(cc.clone(), dst.clone()));

    Ok(())
}


fn generate_code_for_tacky_binary_instruction(
    bin_op: &tacky::ast::BinaryOperator,
    src1: &tacky::ast::Val,
    src2: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1 = convert_tacky_value_to_operand(src1)?;
    let src2 = convert_tacky_value_to_operand(src2)?;
    let dst  = convert_tacky_value_to_operand(dst)?;

    let result = match bin_op {
        tacky::ast::BinaryOperator::Add => generate_code_for_binary_instruction(&BinaryOperator::Add, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Subtract => generate_code_for_binary_instruction(&BinaryOperator::Sub, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Multiply => generate_code_for_binary_instruction(&BinaryOperator::Mul, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Divide => generate_code_for_divide_instruction(&src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Remainder => generate_code_for_remainder_instruction(&src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::BitwiseAnd => generate_code_for_binary_instruction(&BinaryOperator::And, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::BitwiseOr => generate_code_for_binary_instruction(&BinaryOperator::Or, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::BitwiseXor => generate_code_for_binary_instruction(&BinaryOperator::Xor, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::ShiftLeft => generate_code_for_binary_instruction(&BinaryOperator::Shl, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::ShiftRight => generate_code_for_binary_instruction(&BinaryOperator::Shr, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Equal => generate_code_for_condition(&CC::E, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::NotEqual => generate_code_for_condition(&CC::NE, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::LessThan => generate_code_for_condition(&CC::L, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::LessOrEqual => generate_code_for_condition(&CC::LE, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::GreaterThan => generate_code_for_condition(&CC::G, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::GreaterOrEqual => generate_code_for_condition(&CC::GE, &src1, &src2, &dst, symbol_table, instructions)?,

        //_ => { panic!("codegen::generate_binary_instruction(): Unimplemented binop: {:?}", bin_op); }
    };

    Ok(result)
}

fn generate_code_for_tacky_copy_instruction(
    src: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src = convert_tacky_value_to_operand(src)?;
    let dst  = convert_tacky_value_to_operand(dst)?;
    let src_ass_type = get_operand_assembly_type(&src, symbol_table);

    instructions.push(Instruction::Mov(src_ass_type, src, dst));

    Ok(())
}


fn generate_code_for_tacky_jump_instruction(
    label: &String,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    instructions.push(Instruction::Jmp(label.clone()));

    Ok(())
}


fn generate_code_for_tacky_conditional_jump_instruction(
    cc: &CC,
    val: &tacky::ast::Val,
    label: &String,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let cmp_arg = convert_tacky_value_to_operand(val)?;
    let cmp_arg_ass_type = get_operand_assembly_type(&cmp_arg, symbol_table);
    instructions.push(Instruction::Cmp(cmp_arg_ass_type, Operand::Imm(0), cmp_arg));
    instructions.push(Instruction::JmpCC(cc.clone(), label.clone()));

    Ok(())
}

fn generate_code_for_tacky_label(
    label: &String,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    instructions.push(Instruction::Label(label.clone()));
    Ok(())
}



fn generate_code_for_tacky_function_call(
    func_name: &String,
    args: &Vec<tacky::ast::Val>,
    ret_val: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    // First 6 arguments are found in the registers below; The following arguments are pushed onto stack, in reverse order
    let args_registers = [
        Register::DI,
        Register::SI,
        Register::DX,
        Register::CX,
        Register::R8,
        Register::R9
    ];

    let num_reg_args = min(args.len(), args_registers.len());

    let mut register_args = vec![];
    for i in 0..num_reg_args {
        register_args.push(args[i].clone());
    }

    let mut stack_args = vec![];
    for i in num_reg_args..args.len() {
        stack_args.push(args[i].clone());
    }

    let stack_padding: usize = if stack_args.len() % 2 == 0 {
        0
    } else {
        8
    };

    if stack_padding != 0 {
        //AllocateStack(stack_padding)
        instructions.push(
            Instruction::Binary(
                BinaryOperator::Sub,
                AssemblyType::QuadWord,
                Operand::Imm(stack_padding as i64),
                Operand::Reg(Register::SP)
            )
        );
    }

    let mut reg_index = 0;
    for tacky_arg in register_args {
        let assembly_arg = convert_tacky_value_to_operand(&tacky_arg)?;
        let assembly_arg_ass_type = get_operand_assembly_type(&assembly_arg, symbol_table);
        instructions.push(Instruction::Mov(assembly_arg_ass_type, assembly_arg, Operand::Reg(args_registers[reg_index].clone())));
        reg_index += 1;
    }

    for tacky_arg in stack_args.iter().rev() {
        let assembly_arg = convert_tacky_value_to_operand(tacky_arg)?;
        match assembly_arg {
            Operand::Imm(_) |
            Operand::Reg(_) => {
                instructions.push(Instruction::Push(assembly_arg.clone()));
            },
            _ => {
                match get_operand_assembly_type(&assembly_arg, symbol_table) {
                    AssemblyType::QuadWord => {
                        instructions.push(Instruction::Push(assembly_arg.clone()));
                    },
                    AssemblyType::LongWord => {
                        instructions.push(Instruction::Mov(AssemblyType::LongWord, assembly_arg.clone(), Operand::Reg(Register::AX)));
                        instructions.push(Instruction::Push(Operand::Reg(Register::AX)));
                    }
                }
            }
        }
    }

    instructions.push(Instruction::Call(func_name.clone()));

    let bytes_to_remove = 8 * stack_args.len() + stack_padding;

    if bytes_to_remove != 0 {
        //DeallocateStack(bytes_to_remove)
        instructions.push(
            Instruction::Binary(
                BinaryOperator::Add,
                AssemblyType::QuadWord,
                Operand::Imm(bytes_to_remove as i64),
                Operand::Reg(Register::SP)
            )
        );
    }

    let assembly_dst = convert_tacky_value_to_operand(ret_val)?;
    let assembly_dst_ass_type = get_operand_assembly_type(&assembly_dst, symbol_table);
    instructions.push(Instruction::Mov(assembly_dst_ass_type, Operand::Reg(Register::AX), assembly_dst));


    Ok(())
}


fn generate_code_for_tacky_sign_extend(src: &tacky::ast::Val, dst: &tacky::ast::Val, _symbol_table: &HashMap<String, SymbolInfo>, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_op = convert_tacky_value_to_operand(src)?;
    let dst_op = convert_tacky_value_to_operand(dst)?;
    instructions.push(Instruction::Movsx(src_op, dst_op));

    Ok(())
}


fn generate_code_for_tacky_truncate(src: &tacky::ast::Val, dst: &tacky::ast::Val, _symbol_table: &HashMap<String, SymbolInfo>, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_op = convert_tacky_value_to_operand(src)?;
    let dst_op = convert_tacky_value_to_operand(dst)?;
    instructions.push(Instruction::Mov(AssemblyType::LongWord, src_op, dst_op));

    Ok(())
}


fn generate_code_for_tacky_instructions(tacky_instructions: &Vec<tacky::ast::Instruction>, symbol_table: &HashMap<String, SymbolInfo>, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    for tacky_inst in tacky_instructions {
        match tacky_inst {
            tacky::ast::Instruction::Return(ret_val) => {
                generate_code_for_tacky_ret_instruction(ret_val, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::Unary(tacky_unary_op, src, dst) => {
                generate_code_for_tacky_unary_instruction(tacky_unary_op, &src, &dst, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::Binary(tacky_binary_op, src1, src2, dst) => {
                 generate_code_for_tacky_binary_instruction(tacky_binary_op, &src1, &src2, &dst, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::Copy(src, dst) => {
                generate_code_for_tacky_copy_instruction(&src, &dst, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::Jump(label) => {
                generate_code_for_tacky_jump_instruction(&label, instructions)?;
            },
            tacky::ast::Instruction::JumpIfZero(val, label) => {
                generate_code_for_tacky_conditional_jump_instruction(&CC::E, &val, &label, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::JumpIfNotZero(val, label) => {
                generate_code_for_tacky_conditional_jump_instruction(&CC::NE, &val, &label, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::Label(label) => {
                generate_code_for_tacky_label(&label, instructions)?;
            },
            tacky::ast::Instruction::FuncCall(func_name, args, ret_val) => {
                generate_code_for_tacky_function_call(func_name, args, ret_val, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::SignExtend(src, dst) => {
                generate_code_for_tacky_sign_extend(src, dst, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::Truncate(src, dst, ) => {
                generate_code_for_tacky_truncate(src, dst, symbol_table, instructions)?;
            }


            //_ => { panic!("Invalid TACKY Instruction: {:?}", tacky_inst); }
        };
    }

    Ok(())
}



fn generate_code_for_function_definition(func_name: &String, global: bool, params: &Vec<String>, tacky_instructions: &Vec<tacky::ast::Instruction>, symbol_table: &HashMap<String, SymbolInfo>) -> Result<TopLevel, String>
{
    let mut new_instructions = vec![];
    // First 6 parameters are found in the registers below; The following parameters are pushed onto stack, in reverse order
    let params_registers = [
        Register::DI,
        Register::SI,
        Register::DX,
        Register::CX,
        Register::R8,
        Register::R9
    ];

    let mut param_idx: usize = 0;
    let num_reg_params = min(params.len(), params_registers.len());

    for reg_idx in 0..num_reg_params {
        let ass_type = get_symbol_assembly_type(&params[param_idx], symbol_table);
        new_instructions.push(Instruction::Mov(ass_type, Operand::Reg(params_registers[reg_idx].clone()), Operand::Pseudo(params[reg_idx].clone())));
        param_idx += 1;
    }

    for param_idx in num_reg_params.. params.len() {
        let stack_idx =  16 + 8 * (param_idx - num_reg_params);
        let ass_type = get_symbol_assembly_type(&params[param_idx], symbol_table);
        new_instructions.push(Instruction::Mov(ass_type, Operand::Stack(stack_idx as i64), Operand::Pseudo(params[param_idx].clone())));
    }

    generate_code_for_tacky_instructions(tacky_instructions, symbol_table, &mut new_instructions)?;

    Ok(TopLevel::Function(func_name.clone(), global, new_instructions))

}



pub fn generate_code_for_top_level_item(tacky_top_level_item: &tacky::ast::TopLevel, symbol_table: &HashMap<String, SymbolInfo>) -> Result<TopLevel, String>
{
    let top_level_item = match tacky_top_level_item {
        tacky::ast::TopLevel::Function(func_name, global, params, tacky_instructions) => {
            let top_level_item = generate_code_for_function_definition(func_name, *global, params, tacky_instructions, symbol_table)?;
            top_level_item
        },

        tacky::ast::TopLevel::StaticVariable(var_name, global, typ, initial_value) => {
            let top_level_item = TopLevel::StaticVariable(var_name.clone(), *global, typ.alignment(), initial_value.clone());
            top_level_item
        }
    };

    Ok(top_level_item)
}



pub fn generate_code(program: &tacky::ast::Program, symbol_table: &HashMap<String, SymbolInfo>) -> Result<(Program, HashMap<String, AssemblySymbolInfo>), String>
{
    let tacky::ast::Program::ProgramDefinition(tacky_top_level_items) = program;
    let mut top_level_items = vec![];

    for tacky_top_level_item in tacky_top_level_items {

        let top_level_item = generate_code_for_top_level_item(&tacky_top_level_item, symbol_table)?;
        top_level_items.push(top_level_item);

    }

    let mut assembly_symbol_table = HashMap::new();
    for (symbol, symbol_info) in symbol_table {
        assembly_symbol_table.insert(symbol.clone(), symbol_info_to_assembly_symbol_info(symbol_info));
    }

    Ok((Program::ProgramDefinition(top_level_items), assembly_symbol_table))
}