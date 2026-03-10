use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::ast::*;

static LOCAL_TMP_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);

fn make_unique_global_name(var_name: &str) -> String
{
    let index = LOCAL_TMP_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let temp_name = format!("local.var.{}.{}", var_name, index);

    temp_name
}


fn resolve_expression(expr: &Expression, var_map: &mut HashMap<String, String>) -> Result<Expression, String>
{
    match expr {
        Expression::Assignment(left, right ) => {
            match **left {
                Expression::Var(_) => {
                    let resolved_left = resolve_expression(&left, var_map)?;
                    let resolved_right = resolve_expression(right, var_map)?;

                    Ok(Expression::Assignment(Box::new(resolved_left), Box::new(resolved_right)))
                },
                _ => {
                    return Err(format!("Non lval on the left side of '='"));
                }
            }
        },

        Expression::CompoundAssignment(binop,left, right) => {
            match **left {
                Expression::Var(_) => {
                    let resolved_left = resolve_expression(&left, var_map)?;
                    let resolved_right = resolve_expression(right, var_map)?;

                    Ok(Expression::CompoundAssignment(binop.clone(), Box::new(resolved_left), Box::new(resolved_right)))
                },
                _ => {
                    return Err(format!("Non lval on the left side of compound assignment"));
                }
            }
        },
        Expression::PreIncrement(expr)  => {
            match **expr {
                Expression::Var(_) => {
                    let resolved_expr = resolve_expression(expr, var_map)?;
                    Ok(Expression::PreIncrement(Box::new(resolved_expr)))
                },
                _ => {
                    return Err(format!("Non lval in pre-increment"));
                }
            }
        },

        Expression::PreDecrement(expr) => {
            match **expr {
                Expression::Var(_) => {
                    let resolved_expr = resolve_expression(expr, var_map)?;
                    Ok(Expression::PreDecrement(Box::new(resolved_expr)))
                },
                _ => {
                    return Err(format!("Non lval in pre-decrement"));
                }
            }
        },

        Expression::PostIncrement(expr) => {
            match **expr {
                Expression::Var(_) => {
                    let resolved_expr = resolve_expression(expr, var_map)?;
                    Ok(Expression::PostIncrement(Box::new(resolved_expr)))
                },
                _ => {
                    return Err(format!("Non lval in post-increment"));
                }
            }
        },

        Expression::PostDecrement(expr) => {
            match **expr {
                Expression::Var(_) => {
                    let resolved_expr = resolve_expression(expr, var_map)?;
                    Ok(Expression::PostDecrement(Box::new(resolved_expr)))
                },
                _ => {
                    return Err(format!("Non lval in post-decrement"));
                }
            }
        },

        Expression::Var(var_name) => {
            let resolved_var_name = var_map.get(var_name);

            if resolved_var_name.is_none() {
                return Err(format!("Use of undeclared variable '{}'", var_name));
            }

            Ok(Expression::Var(resolved_var_name.unwrap().clone()))
        },

        Expression::Conditional(cond, true_exp, false_exp) => {
            let resolved_cond = resolve_expression(cond, var_map)?;
            let resolved_true_exp = resolve_expression(true_exp, var_map)?;
            let resolved_false_exp = resolve_expression(false_exp, var_map)?;

            Ok(Expression::Conditional(Box::new(resolved_cond), Box::new(resolved_true_exp), Box::new(resolved_false_exp)))
        },

        Expression::Binary(binary_op,left, right , ) => {
            let resolved_left = resolve_expression(left, var_map)?;
            let resolved_right = resolve_expression(right, var_map)?;

            Ok(Expression::Binary(binary_op.clone(), Box::new(resolved_left), Box::new(resolved_right)))
        },

        Expression::Unary(unary_op, expr) => {
            let resolved_expr = resolve_expression(expr, var_map)?;

            Ok(Expression::Unary(unary_op.clone(), Box::new(resolved_expr)))
        },

        Expression::IntConstant(c) => {
            Ok(Expression::IntConstant(*c))
        }

    }
}


fn resolve_statement(stmnt: &Statement, var_map: &mut HashMap<String, String>) -> Result<Statement, String>
{
    match stmnt {
        Statement::Return(expr) => {
            let resolved_expression = resolve_expression(expr, var_map)?;
            Ok(Statement::Return(resolved_expression))
        },
        Statement::If(cond, then_stmnt , else_stmnt) => {
            let resolved_cond = resolve_expression(cond, var_map)?;
            let resolved_then_stmnt = resolve_statement(then_stmnt, var_map)?;
            let resolved_else_stmnt = if let Some(else_stmnt) = else_stmnt {
                let resolved_else_stmnt = resolve_statement(else_stmnt, var_map)?;
                Some(Box::new(resolved_else_stmnt))
            }
            else {
                None
            };
            Ok(Statement::If(resolved_cond, Box::new(resolved_then_stmnt), resolved_else_stmnt))
        },
        Statement::Expr(expr) => {
            let resolved_expression = resolve_expression(expr, var_map)?;
            Ok(Statement::Expr(resolved_expression))
        },
        Statement::Null => Ok(Statement::Null)
    }
}

fn resolve_declaration(decl: &Declaration, var_map: &mut HashMap<String, String>) -> Result<Declaration, String>
{
    match decl {
        Declaration::Declarant(var_name, initializer) => {
            if var_map.contains_key(var_name) {
                return Err(format!("Variable '{}' is already defined", var_name));
            }

            let temp_name = make_unique_global_name(var_name);

            var_map.insert(var_name.clone(), temp_name.clone());

            let resolved_initializer = match initializer {
                Some(init_expression) => {
                    let resolved_init_expression = resolve_expression(init_expression, var_map)?;
                    Some(resolved_init_expression)
                },
                None => None
            };

            Ok(Declaration::Declarant(temp_name, resolved_initializer))
        }
    }
}


fn resolve_block_item(block_item: &BlockItem, var_map: &mut HashMap<String, String>) -> Result<BlockItem, String>
{
    match block_item {
        BlockItem::D(decl) => {
            let resolved_decl = resolve_declaration(decl, var_map)?;
            Ok(BlockItem::D(resolved_decl))
        },
        BlockItem::S(stmnt) => {
            let resolved_stmnt = resolve_statement(stmnt, var_map)?;
            Ok(BlockItem::S(resolved_stmnt))
        }
    }
}


fn resolve_block(block: &Vec<BlockItem>, var_map: &mut HashMap<String, String>) -> Result<Vec<BlockItem>, String>
{
    let mut resolved_block = vec![];

    for block_item in block {
        let resolved_block_item = resolve_block_item(&block_item, var_map)?;
        resolved_block.push(resolved_block_item);
    }

    Ok(resolved_block)
}


fn resolve_function(func_def: &FunctionDefinition, var_map: &mut HashMap<String, String>) -> Result<FunctionDefinition, String>
{
    match func_def {
        FunctionDefinition::Function(name, block ) => {
            let resolved_block = resolve_block(block, var_map)?;
            Ok(FunctionDefinition::Function(name.clone(), resolved_block))
        }
    }
}


pub fn resolve_program(prog: &Program, var_map: &mut HashMap<String, String>) -> Result<Program, String>
{
    match prog {
        Program::ProgramDefinition(func_def) => {
            let resolved_func_def = resolve_function(func_def, var_map)?;
            Ok(Program::ProgramDefinition(resolved_func_def))
        }
    }
}