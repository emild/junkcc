use super::ast::*;


fn assembly_type_suffix(ass_type: &AssemblyType) -> &str
{
    match ass_type {
        AssemblyType::LongWord => "l",
        AssemblyType::QuadWord => "q",
        AssemblyType::Double   => "sd",
    }
}


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
        BinaryOperator::Add         => { print!("add"); },
        BinaryOperator::Sub         => { print!("sub"); },
        BinaryOperator::Mul         => { print!("mul"); },
        BinaryOperator::And         => { print!("and"); },
        BinaryOperator::Or          => { print!("or");  },
        BinaryOperator::Xor         => { print!("xor"); },
        BinaryOperator::DivDouble   => { print!("divsd"); },
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
        CC::GE => print!("jge"),

        CC::B  => print!("jb"),
        CC::BE => print!("jbe"),
        CC::A  => print!("ja"),
        CC::AE => print!("jae"),

        CC::PO => print!("jpo"),
        CC::PE => print!("jpe"),
        CC::P  => print!("jp")
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
        CC::GE => print!("setge"),

        CC::B  => print!("setb"),
        CC::BE => print!("setbe"),
        CC::A  => print!("seta"),
        CC::AE => print!("setae"),

        CC::PO => print!("setpo"),
        CC::PE => print!("setpe"),
        CC::P  => print!("setp")
    }
}

fn pretty_print_instructions(instructions: &Vec<Instruction>, indent: usize)
{
    for ins in instructions {
        match ins {
            Instruction::Mov(ass_type, src, dest) => {
                print!("{}mov typ={}, src=", " ".repeat(indent), assembly_type_suffix(ass_type));
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
            Instruction::Unary(unary_op, ass_type, dest) => {
                print!("{}", " ".repeat(indent));
                pretty_print_unary_operator(&unary_op);
                print!(" typ={}", assembly_type_suffix(&ass_type));
                print!(" dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Binary(binary_op, ass_type, src, dest) => {
                print!("{}", " ".repeat(indent));
                pretty_print_binary_operator(&binary_op);
                print!(" typ={}", assembly_type_suffix(&ass_type));
                print!(" src=");
                pretty_print_operand(&src);
                print!(" dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Cmp(ass_type, src, dst ) => {
                print!("{}cmp typ={}, src=", " ".repeat(indent), assembly_type_suffix(ass_type));
                pretty_print_operand(&src);
                print!(", dst=");
                pretty_print_operand(&dst);
                println!("");
            },
            Instruction::Cdq(ass_type) => {
                println!("{}cdq typ={}", " ".repeat(indent), assembly_type_suffix(ass_type));
            },
            Instruction::Shl(ass_type, shift_count, dest) => {
                print!("{}shl typ={}, count=", " ".repeat(indent), assembly_type_suffix(ass_type));
                pretty_print_operand(&shift_count);
                print!(", dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Shrl(ass_type, shift_count, dest) => {
                print!("{}srl typ={}, count=", " ".repeat(indent), assembly_type_suffix(ass_type));
                pretty_print_operand(&shift_count);
                print!(", dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Shra(ass_type, shift_count, dest) => {
                print!("{}sra typ={}, count=", " ".repeat(indent), assembly_type_suffix(ass_type));
                pretty_print_operand(&shift_count);
                print!(", dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::Idiv(ass_type, divisor) => {
                print!("{}idiv typ={}, divisor=", " ".repeat(indent), assembly_type_suffix(ass_type));
                pretty_print_operand(divisor);
                println!("");
            },
            Instruction::Div(ass_type, divisor) => {
                print!("{}div typ={}, divisor=", " ".repeat(indent), assembly_type_suffix(ass_type));
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
            Instruction::Movsx(src, dest) => {
                print!("{}movsx src=", " ".repeat(indent));
                pretty_print_operand(&src);
                print!(", dest=");
                pretty_print_operand(&dest);
                println!("");
            },
            Instruction::MovZeroExtend(src, dst) => {
                print!("{}MovZeroExtend src=", " ".repeat(indent));
                pretty_print_operand(&src);
                print!(", dest=");
                pretty_print_operand(&dst);
                println!("");
            },
            Instruction::Cvttsd2si(ass_type, src, dst) => {
                print!("{}Cvttsd2si typ={}, src=", " ".repeat(indent), assembly_type_suffix(ass_type));
                pretty_print_operand(&src);
                print!(", dest=");
                pretty_print_operand(&dst);
                println!("");
            },
            Instruction::Cvtsi2sd(ass_type, src, dst) => {
                print!("{}Cvtsi2sd typ={}, src=", " ".repeat(indent), assembly_type_suffix(ass_type));
                pretty_print_operand(&src);
                print!(", dest=");
                pretty_print_operand(&dst);
                println!("");
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
        TopLevel::StaticVariable(var_name, global, _align, init_value) => {
            print!("{}", " ".repeat(indent));
            if *global {
                print!("GLOBAL ");
            }
            println!("Static var {var_name} = {}", init_value.to_string());
        },
        TopLevel::StaticConstant(const_name, _align, init_value) => {
            print!("{}", " ".repeat(indent));
            println!("Static Const {const_name} = {}", init_value.to_string());
        },

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
