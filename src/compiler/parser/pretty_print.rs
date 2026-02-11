use super::ast::*;

fn pretty_print_expression(expr: &Expression, indent: usize)
{
    match expr {
        Expression::IntConstant(c) => {
            println!("{}Constant({})", " ".repeat(indent), c);
        },
        Expression::Unary(unary_op, inner_expression) => {
            pretty_print_unary_operator(unary_op, inner_expression, indent);
        },
        Expression::Binary(binary_op, left, right) => {
            pretty_print_binary_operator(binary_op, left, right, indent);
        }
    }
}

fn pretty_print_unary_operator(unary_op: &UnaryOperator, inner_expression: &Expression, indent: usize)
{
    match unary_op {
        UnaryOperator::Plus => {
            println!("{}UnaryPlus(", " ".repeat(indent));
        }
        UnaryOperator::Complement => {
            println!("{}BinaryNot(", " ".repeat(indent));
        },
        UnaryOperator::Negate => {
            println!("{}Minus(", " ".repeat(indent));
        }
    }

    pretty_print_expression(inner_expression, indent + 4);
    println!("{})", " ".repeat(indent));
}


fn pretty_print_binary_operator(
    binary_op: &BinaryOperator,
    left: &Expression, 
    right: &Expression, 
    indent: usize)
{
    match binary_op {
        BinaryOperator::Add => {
            println!("{}Add(", " ".repeat(indent));
        },
        BinaryOperator::Subtract => {
            println!("{}Subtract(", " ".repeat(indent));
        },
        BinaryOperator::Multiply => {
            println!("{}Multiply(", " ".repeat(indent));
        },
        BinaryOperator::Divide => {
            println!("{}Divide(", " ".repeat(indent));
        },
        BinaryOperator::Remainder => {
            println!("{}Remainder(", " ".repeat(indent));
        }
    };

    pretty_print_expression(left, indent + 4);
    println!("{},", " ".repeat(indent + 4));
    pretty_print_expression(right, indent + 4);
    println!("{})", " ".repeat(indent));

}


fn pretty_print_statement(s: &Statement, indent: usize)
{
    match s {
        Statement::Return(expr) => {
            println!("{}Return(", " ".repeat(indent));
            pretty_print_expression(&expr, indent + 4);
            println!("{})", " ".repeat(indent));
        }
    }
}

fn pretty_print_function(f: &FunctionDefinition, indent: usize)
{
    match f {
        FunctionDefinition::Function(func_name, stmt) => {
            println!("{}Function(", " ".repeat(indent));
            println!("{}name={func_name}", " ".repeat(indent + 4));
            print!("{}body=(", " ".repeat(indent + 4));
            pretty_print_statement(&stmt, indent + 4);
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


pub fn pretty_print_ast(p: &Program)
{
    pretty_print_program(p, 0);
}
