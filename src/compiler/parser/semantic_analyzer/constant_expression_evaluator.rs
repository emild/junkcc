use crate::compiler::parser::ast::*;

fn evaluate_constant_unary_operator(unop: &UnaryOperator, sub_expr: &TypedExpression) -> Result<Const, String>
{
    let sub_expr_val = evaluate_constant_expression(sub_expr)?;
    let unary_result = match unop {
        UnaryOperator::Complement => {
            sub_expr_val.complement()
        },
        UnaryOperator::LogicalNot => {
            sub_expr_val.logical_not()
        },
        UnaryOperator::Negate => {
            sub_expr_val.unary_minus()
        },
        UnaryOperator::Plus => {
            sub_expr_val.unary_plus()
        },
        UnaryOperator::PostDecrement |
        UnaryOperator::PostIncrement |
        UnaryOperator::PreDecrement |
        UnaryOperator::PreIncrement => {
            return Err(format!("Non constant expression (cannot use '{:?}')", unop));
        }

    };

    Ok(unary_result)

}


fn evaluate_constant_binary_operator(binop: &BinaryOperator, sub_expr_1: &TypedExpression, sub_expr_2: &TypedExpression) -> Result<Const, String>
{
    let arg1 = evaluate_constant_expression(sub_expr_1)?;
    let arg2 = evaluate_constant_expression(sub_expr_2)?;

    let binop_val = match binop {
        BinaryOperator::Add             =>  arg1.add(&arg2),
        BinaryOperator::BitwiseAnd      =>  arg1.bin_and(&arg2),
        BinaryOperator::BitwiseOr       =>  arg1.bin_or(&arg2),
        BinaryOperator::BitwiseXor      =>  arg1.bin_xor(&arg2),
        BinaryOperator::Divide          => {
            if arg2.is_false() {
                return Err(format!("Division by zero"))?;
            }
            arg1.div(&arg2)
        },
        BinaryOperator::Equal           => arg1.eq(&arg2),
        BinaryOperator::GreaterOrEqual  => arg1.ge(&arg2),
        BinaryOperator::GreaterThan     => arg1.gt(&arg2),
        BinaryOperator::LessOrEqual     => arg1.le(&arg2),
        BinaryOperator::LessThan        => arg1.lt(&arg2),
        BinaryOperator::LogicalAnd      => arg1.logical_and(&arg2),
        BinaryOperator::LogicalOr       => arg1.logical_or(&arg2),
        BinaryOperator::Multiply        => arg1.mul(&arg2),
        BinaryOperator::NotEqual        => arg1.ne(&arg2),
        BinaryOperator::Remainder       => {
            if arg2.is_false() {
                return Err(format!("Division by zero"))
            }
            arg1.modulo(&arg2)
        },
        BinaryOperator::ShiftLeft       => arg1.left_shift(&arg2),
        BinaryOperator::ShiftRight      => arg1.right_shift(&arg2),
        BinaryOperator::Subtract        => arg1.sub(&arg2),
        _ => {
            return Err(format!("Non constant expression '{:?}'", binop));
        }

    };

    Ok(binop_val)

}



pub fn evaluate_constant_expression(typed_expr: &TypedExpression) -> Result<Const, String>
{
    let TypedExpression::TypedExp(typ, expr) = typed_expr;
    let val = match expr {
        Expression::Assignment(_,_) |
        Expression::CompoundAssignment(_,_,_) |
        Expression::PostDecrement(_) |
        Expression::PostIncrement(_) |
        Expression::PreDecrement(_) |
        Expression::PreIncrement(_) |
        Expression::Var(_) |
        Expression::FunctionCall(_,_)=> {
            return Err(format!("Non-constant expression: '{:?}'", expr));
        },
        Expression::Unary(unop, sub_expr) => {
            evaluate_constant_unary_operator(unop, sub_expr)?
        },
        Expression::Binary(binop, sub_expr_1 , sub_expr_2) => {
            evaluate_constant_binary_operator(binop, sub_expr_1, sub_expr_2)?
        },
        Expression::Constant(Const::ConstInt(c)) => Const::ConstInt(*c),
        Expression::Constant(Const::ConstLong(c)) => Const::ConstLong(*c),
        Expression::Conditional(cond_exp, true_exp, false_exp) => {
            let cond_val = evaluate_constant_expression(cond_exp)?;
            let val = if cond_val.is_true() {
                evaluate_constant_expression(true_exp)?
            }
            else {
                evaluate_constant_expression(false_exp)?
            };

            val
        },
        Expression::Cast(typ, inner_exp) => {
            let inner_val = evaluate_constant_expression(inner_exp)?;
            inner_val.convert_to(typ)
        }
    };

    Ok(val)
}
