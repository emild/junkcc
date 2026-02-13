use super::ast::*;

fn pretty_print_operand(op: &Operand)
{
    match op {
        Operand::Reg(reg) => {
            print!("REG({:?})", reg);
        },
        Operand::Imm(c) => {
            print!("IMM({})", c);
        },
        Operand::Pseudo(var_name) => {
            print!("PSEUDO({})", var_name);
        },
        Operand::Stack(idx) => {
            print!("STACK({})", idx);
        }
        _ => { panic!("Invalid operand: '{:?}'", op); }
    };
}

fn pretty_print_unary_operator(unary_op: &UnaryOperator)
{
    match unary_op {
        UnaryOperator::Neg => { print!("neg"); },
        UnaryOperator::Not => { print!("not"); },
        _ => { panic!("Unexpected unary operator: '{:?}'", unary_op); }
    }
}


fn pretty_print_binary_operator(binary_op: &BinaryOperator)
{
    match binary_op {
        BinaryOperator::Add => { print!("add"); },
        BinaryOperator::Sub =>  { print!("sub"); },
        BinaryOperator::Mul => { print!("mul"); },
        _ => { panic!("Unexpected binary operator: '{:?}'", binary_op); }
    }
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
            Instruction::AllocateStack(size) => {
                println!("{}AllocateStack({})", " ".repeat(indent), size);
            },
            Instruction::Unary(unary_op, dest) => {
                print!("{}", " ".repeat(indent));
                pretty_print_unary_operator(&unary_op);
                print!(" dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Binary(binary_op, src, dest) => {
                print!("{}", " ".repeat(indent));
                pretty_print_binary_operator(&binary_op);
                print!(" src=");
                pretty_print_operand(&src);
                print!(" dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Cdq => {
                println!("{}cdq", " ".repeat(indent));
            },
            Instruction::Idiv(divisor) => {
                print!("{}idiv divisor=", " ".repeat(indent));
                pretty_print_operand(divisor);
                println!("");
            }
            _ => { panic!("Unknown instruction: '{:?}'", ins); }
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
        _ => { panic!("Invalid function definiton: '{:?}'", f); }
    }   
}


fn pretty_print_program(p: &Program, indent: usize)
{
    println!("{}Program(", " ".repeat(indent));
    match p {
        Program::ProgramDefinition(f) => pretty_print_function(&f, indent + 4),
        _ => { panic!("Invalid program definition: '{:?}'", p); }
    };

    println!("{})", " ".repeat(indent));    
}


pub fn pretty_print_ast(program: &Program)
{
    pretty_print_program(&program, 0);
}
