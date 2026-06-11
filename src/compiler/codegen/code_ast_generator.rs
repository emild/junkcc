use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};



use super::const_double::{get_double_constant_top_level_items, make_double_constant};
use super::super::parser::{StaticInit, SymbolInfo, IdentifierAttrs};
use super::super::parser::ast::{Type, Const, FloatingPointConst, IntegerConst};
use super::super::tacky;

use super::ast::*;



static JUMP_LABEL_INDEX: AtomicUsize = AtomicUsize::new(0);

fn make_unique_label(prefix: &String) -> String
{
    let idx = JUMP_LABEL_INDEX.fetch_add(1, Ordering::SeqCst);

    format!("L.lbl_{}_{}", prefix, idx)
}


fn convert_type_to_assembly_type(typ: &Type) -> AssemblyType
{
    match typ {
        Type::Int |
        Type::UInt  => AssemblyType::LongWord,
        Type::Long|
        Type::ULong => AssemblyType::QuadWord,
        Type::Double => AssemblyType::Double,
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



fn get_tacky_value_assembly_type(val: &tacky::ast::Val, symbol_table: &HashMap<String, SymbolInfo>) -> AssemblyType
{
    match val {
        tacky::ast::Val::Var(var_name)
            => get_symbol_assembly_type(var_name, symbol_table),

        tacky::ast::Val::Constant(Const::I(IntegerConst::ConstInt(_))) |
        tacky::ast::Val::Constant(Const::I(IntegerConst::ConstUInt(_)))
            => AssemblyType::LongWord,

        tacky::ast::Val::Constant(Const::I(IntegerConst::ConstLong(_))) |
        tacky::ast::Val::Constant(Const::I(IntegerConst::ConstULong(_)))
            => AssemblyType::QuadWord,

        tacky::ast::Val::Constant(Const::F(FloatingPointConst::ConstDouble(_)))
            => AssemblyType::Double
    }
}


fn is_tacky_variable_signed(var_name: &String, symbol_table: &HashMap<String, SymbolInfo>) -> bool
{
    if let Some(sym_info) = symbol_table.get(var_name) {
        sym_info.typ.is_signed_integer()
    }
    else {
        panic!("code_ast_generator: Symbol '{}' not found in [frontend] symbol table", var_name);
    }

}


fn is_tacky_value_signed(val: &tacky::ast::Val, symbol_table: &HashMap<String, SymbolInfo>) -> bool
{
    match val {
        tacky::ast::Val::Var(var_name)
            => is_tacky_variable_signed(var_name, symbol_table),

        tacky::ast::Val::Constant(Const::I(IntegerConst::ConstInt(_))) |
        tacky::ast::Val::Constant(Const::I(IntegerConst::ConstLong(_)))
            => true,

        tacky::ast::Val::Constant(Const::I(IntegerConst::ConstUInt(_))) |
        tacky::ast::Val::Constant(Const::I(IntegerConst::ConstULong(_)))
            => false,

        tacky::ast::Val::Constant(Const::F(_))
            => false,
    }
}


fn convert_tacky_value_to_operand(val: &tacky::ast::Val) -> Result<Operand, String>
{
    match val {
        tacky::ast::Val::Constant(Const::I(c)) => Ok(Operand::Imm(c.to_i64())),
        tacky::ast::Val::Constant(Const::F(FloatingPointConst::ConstDouble(c))) => {
            let const_double_label = make_double_constant(*c);
            Ok(Operand::Data(const_double_label))
        },
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
        AssemblySymbolInfo::ObjEntry(ass_typ, is_static, false)
    }
}


fn generate_code_for_tacky_ret_instruction(
    ret_val: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let ret_val_src = convert_tacky_value_to_operand(&ret_val)?;
    let ass_type = get_tacky_value_assembly_type(ret_val, symbol_table);
    if ass_type == AssemblyType::Double {
        instructions.push(Instruction::Mov(AssemblyType::Double, ret_val_src, Operand::Reg(Register::XMM0)));
    }
    else {
        instructions.push(Instruction::Mov(ass_type.clone(), ret_val_src, Operand::Reg(Register::AX)));
    }
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
    let src_ass_type = get_tacky_value_assembly_type(src, symbol_table);
    let dst_ass_type = get_tacky_value_assembly_type(dst, symbol_table);

    let mut unary_op_instructions = match tacky_unary_op {
        tacky::ast::UnaryOperator::Complement =>
            vec![
                Instruction::Mov(src_ass_type.clone(), unary_op_src, unary_op_dst.clone()),
                Instruction::Unary(UnaryOperator::Not, src_ass_type.clone(), unary_op_dst)
            ],
        tacky::ast::UnaryOperator::Negate =>
            if src_ass_type == AssemblyType::Double {
                let minus_zero = make_double_constant(-0.0);
                vec![
                    Instruction::Mov(AssemblyType::Double, unary_op_src, unary_op_dst.clone()),
                    Instruction::Binary(BinaryOperator::Xor, AssemblyType::Double, Operand::Data(minus_zero.clone()), unary_op_dst.clone())
                ]
            }
            else {
                vec![
                    Instruction::Mov(src_ass_type.clone(), unary_op_src, unary_op_dst.clone()),
                    Instruction::Unary(UnaryOperator::Neg, src_ass_type.clone(), unary_op_dst.clone())
                ]
            },
        tacky::ast::UnaryOperator::Plus =>
            vec![
               Instruction::Mov(src_ass_type.clone(), unary_op_src, unary_op_dst.clone()),
            ],
        tacky::ast::UnaryOperator::LogicalNot =>
            if src_ass_type == AssemblyType::Double {
                vec![
                    Instruction::Binary(BinaryOperator::Xor, AssemblyType::Double, Operand::Reg(Register::XMM0), Operand::Reg(Register::XMM0)),
                    Instruction::Cmp(AssemblyType::Double, unary_op_src.clone(), Operand::Reg(Register::XMM0)),
                    Instruction::Mov(AssemblyType::LongWord, Operand::Imm(0), unary_op_dst.clone()),
                    Instruction::SetCC(CC::E, unary_op_dst.clone())
                ]
            }
            else {
                vec![
                    Instruction::Mov(dst_ass_type.clone(), Operand::Imm(0), unary_op_dst.clone()),
                    Instruction::Cmp(src_ass_type.clone(), unary_op_src, Operand::Imm(0)),
                    Instruction::SetCC(CC::E, unary_op_dst.clone())
                ]
            }
         //   _ => { panic!("codegen::generate_code_for_unary_instruction: Unimplemented Unary Operand: {:?}", tacky_unary_op); }
    };

    instructions.append(&mut unary_op_instructions);

    Ok(())
}



fn generate_code_for_remainder_instruction(
    is_signed_int: bool,
    src1: &tacky::ast::Val,
    src2: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1_ass_type = get_tacky_value_assembly_type(src1, symbol_table);

    let src1 = convert_tacky_value_to_operand(src1)?;
    let src2 = convert_tacky_value_to_operand(src2)?;
    let dst  = convert_tacky_value_to_operand(dst)?;

    let mut div_instructions =
    if is_signed_int {
        vec![
            Instruction::Mov(src1_ass_type.clone(), src1.clone(), Operand::Reg(Register::AX)),
            Instruction::Cdq(src1_ass_type.clone()),
            Instruction::Idiv(src1_ass_type.clone(), src2.clone()),
            Instruction::Mov(src1_ass_type.clone(), Operand::Reg(Register::DX), dst.clone())
        ]
    }
    else {
        vec![
            Instruction::Mov(src1_ass_type.clone(), src1.clone(), Operand::Reg(Register::AX)),
            Instruction::Mov(src1_ass_type.clone(), Operand::Imm(0), Operand::Reg(Register::DX)),
            Instruction::Div(src1_ass_type.clone(), src2.clone()),
            Instruction::Mov(src1_ass_type.clone(), Operand::Reg(Register::DX), dst.clone())
        ]
    };

    instructions.append(&mut div_instructions);

    Ok(())
}


fn generate_code_for_divide_instruction(
    is_signed_int: bool,
    src1: &tacky::ast::Val,
    src2: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1_ass_type = get_tacky_value_assembly_type(src1, symbol_table);

    let src1 = convert_tacky_value_to_operand(src1)?;
    let src2 = convert_tacky_value_to_operand(src2)?;
    let dst  = convert_tacky_value_to_operand(dst)?;

    let mut div_instructions = if src1_ass_type == AssemblyType::Double {
        vec![
            Instruction::Mov(AssemblyType::Double, src1.clone(), dst.clone()),
            Instruction::Binary(BinaryOperator::DivDouble, AssemblyType::Double, src2.clone(), dst.clone())
        ]
    }
    else if is_signed_int {
        vec![
            Instruction::Mov(src1_ass_type.clone(), src1.clone(), Operand::Reg(Register::AX)),
            Instruction::Cdq(src1_ass_type.clone()),
            Instruction::Idiv(src1_ass_type.clone(), src2.clone()),
            Instruction::Mov(src1_ass_type.clone(), Operand::Reg(Register::AX), dst.clone())
        ]
    }
    else {
        vec![
            Instruction::Mov(src1_ass_type.clone(), src1.clone(), Operand::Reg(Register::AX)),
            Instruction::Mov(src1_ass_type.clone(), Operand::Imm(0), Operand::Reg(Register::DX)),
            Instruction::Div(src1_ass_type.clone(), src2.clone()),
            Instruction::Mov(src1_ass_type.clone(), Operand::Reg(Register::AX), dst.clone())
        ]
    };

    instructions.append(&mut div_instructions);

    Ok(())
}


fn generate_code_for_shift_left_instruction(
    src: &tacky::ast::Val,
    count: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_ass_type = get_tacky_value_assembly_type(src, symbol_table);

    let src = convert_tacky_value_to_operand(src)?;
    let count = convert_tacky_value_to_operand(count)?;
    let dst  = convert_tacky_value_to_operand(dst)?;

    instructions.push(Instruction::Mov(src_ass_type.clone(), src.clone(), dst.clone()));
    instructions.push(Instruction::Shl(src_ass_type.clone(), count.clone(), dst.clone()));

    Ok(())
}

fn generate_code_for_shift_right_instruction(
    is_signed: bool,
    src: &tacky::ast::Val,
    count: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_ass_type = get_tacky_value_assembly_type(src, symbol_table);

    let src = convert_tacky_value_to_operand(src)?;
    let count = convert_tacky_value_to_operand(count)?;
    let dst  = convert_tacky_value_to_operand(dst)?;

    instructions.push(Instruction::Mov(src_ass_type.clone(), src.clone(), dst.clone()));
    if is_signed {
        instructions.push(Instruction::Shra(src_ass_type.clone(), count.clone(), dst.clone()));
    }
    else {
        instructions.push(Instruction::Shrl(src_ass_type.clone(), count.clone(), dst.clone()));
    }

    Ok(())
}


//add, sub, mul, and, or, xor
fn generate_code_for_binary_instruction(
    bin_op: &BinaryOperator,
    src1: &tacky::ast::Val,
    src2: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src1_ass_type = get_tacky_value_assembly_type(src1, symbol_table);

    let src1 = convert_tacky_value_to_operand(src1)?;
    let src2 = convert_tacky_value_to_operand(src2)?;
    let dst  = convert_tacky_value_to_operand(dst)?;

    instructions.push(Instruction::Mov(src1_ass_type.clone(), src1.clone(), dst.clone()));
    instructions.push(Instruction::Binary(bin_op.clone(), src1_ass_type.clone(), src2.clone(), dst.clone()));

    Ok(())
}


fn generate_code_for_condition(
    cc: &CC,
    src1: &tacky::ast::Val,
    src2: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_ass_type = get_tacky_value_assembly_type(src1, symbol_table);
    let cmp_src = convert_tacky_value_to_operand(src1)?;
    let cmp_dst = convert_tacky_value_to_operand(src2)?;
    let dst  = convert_tacky_value_to_operand(dst)?;

    instructions.push(Instruction::Mov(AssemblyType::LongWord, Operand::Imm(0), dst.clone()));
    instructions.push(Instruction::Cmp(src_ass_type.clone(), cmp_src.clone(), cmp_dst.clone()));
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
    let is_src1_signed_int = is_tacky_value_signed(src1, symbol_table);
    let is_src2_signed_int = is_tacky_value_signed(src2, symbol_table);
    match bin_op {
        tacky::ast::BinaryOperator::ShiftLeft |
        tacky::ast::BinaryOperator::ShiftRight => {
            assert!(is_src2_signed_int);
        },
        _ => {
            assert_eq!(is_src1_signed_int, is_src2_signed_int);
        }
    }
    let is_signed_int = is_src1_signed_int;

    let result = match bin_op {
        tacky::ast::BinaryOperator::Add => generate_code_for_binary_instruction(&BinaryOperator::Add, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Subtract => generate_code_for_binary_instruction(&BinaryOperator::Sub, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Multiply => generate_code_for_binary_instruction(&BinaryOperator::Mul, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Divide => generate_code_for_divide_instruction(is_signed_int, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Remainder => generate_code_for_remainder_instruction(is_signed_int, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::BitwiseAnd => generate_code_for_binary_instruction(&BinaryOperator::And, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::BitwiseOr => generate_code_for_binary_instruction(&BinaryOperator::Or, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::BitwiseXor => generate_code_for_binary_instruction(&BinaryOperator::Xor, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::ShiftLeft => generate_code_for_shift_left_instruction(&src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::ShiftRight => generate_code_for_shift_right_instruction(is_signed_int, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::Equal => generate_code_for_condition(&CC::E, &src1, &src2, &dst, symbol_table, instructions)?,
        tacky::ast::BinaryOperator::NotEqual => generate_code_for_condition(&CC::NE, &src1, &src2, &dst, symbol_table, instructions)?,

        //Please note that the cmp expression subtracts src1 from src2, so the conditions are the 'opposite' of the operator
        tacky::ast::BinaryOperator::LessThan => {
            let cc = if is_signed_int { CC::G } else { CC::A };
            generate_code_for_condition(&cc, &src1, &src2, &dst, symbol_table, instructions)?
        },
        tacky::ast::BinaryOperator::LessOrEqual => {
            let cc = if is_signed_int { CC::GE } else { CC::AE };
            generate_code_for_condition(&cc, &src1, &src2, &dst, symbol_table, instructions)?
        },
        tacky::ast::BinaryOperator::GreaterThan => {
            let cc = if is_signed_int { CC::L } else { CC::B };
            generate_code_for_condition(&cc, &src1, &src2, &dst, symbol_table, instructions)?
        },
        tacky::ast::BinaryOperator::GreaterOrEqual => {
            let cc = if is_signed_int { CC::LE } else { CC::BE };
            generate_code_for_condition(&cc, &src1, &src2, &dst, symbol_table, instructions)?
        }

        // _ => { panic!("codegen::generate_binary_instruction(): Unimplemented binop: {:?}", bin_op); }
    };

    Ok(result)
}

fn generate_code_for_tacky_copy_instruction(
    src: &tacky::ast::Val,
    dst: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_oprnd = convert_tacky_value_to_operand(src)?;
    let dst_oprnd = convert_tacky_value_to_operand(dst)?;
    let src_ass_type = get_tacky_value_assembly_type(src, symbol_table);

    instructions.push(Instruction::Mov(src_ass_type, src_oprnd, dst_oprnd));

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
    let cmp_dst = convert_tacky_value_to_operand(val)?;
    let cmp_dst_ass_type = get_tacky_value_assembly_type(val, symbol_table);

    if cmp_dst_ass_type == AssemblyType::Double {
        instructions.push(Instruction::Binary(BinaryOperator::Xor, AssemblyType::Double, Operand::Reg(Register::XMM0), Operand::Reg(Register::XMM0)));
        instructions.push(Instruction::Cmp(AssemblyType::Double, Operand::Reg(Register::XMM0), cmp_dst));
    }
    else {
        instructions.push(Instruction::Cmp(cmp_dst_ass_type, Operand::Imm(0), cmp_dst));
    }

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


struct ClassifiedParameters {
    int_reg_args: Vec<(AssemblyType, Operand)>,
    double_reg_args: Vec<Operand>,
    stack_args: Vec<(AssemblyType, Operand)>
}

fn classify_parameters(
    args: &Vec<tacky::ast::Val>,
    int_args_registers: &[Register],
    double_args_registers: &[Register],
    symbol_table: &HashMap<String, SymbolInfo>) -> Result<ClassifiedParameters, String>
{
    let mut int_reg_args = vec![];
    let mut double_reg_args = vec![];
    let mut stack_args = vec![];

    for arg in args {
        let oprnd = convert_tacky_value_to_operand(arg)?;
        let typ = get_tacky_value_assembly_type(arg, symbol_table);

        let typed_arg = (typ.clone(), oprnd.clone());

        if typ == AssemblyType::Double {
            if double_reg_args.len() < double_args_registers.len() {
                double_reg_args.push(oprnd.clone());
            }
            else {
                stack_args.push(typed_arg);
            }
        }
        else if int_reg_args.len() < int_args_registers.len() {
            int_reg_args.push(typed_arg);
        }
        else {
            stack_args.push(typed_arg);
        }
    }

    Ok(ClassifiedParameters { int_reg_args , double_reg_args, stack_args })
}




fn generate_code_for_tacky_function_call(
    func_name: &String,
    args: &Vec<tacky::ast::Val>,
    ret_val: &tacky::ast::Val,
    symbol_table: &HashMap<String, SymbolInfo>,
    instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    // First 6 integer arguments are found in the registers below
    let int_args_registers = [
        Register::DI,
        Register::SI,
        Register::DX,
        Register::CX,
        Register::R8,
        Register::R9
    ];


    //First 8 double arguments are found in the registers below
    let double_args_registers = [
        Register::XMM0,
        Register::XMM1,
        Register::XMM2,
        Register::XMM3,
        Register::XMM4,
        Register::XMM5,
        Register::XMM6,
        Register::XMM7
    ];

    // The following arguments are pushed onto stack, in reverse order

    let classified_args =
        classify_parameters(
            args,
            &int_args_registers,
            &double_args_registers,
            &symbol_table)?;

    let (int_reg_args, double_reg_args, stack_args) = (classified_args.int_reg_args, classified_args.double_reg_args, classified_args.stack_args);

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
    for (ass_type, ass_arg) in int_reg_args {
        instructions.push(Instruction::Mov(ass_type, ass_arg, Operand::Reg(int_args_registers[reg_index].clone())));
        reg_index += 1;
    }

    let mut reg_index = 0;
    for ass_double_arg in double_reg_args {
        instructions.push(Instruction::Mov(AssemblyType::Double, ass_double_arg, Operand::Reg(double_args_registers[reg_index].clone())));
        reg_index += 1;
    }


    for (ass_type, ass_arg) in stack_args.iter().rev() {
        match ass_arg {
            Operand::Imm(_) |
            Operand::Reg(_) => {
                instructions.push(Instruction::Push(ass_arg.clone()));
            },
            _ => {
                match ass_type {
                    AssemblyType::QuadWord |
                    AssemblyType::Double => {
                        instructions.push(Instruction::Push(ass_arg.clone()));
                    },
                    AssemblyType::LongWord => {
                        instructions.push(Instruction::Mov(AssemblyType::LongWord, ass_arg.clone(), Operand::Reg(Register::AX)));
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
    let assembly_dst_ass_type = get_tacky_value_assembly_type(ret_val, symbol_table);

    if assembly_dst_ass_type == AssemblyType::Double {
        instructions.push(Instruction::Mov(AssemblyType::Double, Operand::Reg(Register::XMM0), assembly_dst));
    }
    else {
        instructions.push(Instruction::Mov(assembly_dst_ass_type, Operand::Reg(Register::AX), assembly_dst));
    }


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

fn generate_code_for_tacky_zero_extend(src: &tacky::ast::Val, dst: &tacky::ast::Val, _symbol_table: &HashMap<String, SymbolInfo>, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_op = convert_tacky_value_to_operand(src)?;
    let dst_op = convert_tacky_value_to_operand(dst)?;
    instructions.push(Instruction::MovZeroExtend(src_op, dst_op));

    Ok(())
}

fn generate_code_for_tacky_int_to_double(src: &tacky::ast::Val, dst: &tacky::ast::Val, symbol_table: &HashMap<String, SymbolInfo>, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_op = convert_tacky_value_to_operand(src)?;
    let dst_op = convert_tacky_value_to_operand(dst)?;
    let ass_type = get_tacky_value_assembly_type(src, symbol_table);

    instructions.push(Instruction::Cvtsi2sd(ass_type, src_op, dst_op));

    Ok(())
}

fn generate_code_for_tacky_uint_to_double(src: &tacky::ast::Val, dst: &tacky::ast::Val, symbol_table: &HashMap<String, SymbolInfo>, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_op = convert_tacky_value_to_operand(src)?;
    let dst_op = convert_tacky_value_to_operand(dst)?;
    let ass_type = get_tacky_value_assembly_type(src, symbol_table);

    match ass_type {
        AssemblyType::LongWord => {
            instructions.push(Instruction::MovZeroExtend(src_op, Operand::Reg(Register::AX)));
            instructions.push(Instruction::Cvtsi2sd(AssemblyType::QuadWord, Operand::Reg(Register::AX), dst_op));
        },
        AssemblyType::QuadWord => {
            let label_1 = make_unique_label(&String::from("uint_to_double_out_of_range"));
            let label_2 = make_unique_label(&String::from("uint_to_double_end"));

            instructions.push(Instruction::Cmp(AssemblyType::QuadWord, Operand::Imm(0), src_op.clone()));
            instructions.push(Instruction::JmpCC(CC::L, label_1.clone()));
            instructions.push(Instruction::Cvtsi2sd(AssemblyType::QuadWord, src_op.clone(), dst_op.clone()));
            instructions.push(Instruction::Jmp(label_2.clone()));
            instructions.push(Instruction::Label(label_1.clone()));
            instructions.push(Instruction::Mov(AssemblyType::QuadWord, src_op.clone(), Operand::Reg(Register::AX)));
            instructions.push(Instruction::Mov(AssemblyType::QuadWord, Operand::Reg(Register::AX), Operand::Reg(Register::DX)));
            instructions.push(Instruction::Shrl(AssemblyType::QuadWord, Operand::Imm(1), Operand::Reg(Register::DX)));
            instructions.push(Instruction::Binary(BinaryOperator::And, AssemblyType::QuadWord, Operand::Imm(1), Operand::Reg(Register::AX)));
            instructions.push(Instruction::Binary(BinaryOperator::Or, AssemblyType::QuadWord, Operand::Reg(Register::AX), Operand::Reg(Register::DX)));
            instructions.push(Instruction::Cvtsi2sd(AssemblyType::QuadWord, Operand::Reg(Register::DX), dst_op.clone()));
            instructions.push(Instruction::Binary(BinaryOperator::Add, AssemblyType::Double, dst_op.clone(), dst_op.clone()));
            instructions.push(Instruction::Label(label_2));
        }
        AssemblyType::Double => {
            panic!("generate_code_for_tacky_uint_to_double(): Attempt to convert from double");
        }
    }

    Ok(())
}


fn generate_code_for_tacky_double_to_int(src: &tacky::ast::Val, dst: &tacky::ast::Val, symbol_table: &HashMap<String, SymbolInfo>, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_op = convert_tacky_value_to_operand(src)?;
    let dst_op = convert_tacky_value_to_operand(dst)?;
    let ass_type = get_tacky_value_assembly_type(dst, symbol_table);

    instructions.push(Instruction::Cvttsd2si(ass_type.clone(), src_op.clone(), dst_op.clone()));

    Ok(())
}


fn generate_code_for_tacky_double_to_uint(src: &tacky::ast::Val, dst: &tacky::ast::Val, symbol_table: &HashMap<String, SymbolInfo>, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    let src_op = convert_tacky_value_to_operand(src)?;
    let dst_op = convert_tacky_value_to_operand(dst)?;
    let ass_type = get_tacky_value_assembly_type(dst, symbol_table);

    match ass_type {
        AssemblyType::LongWord => {
            instructions.push(Instruction::Cvttsd2si(AssemblyType::QuadWord, src_op.clone(), Operand::Reg(Register::AX)));
            instructions.push(Instruction::Mov(AssemblyType::LongWord, Operand::Reg(Register::AX), dst_op.clone()));
        },

        AssemblyType::QuadWord => {
            let out_of_range_label = make_unique_label(&String::from("out_of_range"));
            let end_label = make_unique_label(&String::from("end"));
            let upper_bound = make_double_constant(9223372036854775808.0);

            instructions.push(Instruction::Cmp(AssemblyType::Double, Operand::Data(upper_bound.clone()), src_op.clone()));
            instructions.push(Instruction::JmpCC(CC::AE, out_of_range_label.clone()));
            instructions.push(Instruction::Cvttsd2si(AssemblyType::QuadWord, src_op.clone(), dst_op.clone()));
            instructions.push(Instruction::Jmp(end_label.clone()));
            instructions.push(Instruction::Label(out_of_range_label.clone()));
            instructions.push(Instruction::Mov(AssemblyType::Double, src_op.clone(), Operand::Reg(Register::XMM0)));
            instructions.push(Instruction::Binary(BinaryOperator::Sub, AssemblyType::Double, Operand::Data(upper_bound.clone()), Operand::Reg(Register::XMM0)));
            instructions.push(Instruction::Cvttsd2si(AssemblyType::QuadWord, Operand::Reg(Register::XMM0), dst_op.clone()));
            instructions.push(Instruction::Mov(AssemblyType::QuadWord, Operand::Imm(9223372036854775808u64 as i64), Operand::Reg(Register::AX)));
            instructions.push(Instruction::Binary(BinaryOperator::Add, AssemblyType::QuadWord, Operand::Reg(Register::AX), dst_op.clone()));
            instructions.push(Instruction::Label(end_label.clone()));
        },

        AssemblyType::Double => {
            panic!("generate_code_for_tacky_double_to_uint(): Attempt to convert to double");
        }
    }

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
            },
            tacky::ast::Instruction::ZeroExtend(src, dst ) => {
                generate_code_for_tacky_zero_extend(src, dst, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::IntToDouble(src, dst) => {
                generate_code_for_tacky_int_to_double(src, dst, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::UIntToDouble(src, dst) => {
                generate_code_for_tacky_uint_to_double(src, dst, symbol_table, instructions)?;
            },
            tacky::ast::Instruction::DoubleToInt(src, dst) => {
                generate_code_for_tacky_double_to_int(src, dst, symbol_table, instructions)?;
            }
            tacky::ast::Instruction::DoubleToUInt(src, dst) => {
                generate_code_for_tacky_double_to_uint(src, dst, symbol_table, instructions)?;
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
    let int_params_registers = [
        Register::DI,
        Register::SI,
        Register::DX,
        Register::CX,
        Register::R8,
        Register::R9
    ];

    let double_params_registers = [
        Register::XMM0,
        Register::XMM1,
        Register::XMM2,
        Register::XMM3,
        Register::XMM4,
        Register::XMM5,
        Register::XMM6,
        Register::XMM7
    ];

    let mapped_params: Vec<_> =params.into_iter().map(|elem| tacky::ast::Val::Var(elem.clone())).collect();

    let classified_parameters = classify_parameters(&mapped_params, &int_params_registers, &double_params_registers, symbol_table)?;

    let (int_reg_params, double_reg_params, stack_params) = (classified_parameters.int_reg_args, classified_parameters.double_reg_args, classified_parameters.stack_args);

    let mut reg_idx = 0;
    for (int_ass_type, int_reg_op) in int_reg_params {
        new_instructions.push(Instruction::Mov(int_ass_type.clone(), Operand::Reg(int_params_registers[reg_idx].clone()), int_reg_op.clone()));
        reg_idx += 1;
    }

    let mut reg_idx = 0;
    for double_reg_op in double_reg_params {
        new_instructions.push(Instruction::Mov(AssemblyType::Double, Operand::Reg(double_params_registers[reg_idx].clone()), double_reg_op.clone()));
        reg_idx += 1;
    }

    let mut stack_idx = 16i64;
    for (ass_stack_type, ass_stack_param) in stack_params {
        new_instructions.push(Instruction::Mov(ass_stack_type, Operand::Stack(stack_idx as i64), ass_stack_param.clone()));
        stack_idx += 8;
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

    let mut top_level_double_constants = get_double_constant_top_level_items();


    let mut assembly_symbol_table = HashMap::new();
    for (symbol, symbol_info) in symbol_table {
        assembly_symbol_table.insert(symbol.clone(), symbol_info_to_assembly_symbol_info(symbol_info));
    }

    for top_level_const_double in &top_level_double_constants {
        match top_level_const_double {
            TopLevel::StaticConstant(name, align, StaticInit::DoubleInit(_)) => {
                let ass_sym_info = AssemblySymbolInfo::ObjEntry(AssemblyType::Double, true, true);
                assembly_symbol_table.insert(name.clone(), ass_sym_info);
            },
            _ => {
                panic!("Non-double constant found in top_level_double_constants: '{:?}'", top_level_const_double);
            }
        }
    }

    top_level_items.append(&mut top_level_double_constants);

    Ok((Program::ProgramDefinition(top_level_items), assembly_symbol_table))
}