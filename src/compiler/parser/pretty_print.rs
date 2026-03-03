use super::ast::*;

fn pretty_print_expression(expr: &Expression, indent: usize)
{
    match expr {
        Expression::IntConstant(c) => {
            println!("{}Constant({})", " ".repeat(indent), c);
        },
        Expression::Var(var_name) => {
            println!("{}Var({})", " ".repeat(indent), var_name);
        },
        Expression::Unary(unary_op, inner_expression) => {
            pretty_print_unary_operator(unary_op, inner_expression, indent);
        },
        Expression::PreIncrement(inner_expression) => {
            pretty_print_unary_operator(&UnaryOperator::PreIncrement, inner_expression, indent)
        },
        Expression::PreDecrement(inner_expression) => {
            pretty_print_unary_operator(&UnaryOperator::PreDecrement, inner_expression, indent)
        },
        Expression::PostIncrement(inner_expression) => {
            pretty_print_unary_operator(&UnaryOperator::PostIncrement, inner_expression, indent)
        },
        Expression::PostDecrement(inner_expression) => {
            pretty_print_unary_operator(&UnaryOperator::PostDecrement, inner_expression, indent)
        },
        Expression::Assignment(left, right) => {
            pretty_print_assignment(left, right, indent);
        },
        Expression::Binary(binary_op, left, right) => {
            pretty_print_binary_operator(binary_op, left, right, indent);
        },
        Expression::CompoundAssignment(binary_op, left, right) => {
            pretty_print_compound_assignment(binary_op, left, right, indent);
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
        },
        UnaryOperator::LogicalNot => {
            println!("{}LogicalNot(", " ".repeat(indent));
        },
        UnaryOperator::PreIncrement => {
            println!("{}PreIncrement(", " ".repeat(indent));
        },
        UnaryOperator::PreDecrement => {
            println!("{}PreDecrement(", " ".repeat(indent));
        },
        UnaryOperator::PostIncrement => {
            println!("{}PostIncrement(", " ".repeat(indent));
        },
        UnaryOperator::PostDecrement => {
            println!("{}PostDecrement(", " ".repeat(indent));
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
        },
        BinaryOperator::BitwiseAnd => {
            println!("{}BitwiseAnd(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseOr => {
            println!("{}BitwiseOr(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseXor => {
            println!("{}BitwiseXor(", " ".repeat(indent));
        },
        BinaryOperator::ShiftLeft => {
            println!("{}ShiftLeft(", " ".repeat(indent));
        },
        BinaryOperator::ShiftRight => {
            println!("{}ShiftRight(", " ".repeat(indent));
        },
        BinaryOperator::LogicalOr => {
            println!("{}LogicalOr(", " ".repeat(indent));
        },
        BinaryOperator::LogicalAnd => {
            println!("{}LogicalAnd(", " ".repeat(indent));
        },
        BinaryOperator::Equal => {
            println!("{}Equal(", " ".repeat(indent));
        },
        BinaryOperator::NotEqual => {
            println!("{}NotEqual(", " ".repeat(indent));
        },
        BinaryOperator::LessThan => {
            println!("{}LessThan(", " ".repeat(indent));
        },
        BinaryOperator::LessOrEqual => {
            println!("{}LessOrEqual(", " ".repeat(indent));
        },
        BinaryOperator::GreaterThan => {
            println!("{}GreaterThan(", " ".repeat(indent));
        },
        BinaryOperator::GreaterOrEqual => {
            println!("{}GreaterOrEqual(", " ".repeat(indent));
        },
        BinaryOperator::Assign => {
            println!("{}Assign(", " ".repeat(indent));
        },
        BinaryOperator::AddAssign => {
            println!("{}AddAssign(", " ".repeat(indent));
        },
        BinaryOperator::SubtractAssign => {
            println!("{}SubtractAssign(", " ".repeat(indent));
        },
        BinaryOperator::MultiplyAssign => {
            println!("{}MultiplyAssign(", " ".repeat(indent));
        },
        BinaryOperator::DivideAssign => {
            println!("{}DivideAssign(", " ".repeat(indent));
        },
        BinaryOperator::RemainderAssign => {
            println!("{}RemainderAssign(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseAndAssign => {
            println!("{}BitwiseAndAssign(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseOrAssign => {
            println!("{}BitwiseOrAssign(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseXorAssign => {
            println!("{}BitwiseXorAssign(", " ".repeat(indent));
        },
        BinaryOperator::ShiftLeftAssign => {
            println!("{}ShiftLeftAssign(", " ".repeat(indent));
        },
        BinaryOperator::ShiftRightAssign => {
            println!("{}ShiftRightAssign(", " ".repeat(indent));
        }
    };

    pretty_print_expression(left, indent + 4);
    println!("{},", " ".repeat(indent + 4));
    pretty_print_expression(right, indent + 4);
    println!("{})", " ".repeat(indent));

}

fn pretty_print_assignment(left: &Expression, right: &Expression, indent: usize)
{
    pretty_print_binary_operator(&BinaryOperator::Assign, left, right, indent);
}

fn pretty_print_compound_assignment(binary_op: &BinaryOperator, left: &Expression, right: &Expression, indent: usize)
{
    pretty_print_binary_operator(binary_op, left, right, indent);
}

fn pretty_print_statement(s: &Statement, indent: usize)
{
    match s {
        Statement::Return(expr) => {
            println!("{}Return(", " ".repeat(indent));
            pretty_print_expression(&expr, indent + 4);
            println!("{})", " ".repeat(indent));
        },
        Statement::Expr(expr) => {
            println!("{}Expr(", " ".repeat(indent));
            pretty_print_expression(expr, indent + 4);
            println!("{})", " ".repeat(indent));
        },
        Statement::Null => {
            println!("{}NOP()", " ".repeat(indent));
        }
    }
}


fn pretty_print_declaration(decl: &Declaration, indent: usize)
{
    match decl {
        Declaration::Declarant(var_name, Some(expr_init) ) => {
            println!("{}Var {} = (", " ".repeat(indent),  var_name);
            pretty_print_expression(expr_init, indent + 4);
            println!("{})", " ".repeat(indent));
        },
        Declaration::Declarant(var_name,None ) => {
            println!("{}var {}", " ".repeat(indent),  var_name);
        }
    }
}


fn pretty_print_block_item(block_item: &BlockItem, indent: usize)
{
    match block_item {
        BlockItem::D(decl) => {
            pretty_print_declaration(decl, indent);
        },
        BlockItem::S(stmnt) => {
            pretty_print_statement(stmnt, indent);
        }
    }
}


fn pretty_print_function(f: &FunctionDefinition, indent: usize)
{
    match f {
        FunctionDefinition::Function(func_name, block) => {
            println!("{}Function(", " ".repeat(indent));
            println!("{}name={func_name}", " ".repeat(indent + 4));
            println!("{}body=(", " ".repeat(indent + 4));
            for block_item in block {
                pretty_print_block_item(block_item, indent + 8);
            }
            println!("{})", " ".repeat(indent+4));
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
