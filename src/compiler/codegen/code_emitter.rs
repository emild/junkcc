use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::io::BufWriter;
use super::ast::*;
use super::super::parser::StaticInit;


enum OperandSize {
    Byte,
    Word,
    Dword,
    Qword
}


fn get_operand_size(ass_type: &AssemblyType) -> OperandSize
{
    match ass_type {
        AssemblyType::LongWord => OperandSize::Dword,
        AssemblyType::QuadWord => OperandSize::Qword
    }
}


fn emit_instruction_suffix(ass_type: &AssemblyType) -> String
{
    let suffix = match ass_type {
        AssemblyType::LongWord => "l",
        AssemblyType::QuadWord => "q"
    };

    String::from(suffix)
}


fn emit_byte_register(reg: &Register) -> std::io::Result<String>
{
    let reg_str = match reg {
        Register::AX  => "al",
        Register::BX  => "bl",
        Register::CX  => "cl",
        Register::DX  => "dl",
        Register::SP |
        Register::SI |
        Register::DI  => { return Err(std::io::Error::other(format!("Register '{:?}' cannot be used as byte operand", reg))); }
        Register::R8  => "r8b",
        Register::R9  => "r9b",
        Register::R10 => "r10b",
        Register::R11 => "r11b"
    };

    Ok(reg_str.to_string())
}

fn emit_word_register(reg: &Register) -> std::io::Result<String>
{
    let reg_str = match reg {
        Register::AX  => "ax",
        Register::BX  => "bx",
        Register::CX  => "cx",
        Register::DX  => "dx",
        Register::SP  => "sp",
        Register::SI  => "si",
        Register::DI  => "di",
        Register::R8  => "r8w",
        Register::R9  => "r9w",
        Register::R10 => "r10w",
        Register::R11 => "r11w"
    };

    Ok(reg_str.to_string())
}


fn emit_dword_register(reg: &Register) -> std::io::Result<String>
{
    let reg_str = match reg {
        Register::AX  => "eax",
        Register::BX  => "ebx",
        Register::CX  => "ecx",
        Register::DX  => "edx",
        Register::SP  => "esp",
        Register::SI  => "esi",
        Register::DI  => "edi",
        Register::R8  => "r8d",
        Register::R9  => "r9d",
        Register::R10 => "r10d",
        Register::R11 => "r11d"
    };

    Ok(reg_str.to_string())
}


fn emit_qword_register(reg: &Register) -> std::io::Result<String>
{
    let reg_str = match reg {
        Register::AX  => "rax",
        Register::BX  => "rbx",
        Register::CX  => "rcx",
        Register::DX  => "rdx",
        Register::SP  => "rsp",
        Register::SI  => "rsi",
        Register::DI  => "rdi",
        Register::R8  => "r8",
        Register::R9  => "r9",
        Register::R10 => "r10",
        Register::R11 => "r11"
    };

    Ok(reg_str.to_string())
}


fn emit_register(reg: &Register, op_size: &OperandSize) -> std::io::Result<String>
{
    let reg_str = match op_size {
        OperandSize::Byte => emit_byte_register(reg)?,
        OperandSize::Word => emit_word_register(reg)?,
        OperandSize::Dword => emit_dword_register(reg)?,
        OperandSize::Qword => emit_qword_register(reg)?
    };

    Ok(reg_str)
}


fn emit_operand(op: &Operand, op_size: &OperandSize, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match op {
        Operand::Reg(r) => {
            let reg_str = emit_register(r, op_size)?;
            write!(buf_writer, "%{}", reg_str)?;
            Ok(())
        },
        Operand::Imm(c) => {
            write!(buf_writer, "${}", c)?;
            Ok(())
        },
        Operand::Stack(stack_idx) => {
            write!(buf_writer, "{}(%rbp)", stack_idx)?;
            Ok(())
        },
        Operand::Data(global_var_name) => {
            write!(buf_writer, "{}(%rip)", global_var_name)?;
            Ok(())
        }
        _ => { return Err(std::io::Error::other(format!("Emit Code: Unsupported operand '{:?}'", op))); }
    }
}

fn emit_unary_operator(unary_operator: &UnaryOperator, ass_type: &AssemblyType, dst: &Operand, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let operator_str = match unary_operator {
        UnaryOperator::Neg => "neg",
        UnaryOperator::Not => "not",
        _ => { return Err(std::io::Error::other(format!("Emit Code: Unsupported unary operand, got '{:?}'", unary_operator))); }
    };

    let op_size = get_operand_size(ass_type);
    write!(buf_writer, "{}{}{} ", " ".repeat(16), operator_str, instruction_suffix(ass_type))?;
    emit_operand(dst, &op_size, buf_writer)?;
    writeln!(buf_writer, "")?;

    Ok(())
}

fn emit_binary_operator(binary_operator: &BinaryOperator, ass_type: &AssemblyType, src: &Operand, dst: &Operand, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let op_size = get_operand_size(ass_type);
    let (operator_str, src_size, dst_size) = match binary_operator {
        BinaryOperator::Add => ("add",  &op_size, &op_size),
        BinaryOperator::Sub => ("sub",  &op_size, &op_size),
        BinaryOperator::Mul => ("imul", &op_size, &op_size),
        BinaryOperator::And => ("and",  &op_size, &op_size),
        BinaryOperator::Or  => ("or",   &op_size, &op_size),
        BinaryOperator::Xor => ("xor",  &op_size, &op_size),
        BinaryOperator::Shl => ("sal",  &OperandSize::Byte, &op_size),
        BinaryOperator::Shr => ("sar",  &OperandSize::Byte, &op_size),
        _ => { return Err(std::io::Error::other(format!("Emit Code: Unsupported binary operand, got '{:?}'", binary_operator))); }
    };

    write!(buf_writer, "{}{}{} ", " ".repeat(16), operator_str, instruction_suffix(ass_type))?;
    emit_operand(src, &src_size, buf_writer)?;
    write!(buf_writer, ", ")?;
    emit_operand(dst, &dst_size, buf_writer)?;
    writeln!(buf_writer, "")?;

    Ok(())
}


fn emit_cc(cc: &CC, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let cc_str = match cc {
        CC::E  => "e",
        CC::NE => "ne",
        CC::L  => "l",
        CC::LE => "le",
        CC::G  => "g",
        CC::GE => "ge"
    };

    write!(buf_writer, "{}", cc_str)?;

    Ok(())
}


fn emit_label(label: &String, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    write!(buf_writer, "L.{}", label)?;

    Ok(())
}


fn emit_setcc_instruction(cc: &CC, dest: &Operand, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    write!(buf_writer, "set")?;
    emit_cc(&cc, buf_writer)?;
    write!(buf_writer, " ")?;
    emit_operand(dest, &OperandSize::Byte, buf_writer)?;
    writeln!(buf_writer, "")?;

    Ok(())
}


fn emit_jmpcc_instruction(cc: &CC, label: &String, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    write!(buf_writer, "j")?;
    emit_cc(&cc, buf_writer)?;
    write!(buf_writer, " ")?;
    emit_label(label, buf_writer)?;
    writeln!(buf_writer, "")?;

    Ok(())
}

fn emit_call_instruction(label: &String, assembly_symbol_table: &HashMap<String, AssemblySymbolInfo>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let mut target_label = String::new();
    let func_type = assembly_symbol_table.get(label);
    match func_type {
        Some(AssemblySymbolInfo::FuncEntry(true)) => {
            //Locally defined function
            target_label = label.clone();
        },
        Some(AssemblySymbolInfo::FuncEntry(false)) => {
            target_label = format!("{}@PLT", label);
        },
        Some(AssemblySymbolInfo::ObjEntry(_,_)) => {
            panic!("code_emitter: Attempt to call function '{}()', but symbol table says it's not a function", label);
        }
        None => {
            panic!("code_emitter: Function '{}()' not found in assembly symbol table", label);
        }
    };

    writeln!(buf_writer, "call {}", target_label)?;

    Ok(())
}


fn instruction_suffix(ass_type: &AssemblyType) -> &str
{
    match ass_type {
        AssemblyType::LongWord => "l",
        AssemblyType::QuadWord => "q"
    }
}


fn emit_body(instructions: &Vec<Instruction>, assembly_symbol_table: &HashMap<String, AssemblySymbolInfo>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    for ins in instructions {
        match ins {
            Instruction::Mov(ass_type, src, dest ) => {
                let op_size = get_operand_size(ass_type);
                write!(buf_writer, "{}mov{} ", " ".repeat(16), instruction_suffix(ass_type))?;
                emit_operand(&src, &op_size, buf_writer)?;
                write!(buf_writer, ", ")?;
                emit_operand(&dest, &op_size, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::Movsx(src, dst ) => {
                let src_ass_type = AssemblyType::LongWord;
                let dst_ass_type = AssemblyType::QuadWord;
                let src_op_size = get_operand_size(&src_ass_type);
                let dst_op_size = get_operand_size(&dst_ass_type);

                write!(
                    buf_writer,
                    "{}movs{}{} ",
                    " ".repeat(16),
                    instruction_suffix(&src_ass_type),
                    instruction_suffix(&dst_ass_type))?;
                emit_operand(src, &src_op_size, buf_writer)?;
                write!(buf_writer, ", ")?;
                emit_operand(dst, &dst_op_size, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::Push(src) => {
                write!(buf_writer, "{}pushq ", " ".repeat(16))?;
                emit_operand(src, &OperandSize::Qword, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::Ret => {
                //Epilog
                writeln!(buf_writer, "{}movq %rbp, %rsp", " ".repeat(16))?;
                writeln!(buf_writer, "{}popq %rbp", " ".repeat(16))?;
                writeln!(buf_writer, "{}ret", " ".repeat(16))?;
            },
            Instruction::Cdq(ass_type) => {
                match ass_type {
                    AssemblyType::LongWord => writeln!(buf_writer, "{}cdq", " ".repeat(16))?,
                    AssemblyType::QuadWord => writeln!(buf_writer, "{}cqo", " ".repeat(16))?
                };
            },
            Instruction::Unary(unary_operator, ass_type, dst) => {
                emit_unary_operator(unary_operator, ass_type, dst, buf_writer)?;
            },
            Instruction::Binary(binary_operator, ass_type, src, dst) => {
                emit_binary_operator(binary_operator, ass_type, src, dst, buf_writer)?;
            },
            Instruction::Idiv(ass_type, divisor) => {
                write!(buf_writer, "{}idiv{} ", " ".repeat(16), instruction_suffix(ass_type))?;
                let op_size = get_operand_size(ass_type);
                emit_operand(divisor, &op_size, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::Cmp(ass_type, src1, src2) => {
                let op_size = get_operand_size(ass_type);
                write!(buf_writer, "{}cmp{} ", " ".repeat(16), instruction_suffix(ass_type))?;
                emit_operand(src1, &op_size, buf_writer)?;
                write!(buf_writer, ", ")?;
                emit_operand(src2, &op_size, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::SetCC(cc, dest ) => {
                write!(buf_writer, "{}", " ".repeat(16))?;
                emit_setcc_instruction(&cc, &dest, buf_writer)?;
            },
            Instruction::Jmp(label) => {
                write!(buf_writer, "{}jmp ", " ".repeat(16))?;
                emit_label(label, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::JmpCC(cc, label) => {
                write!(buf_writer, "{}", " ".repeat(16))?;
                emit_jmpcc_instruction(cc, label, buf_writer)?;
            },
            Instruction::Call(label) => {
                write!(buf_writer, "{}", " ".repeat(16))?;
                emit_call_instruction(label, assembly_symbol_table, buf_writer)?;
            }
            Instruction::Label(label) => {
                writeln!(buf_writer, "L.{}:", label)?;
            },
          //  _ => {
          //      return Err(std::io::Error::other(format!("Unsupported instruction '{:?}'", ins)));
          //  }
        };
    }
    Ok(())
}

fn emit_zero_static_init(init_value: &StaticInit, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let op_size = match init_value {
        StaticInit::IntInit(0) => 4,
        StaticInit::LongInit(0) => 8,
        _ => { panic!("Attempt to place non-zero value '{:?}' in .bss", init_value); }
    };

    writeln!(buf_writer, "{}.zero {}", " ".repeat(16), op_size)
}


fn emit_nonzero_static_init(init_value: &StaticInit, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match init_value {
        StaticInit::IntInit(c) =>
            writeln!(buf_writer, "{}.long {}", " ".repeat(16), c),
        StaticInit::LongInit(c) =>
            writeln!(buf_writer, "{}.quad {}", " ".repeat(16), c)
    }
}


fn emit_top_level_item(top_level_item: &TopLevel, assembly_symbol_table: &HashMap<String, AssemblySymbolInfo>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match top_level_item {
        TopLevel::Function(func_name, global, instructions) => {
            writeln!(buf_writer, "{}", "#".repeat(40))?;
            writeln!(buf_writer, "# FUNCTION: {}", func_name)?;
            writeln!(buf_writer, "{}\n", "#".repeat(40))?;
            if *global {
                writeln!(buf_writer, "{}.globl {}", " ".repeat(16), func_name)?;
                writeln!(buf_writer, "")?;
            }
            writeln!(buf_writer, "{}.text", " ".repeat(16))?;
            writeln!(buf_writer, "{}:", func_name)?;
            //(Pre(?)Prolog
            writeln!(buf_writer, "{}pushq %rbp", " ".repeat(16))?;
            writeln!(buf_writer, "{}movq %rsp, %rbp", " ".repeat(16))?;

            emit_body(instructions, assembly_symbol_table, buf_writer)?;

            writeln!(buf_writer, "")?;
            Ok(())
        },
        TopLevel::StaticVariable(var_name, global, alignment, init_value) => {
            writeln!(buf_writer, "{}", "#".repeat(40))?;
            writeln!(buf_writer, "# VARIABLE: {}", var_name)?;
            writeln!(buf_writer, "{}\n", "#".repeat(40))?;
            if *global {
                writeln!(buf_writer, "{}.globl {}", " ".repeat(16), var_name)?;
                writeln!(buf_writer, "")?;
            }
            if init_value.is_zero() {
                writeln!(buf_writer, "{}.bss", " ".repeat(16))?;
                writeln!(buf_writer, "{}.align {}", " ".repeat(16), alignment)?;
                writeln!(buf_writer, "{}:", var_name)?;
                emit_zero_static_init(init_value, buf_writer)?;
            }
            else {
                writeln!(buf_writer, "{}.data", " ".repeat(16))?;
                writeln!(buf_writer, "{}.align {}", " ".repeat(16), alignment)?;
                writeln!(buf_writer, "{}:", var_name)?;
                emit_nonzero_static_init(init_value, buf_writer)?;
            }

            Ok(())
        },
        _ => {
            Err(std::io::Error::other(format!("Unsupported function definition: '{:?}'", top_level_item)))
        }
    }
}



fn emit_program(program: &Program, assembly_symbol_table: &HashMap<String, AssemblySymbolInfo>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match program {
        Program::ProgramDefinition(func_defs) => {
            for func_def in func_defs {
                emit_top_level_item(&func_def, assembly_symbol_table, buf_writer)?;
            }
            Ok(())
        },
        //_ => Err(std::io::Error::other(format!("Unsupported Program Definiton: '{:?}'", program)))
    }
}


pub fn emit_code(program: &Program, assembly_symbol_table: &HashMap<String, AssemblySymbolInfo>, output_file_path: &str) -> std::io::Result<()>
{
    let file = fs::File::create(output_file_path)?;
    let mut buf_writer = BufWriter::new(file);

    emit_program(&program, assembly_symbol_table, &mut buf_writer)?;

    let stack_protection = ".section .note.GNU-stack,\"\",@progbits";
    writeln!(&mut buf_writer, "{}", stack_protection)?;

    Ok(())

}
