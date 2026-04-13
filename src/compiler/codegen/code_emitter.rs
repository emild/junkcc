use std::collections::HashMap;
use std::fs;
use std::fs::write;
use std::io::Write;
use std::io::BufWriter;
use super::ast::*;
use super::super::parser::Type;
use super::super::parser::IdentifierAttrs;
use super::super::parser::SymbolInfo;


enum OperandSize {
    Byte,
    Word,
    Dword,
    Qword
}


fn emit_byte_register(reg: &Register) -> std::io::Result<String>
{
    let reg_str = match reg {
        Register::AX  => "al",
        Register::BX  => "bl",
        Register::CX  => "cl",
        Register::DX  => "dl",
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

fn emit_unary_operator(unary_operator: &UnaryOperator, dst: &Operand, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let operator_str = match unary_operator {
        UnaryOperator::Neg => "negl",
        UnaryOperator::Not => "notl",
        _ => { return Err(std::io::Error::other(format!("Emit Code: Unsupported unary operand, got '{:?}'", unary_operator))); }
    };

    write!(buf_writer, "{}{} ", " ".repeat(16), operator_str)?;
    emit_operand(dst, &OperandSize::Dword, buf_writer)?;
    writeln!(buf_writer, "")?;

    Ok(())
}

fn emit_binary_operator(binary_operator: &BinaryOperator, src: &Operand, dst: &Operand, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let (operator_str, src_size, dst_size) = match binary_operator {
        BinaryOperator::Add => ("addl", OperandSize::Dword, OperandSize::Dword),
        BinaryOperator::Sub => ("subl", OperandSize::Dword, OperandSize::Dword),
        BinaryOperator::Mul => ("imull", OperandSize::Dword, OperandSize::Dword),
        BinaryOperator::And => ("andl", OperandSize::Dword, OperandSize::Dword),
        BinaryOperator::Or  => ("orl", OperandSize::Dword, OperandSize::Dword),
        BinaryOperator::Xor => ("xorl", OperandSize::Dword, OperandSize::Dword),
        BinaryOperator::Shl => ("sall", OperandSize::Byte, OperandSize::Dword),
        BinaryOperator::Shr => ("sarl", OperandSize::Byte, OperandSize::Dword),
        _ => { return Err(std::io::Error::other(format!("Emit Code: Unsupported binary operand, got '{:?}'", binary_operator))); }
    };

    write!(buf_writer, "{}{} ", " ".repeat(16), operator_str)?;
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

fn emit_call_instruction(label: &String, symbol_table: &HashMap<String, SymbolInfo>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let mut target_label = String::new();
    let func_type = symbol_table.get(label);
    match func_type {
        Some(SymbolInfo {typ: Type::FuncType(_, true), attrs:_}) => {
            //Locally defined function
            target_label = label.clone();
        },
        _ => {
            target_label = format!("{}@PLT", label);
        }
    };

    writeln!(buf_writer, "call {}", target_label)?;

    Ok(())
}


fn emit_body(instructions: &Vec<Instruction>, symbol_table: &HashMap<String, SymbolInfo>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    for ins in instructions {
        match ins {
            Instruction::AllocateStack(stack_allocation_size) => {
                writeln!(buf_writer, "{}subq ${}, %rsp", " ".repeat(16), stack_allocation_size)?;
            },
            Instruction::DeallocateStack(stack_allocation_size) => {
                writeln!(buf_writer, "{}addq ${}, %rsp", " ".repeat(16), stack_allocation_size)?;
            },
            Instruction::Mov(src, dest ) => {
                write!(buf_writer, "{}movl ", " ".repeat(16))?;
                emit_operand(&src, &OperandSize::Dword, buf_writer)?;
                write!(buf_writer, ", ")?;
                emit_operand(&dest, &OperandSize::Dword, buf_writer)?;
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
            Instruction::Cdq => {
                writeln!(buf_writer, "{}cdq", " ".repeat(16))?;
            },
            Instruction::Unary(unary_operator, dst) => {
                emit_unary_operator(unary_operator, dst, buf_writer)?;
            },
            Instruction::Binary(binary_operator, src, dst) => {
                emit_binary_operator(binary_operator, src, dst, buf_writer)?;
            },
            Instruction::Idiv(divisor) => {
                write!(buf_writer, "{}idivl ", " ".repeat(16))?;
                emit_operand(divisor, &OperandSize::Dword, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::Cmp(src1, src2) => {
                write!(buf_writer, "{}cmpl ", " ".repeat(16))?;
                emit_operand(src1, &OperandSize::Dword, buf_writer)?;
                write!(buf_writer, ", ")?;
                emit_operand(src2, &OperandSize::Dword, buf_writer)?;
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
                emit_call_instruction(label, symbol_table, buf_writer)?;
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



fn emit_function(top_level_item: &TopLevel, symbol_table: &HashMap<String, SymbolInfo>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
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

            emit_body(instructions, symbol_table, buf_writer)?;

            writeln!(buf_writer, "")?;
            Ok(())
        },
        TopLevel::StaticVariable(var_name, global, init_value) => {
            writeln!(buf_writer, "{}", "#".repeat(40))?;
            writeln!(buf_writer, "# VARIABLE: {}", var_name)?;
            writeln!(buf_writer, "{}\n", "#".repeat(40))?;
            if *global {
                writeln!(buf_writer, "{}.globl {}", " ".repeat(16), var_name)?;
                writeln!(buf_writer, "")?;
            }
            if *init_value == 0 {
                writeln!(buf_writer, "{}.bss", " ".repeat(16))?;
                writeln!(buf_writer, "{}.align 4", " ".repeat(16))?;
                writeln!(buf_writer, "{}:", var_name)?;
                writeln!(buf_writer, "{}.zero 4", " ".repeat(16))?;
            }
            else {
                writeln!(buf_writer, "{}.data", " ".repeat(16))?;
                writeln!(buf_writer, "{}.align 4", " ".repeat(16))?;
                writeln!(buf_writer, "{}:", var_name)?;
                writeln!(buf_writer, "{}.long {}", " ".repeat(16), init_value)?;
            }

            Ok(())
        },
        _ => {
            Err(std::io::Error::other(format!("Unsupported function definition: '{:?}'", top_level_item)))
        }
    }
}



fn emit_program(program: &Program, symbol_table: &HashMap<String, SymbolInfo>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match program {
        Program::ProgramDefinition(func_defs) => {
            for func_def in func_defs {
                emit_function(&func_def, symbol_table, buf_writer)?;
            }
            Ok(())
        },
        //_ => Err(std::io::Error::other(format!("Unsupported Program Definiton: '{:?}'", program)))
    }
}


pub fn emit_code(program: &Program, symbol_table: &HashMap<String, SymbolInfo>, output_file_path: &str) -> std::io::Result<()>
{
    let file = fs::File::create(output_file_path)?;
    let mut buf_writer = BufWriter::new(file);

    emit_program(&program, symbol_table, &mut buf_writer)?;

    let stack_protection = ".section .note.GNU-stack,\"\",@progbits";
    writeln!(&mut buf_writer, "{}", stack_protection)?;

    Ok(())
}
