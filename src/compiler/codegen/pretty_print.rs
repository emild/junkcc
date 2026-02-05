use super::ast::*;

fn pretty_print_operand(op: &Operand)
{
    match op {
        Operand::Register => {
            print!("REG");
        },
        Operand::Imm(c) => {
            print!("IMM({})", c);
        }
        _ => {}
    };
}


fn pretty_print_instructions(instructions: &Vec<Instruction>, indent: usize)
{
    for ins in instructions {
        match ins {
            Instruction::Mov(src, dest ) => {
                print!("{}mov src=", " ".repeat(indent));
                pretty_print_operand(&src);
                print!(", dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Ret => {
                println!("{}ret", " ".repeat(indent))
            },
            _ => {}
        };
    }
}

fn pretty_print_function(f: &FunctionDefinition, indent: usize)
{
    match f {
        FunctionDefinition::Function(func_name, instructions) => {
            println!("{}Function(", " ".repeat(indent));
            println!("{}name={func_name}", " ".repeat(indent + 4));
            println!("{}body=(", " ".repeat(indent + 4));
            pretty_print_instructions(instructions, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{})", " ".repeat(indent));
        },
        _ => ()
    }   
}


fn pretty_print_program(p: &Program, indent: usize)
{
    println!("{}Program(", " ".repeat(indent));
    match p {
        Program::ProgramDefinition(f) => pretty_print_function(&f, indent + 4),
        _ => ()
    };

    println!("{})", " ".repeat(indent));    
}


pub fn pretty_print_ast(program: &Program)
{
    pretty_print_program(&program, 0);
}
