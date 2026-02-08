use super::ast::*;

fn pretty_print_tacky_unary_operator(unary_op: &UnaryOperator)
{
    match *unary_op {
        UnaryOperator::Complement => {
            print!("COMPLEMENT");
        },
        UnaryOperator::Negate => {
            print!("NEGATE");
        },
        _ => {}
    };
}


fn pretty_print_tacky_val(val: &Val)
{
    match val {
        Val::IntConstant(c) => {
            print!("IntConstant({})", c);
        },
        Val::Var(var_name) => {
            print!("Var({})", var_name);
        },
        _ => {}
    };
}


fn pretty_print_tacky_instructions(instructions: &Vec<Instruction>, indent: usize)
{
    for ins in instructions {
        match ins {
            Instruction::Return(val) => {
                print!("{}Return(", " ".repeat(indent));
                pretty_print_tacky_val(&val);
                println!(")");
            },
            Instruction::Unary(op, src , dst) => {
                print!("{}", " ".repeat(indent));
                pretty_print_tacky_val(&dst);
                print!(" = ");
                pretty_print_tacky_unary_operator(&op);
                print!("(");
                pretty_print_tacky_val(&src);
                println!(")");
            },
            _ => {}
        };
    }
}

fn pretty_print_tacky_function(f: &FunctionDefinition, indent: usize)
{
    match f {
        FunctionDefinition::Function(func_name, instructions) => {
            println!("{}Function(", " ".repeat(indent));
            println!("{}name={func_name}", " ".repeat(indent + 4));
            println!("{}body=(", " ".repeat(indent + 4));
            pretty_print_tacky_instructions(instructions, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{})", " ".repeat(indent));
        },
        _ => ()
    }   
}


fn pretty_print_tacky_program(p: &Program, indent: usize)
{
    println!("{}Program(", " ".repeat(indent));
    match p {
        Program::ProgramDefinition(f) => pretty_print_tacky_function(&f, indent + 4),
        _ => ()
    };

    println!("{})", " ".repeat(indent));    
}


pub fn pretty_print_ast(program: &Program)
{
    pretty_print_tacky_program(&program, 0);
}
