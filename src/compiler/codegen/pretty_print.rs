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
        },
        Operand::Data(var_name) => {
            print!("DATA({})", var_name);
        }
        //,
        // => { panic!("Invalid operand: '{:?}'", op); }
    };
}

fn pretty_print_unary_operator(unary_op: &UnaryOperator)
{
    match unary_op {
        UnaryOperator::Neg => { print!("neg"); },
        UnaryOperator::Not => { print!("not"); },
        //_ => { panic!("Unexpected unary operator: '{:?}'", unary_op); }
    }
}


fn pretty_print_binary_operator(binary_op: &BinaryOperator)
{
    match binary_op {
        BinaryOperator::Add => { print!("add"); },
        BinaryOperator::Sub => { print!("sub"); },
        BinaryOperator::Mul => { print!("mul"); },
        BinaryOperator::And => { print!("and"); },
        BinaryOperator::Or  => { print!("or");  },
        BinaryOperator::Xor => { print!("xor"); },
        BinaryOperator::Shl => { print!("sal"); },
        BinaryOperator::Shr => { print!("sar"); },
        //_ => { panic!("Unexpected binary operator: '{:?}'", binary_op); }
    }
}


fn pretty_print_conditional_jump(cc: &CC)
{
    match cc {
        CC::E  => print!("je"),
        CC::NE => print!("jne"),
        CC::L  => print!("jl"),
        CC::LE => print!("jle"),
        CC::G  => print!("jg"),
        CC::GE => print!("jge")
    }
}

fn pretty_print_setcc(cc: &CC)
{
    match cc {
        CC::E  => print!("sete"),
        CC::NE => print!("setne"),
        CC::L  => print!("setl"),
        CC::LE => print!("setle"),
        CC::G  => print!("setg"),
        CC::GE => print!("setge")
    }
}

fn pretty_print_instructions(instructions: &Vec<Instruction>, indent: usize)
{
    for ins in instructions {
        match ins {
            Instruction::Mov(src, dest) => {
                print!("{}mov src=", " ".repeat(indent));
                pretty_print_operand(&src);
                print!(", dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Push(src) => {
                print!("{}push ", " ".repeat(indent));
                pretty_print_operand(src);
                println!("");
            },
            Instruction::Ret => {
                println!("{}ret", " ".repeat(indent))
            },
            Instruction::AllocateStack(size) => {
                println!("{}AllocateStack({})", " ".repeat(indent), size);
            },
            Instruction::DeallocateStack(size) => {
                println!("{}DeallocateStack({})", " ".repeat(indent), size);
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
            Instruction::Cmp(src1, src2 ) => {
                print!("{}cmp src1=", " ".repeat(indent));
                pretty_print_operand(&src1);
                print!(", ");
                pretty_print_operand(&src2);
                println!("");
            },
            Instruction::Cdq => {
                println!("{}cdq", " ".repeat(indent));
            },
            Instruction::Idiv(divisor) => {
                print!("{}idiv divisor=", " ".repeat(indent));
                pretty_print_operand(divisor);
                println!("");
            },
            Instruction::Jmp(label) => {
                println!("{}jmp {}", " ".repeat(indent), label);
            },
            Instruction::JmpCC(cc, label) => {
                print!("{}", " ".repeat(indent));
                pretty_print_conditional_jump(&cc);
                println!(" {}", label);
            },
            Instruction::Label(label) => {
                println!("{}:", label);
            },
            Instruction::SetCC(cc, dest) => {
                print!("{}", " ".repeat(indent));
                pretty_print_setcc(&cc);
                print!(" ");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Call(label) => {
                println!("{}call {}", " ".repeat(indent), label);
            },


          //  _ => { panic!("Unknown instruction: '{:?}'", ins); }
        };
    }
}

fn pretty_print_top_level_item(top_level_item: &TopLevel, indent: usize)
{
    match top_level_item {
        TopLevel::Function(func_name, global, instructions) => {
            println!("{}Function(", " ".repeat(indent));
            println!("{}name={func_name}", " ".repeat(indent + 4));
            println!("{}global={global}", " ".repeat(indent + 4));
            println!("{}body=(", " ".repeat(indent + 4));
            pretty_print_instructions(instructions, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{})", " ".repeat(indent));
        },
        TopLevel::StaticVariable(var_name, global, init_value) => {
            print!("{}", " ".repeat(indent));
            if *global {
                print!("GLOBAL ");
            }
            println!("Static var {var_name} = {init_value}");
        }
        //_ => { panic!("Invalid top level item: '{:?}'", f); }
    }
}


fn pretty_print_program(p: &Program, indent: usize)
{
    println!("{}Program(", " ".repeat(indent));
    match p {
        Program::ProgramDefinition(func_defs) => {
            for func_def in func_defs {
                pretty_print_top_level_item(func_def, indent + 4);
                println!("");
            }
        },
        //_ => { panic!("Invalid program definition: '{:?}'", p); }
    };
    println!("{})", " ".repeat(indent))
}


pub fn pretty_print_ast(program: &Program)
{
    pretty_print_program(&program, 0);
}
