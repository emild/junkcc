use crate::compiler::parser::ast::*;

fn evaluate_constant_unary_operator(unop: &UnaryOperator, sub_expr: &Expression) -> Result<i32, String>
{
    let sub_expr_val = evaluate_constant_expression(sub_expr)?;
    let unary_result = match unop {
        UnaryOperator::Complement => {
            !sub_expr_val
        },
        UnaryOperator::LogicalNot => {
            if sub_expr_val != 0 {
                0
            }
            else {
                1
            }
        },
        UnaryOperator::Negate => {
            -sub_expr_val
        },
        UnaryOperator::Plus => {
            sub_expr_val
        },
        UnaryOperator::PostDecrement |
        UnaryOperator::PostIncrement |
        UnaryOperator::PreDecrement |
        UnaryOperator::PreIncrement => {
            return Err(format!("Non constant expression in case label (cannot use '{:?}'", unop));
        }

    };

    Ok(unary_result)

}


fn evaluate_constant_binary_operator(binop: &BinaryOperator, sub_expr_1: &Expression, sub_expr_2: &Expression)->Result<i32, String>
{
    let arg1 = evaluate_constant_expression(sub_expr_1)?;
    let arg2 = evaluate_constant_expression(sub_expr_2)?;

    let binop_val = match binop {
        BinaryOperator::Add         =>  arg1 + arg2 ,
        BinaryOperator::BitwiseAnd  =>  arg1 & arg2 ,
        BinaryOperator::BitwiseOr   =>  arg1 | arg2 ,
        BinaryOperator::BitwiseXor => arg1 ^ arg2,
        BinaryOperator::Divide => {
            if arg2 == 0 {
                return Err(format!("Division by zero"))?;
            }
            arg1 / arg2
        },
        BinaryOperator::Equal => {
            if arg1 == arg2 {
                1
            }
            else {
                0
            }
        },
        BinaryOperator::GreaterOrEqual => {
            if arg1 >= arg2 {
                1
            }
            else {
                0
            }
        },
        BinaryOperator::GreaterThan => {
            if arg1 > arg2 {
                1
            }
            else {
                0
            }
        },
        BinaryOperator::LessOrEqual => {
            if arg1 <= arg2 {
                1
            }
            else {
                0
            }
        },
        BinaryOperator::LessThan => {
            if arg1 < arg2 {
                1
            }
            else {
                0
            }
        },
        BinaryOperator::LogicalAnd => {
            if arg1 == 0 {
                0
            }
            else if arg2 == 0 {
                0
            }
            else {
                1
            }
        },
        BinaryOperator::LogicalOr => {
            if arg1 != 0 {
                1
            }
            else if arg2 != 0 {
                1
            }
            else {
                0
            }
        },
        BinaryOperator::Multiply => {
            arg1 * arg2
        },
        BinaryOperator::NotEqual => {
            if arg1 != arg2 {
                1
            }
            else {
                0
            }
        },
        BinaryOperator::Remainder => {
            if arg2 == 0 {
                return Err(format!("Division by zero"))
            }
            arg1 % arg2
        },
        BinaryOperator::ShiftLeft => arg1 << arg2,
        BinaryOperator::ShiftRight => arg1 >> arg2,
        BinaryOperator::Subtract => arg1 - arg2,
        _ => {
            return Err(format!("Non constant expression after case: '{:?}'", binop));
        }

    };

    Ok(binop_val)

}


pub fn evaluate_constant_expression(expr: &Expression) -> Result<i32, String>
{
    let val = match expr {
        Expression::Assignment(_,_) |
        Expression::CompoundAssignment(_,_,_) |
        Expression::PostDecrement(_) |
        Expression::PostIncrement(_) |
        Expression::PreDecrement(_) |
        Expression::PreIncrement(_) |
        Expression::Var(_) |
        Expression::FunctionCall(_,_)=> {
            return Err(format!("Non-constant expression in case label"))
        },
        Expression::Unary(unop, sub_expr) => {
            evaluate_constant_unary_operator(unop, sub_expr)?
        },
        Expression::Binary(binop, sub_expr_1 , sub_expr_2) => {
            evaluate_constant_binary_operator(binop, sub_expr_1, sub_expr_2)?
        }
        Expression::IntConstant(c) => *c,
        Expression::Conditional(cond_exp, true_exp, false_exp) => {
            let cond_val = evaluate_constant_expression(cond_exp)?;
            let val = if cond_val != 0 {
                evaluate_constant_expression(true_exp)?
            }
            else {
                evaluate_constant_expression(false_exp)?
            };

            val
        }
    };

    Ok(val)
}
