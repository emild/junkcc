use std::fs;
use std::io::Write;
use std::io::BufWriter;
use super::ast::*;


fn emit_operand(op: &Operand, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match op {
        Operand::Reg(r) => {
            let reg_str = match r {
                Register::AL => "al",
                Register::AX => "eax",
                Register::CL => "cl",
                Register::CX => "ecx",
                Register::DL => "dl",
                Register::DX => "edx",
                Register::R10B => "r10b",
                Register::R10 => "r10d",
                Register::R11B => "r11b",
                Register::R11 => "r11d",
                _ => { return Err(std::io::Error::other(format!("Code emit: Unsupported register: '{:?}'", r))); }
            };

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
    emit_operand(dst, buf_writer)?;
    writeln!(buf_writer, "")?;

    Ok(())
}

fn emit_binary_operator(binary_operator: &BinaryOperator, src: &Operand, dst: &Operand, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    let operator_str = match binary_operator {
        BinaryOperator::Add => "addl",
        BinaryOperator::Sub => "subl",
        BinaryOperator::Mul => "imull",
        BinaryOperator::And => "andl",
        BinaryOperator::Or  => "orl",
        BinaryOperator::Xor => "xorl",
        BinaryOperator::Shl => "sall",
        BinaryOperator::Shr => "sarl",
        _ => { return Err(std::io::Error::other(format!("Emit Code: Unsupported binary operand, got '{:?}'", binary_operator))); }
    };

    write!(buf_writer, "{}{} ", " ".repeat(16), operator_str)?;
    emit_operand(src, buf_writer)?;
    write!(buf_writer, ", ")?;
    emit_operand(dst, buf_writer)?;
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
    emit_operand(dest, buf_writer)?;
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


fn emit_body(instructions: &Vec<Instruction>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    for ins in instructions {
        match ins {
            Instruction::AllocateStack(stack_allocation_size) => {
                writeln!(buf_writer, "{}subq ${}, %rsp", " ".repeat(16), stack_allocation_size)?;
            },
            Instruction::Mov(src, dest ) => {
                write!(buf_writer, "{}movl ", " ".repeat(16))?;
                emit_operand(&src, buf_writer)?;
                write!(buf_writer, ", ")?;
                emit_operand(&dest, buf_writer)?;
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
                emit_operand(divisor, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::Cmp(src1, src2) => {
                write!(buf_writer, "{}cmpl ", " ".repeat(16))?;
                emit_operand(src1, buf_writer)?;
                write!(buf_writer, ", ")?;
                emit_operand(src2, buf_writer)?;
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
            Instruction::Label(label) => {
                writeln!(buf_writer, "L.{}:", label)?;
            },
            _ => {
                return Err(std::io::Error::other(format!("Unsupported instruction '{:?}'", ins)));
            }
        };
    }
    Ok(())
}



fn emit_function(f: &FunctionDefinition, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match f {
        FunctionDefinition::Function(func_name, instructions) => {
            writeln!(buf_writer, "{}", "#".repeat(40))?;
            writeln!(buf_writer, "# {}", func_name)?;
            writeln!(buf_writer, "{}\n", "#".repeat(40))?;
            writeln!(buf_writer, "{}.globl {}", " ".repeat(16), func_name)?;
            writeln!(buf_writer, "")?;
            writeln!(buf_writer, "{}:", func_name)?;
            //(Pre(?)Prolog
            writeln!(buf_writer, "{}pushq %rbp", " ".repeat(16))?;
            writeln!(buf_writer, "{}movq %rsp, %rbp", " ".repeat(16))?;

            emit_body(instructions, buf_writer)?;

            writeln!(buf_writer, "")?;
            Ok(())
        },
        _ => Err(std::io::Error::other(format!("Unsupported function definition: '{:?}'", f)))
    }
}



fn emit_program(program: &Program, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match program {
        Program::ProgramDefinition(f) => {
            emit_function(&f, buf_writer)?;
            Ok(())
        },
        _ => Err(std::io::Error::other(format!("Unsupported Program Definiton: '{:?}'", program)))
    }
}


pub fn emit_code(program: &Program, output_file_path: &str) -> std::io::Result<()>
{
    let file = fs::File::create(output_file_path)?;
    let mut buf_writer = BufWriter::new(file);

    emit_program(&program, &mut buf_writer)?;

    let stack_protection = ".section .note.GNU-stack,\"\",@progbits";
    writeln!(&mut buf_writer, "{}", stack_protection)?;

    Ok(())
}
