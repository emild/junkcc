use std::fs;
use std::io::Write;
use std::io::BufWriter;
use super::codegen::ast::*;


fn emit_operand(op: &Operand, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    match op {
        Operand::Register => {
            write!(buf_writer, "%eax")?;
            Ok(())
        },
        Operand::Imm(c) => {
            write!(buf_writer, "${}", c)?;
            Ok(())
        }
        _ => { return Err(std::io::Error::other(format!("Unsupported operand '{:?}'", op))); }
    }
}


fn emit_body(instructions: &Vec<Instruction>, buf_writer: &mut BufWriter<fs::File>) -> std::io::Result<()>
{
    for ins in instructions {
        match ins {
            Instruction::Mov(src, dest ) => {
                write!(buf_writer, "{}movl ", " ".repeat(16))?;
                emit_operand(&src, buf_writer)?;
                write!(buf_writer, ", ")?;
                emit_operand(&dest, buf_writer)?;
                writeln!(buf_writer, "")?;
            },
            Instruction::Ret => {
                writeln!(buf_writer, "{}ret", " ".repeat(16))?;
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
            writeln!(buf_writer, "{}.globl {}", " ".repeat(16), func_name)?;
            writeln!(buf_writer, "")?;
            writeln!(buf_writer, "{}:", func_name)?;
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