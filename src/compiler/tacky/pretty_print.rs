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


fn pretty_print_tacky_binary_operator(binary_op: &BinaryOperator)
{
    match *binary_op {
        BinaryOperator::Add => {
            print!("ADD");
        },
        BinaryOperator::Subtract => {
            print!("SUBTRACT");
        },
        BinaryOperator::Multiply => {
            print!("MULTIPLY");
        },
        BinaryOperator::Divide => {
            print!("DIVIDE");
        },
        BinaryOperator::Remainder => {
            print!("REMAINDER");
        },
        BinaryOperator::BitwiseAnd => {
            print!("BINARY_AND")
        },
        BinaryOperator::BitwiseOr => {
            print!("BINARY_OR")
        },
        BinaryOperator::BitwiseXor => {
            print!("XOR")
        },
        BinaryOperator::ShiftLeft => {
            print!("SHIFT_LEFT")
        },
        BinaryOperator::ShiftRight => {
            print!("SHIFT_RIGHT")
        },
        BinaryOperator::Equal => {
            print!("CHECK_EQUAL")
        },
        BinaryOperator::NotEqual => {
            print!("CHECK_NOT_EQUAL")
        },
        BinaryOperator::LessThan => {
            print!("CHECK_LESS_THAN")
        },
        BinaryOperator::LessOrEqual => {
            print!("CHECK_LESS_OR_EQUAL")
        },
        BinaryOperator::GreaterThan => {
            print!("CHECK_GREATER_THAN")
        },
        BinaryOperator::GreaterOrEqual => {
            print!("CHECK_GREATER_OR_EQUAL")
        }

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
            Instruction::Binary(op, src1 , src2, dst) => {
                print!("{}", " ".repeat(indent));
                pretty_print_tacky_val(&dst);
                print!(" = ");
                pretty_print_tacky_binary_operator(&op);
                print!("(");
                pretty_print_tacky_val(&src1);
                print!(", ");
                pretty_print_tacky_val(&src2);
                println!(")");
            },
            Instruction::Copy(src, dst) => {
                print!("{}", " ".repeat(indent));
                pretty_print_tacky_val(&dst);
                print!(" = COPY(");
                pretty_print_tacky_val(&src);
                println!(")");
            },
            Instruction::Jump(target) => {
                print!("{}", " ".repeat(indent));
                println!("JUMP {}", target);
            },
            Instruction::JumpIfZero(value, target) => {
                print!("{}", " ".repeat(indent));
                print!("JUMP_IF_ZERO ");
                pretty_print_tacky_val(value);
                println!(", {}", target);
            },
            Instruction::JumpIfNotZero(value, target) => {
                print!("{}", " ".repeat(indent));
                print!("JUMP_IF_NOT_ZERO ");
                pretty_print_tacky_val(value);
                println!(", {}", target);
            },
            Instruction::Label(label) => {
                println!("{}:", label);
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


pub fn pretty_print_tacky_ast(program: &Program)
{
    pretty_print_tacky_program(&program, 0);
}
