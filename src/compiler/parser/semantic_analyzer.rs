use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::ast::*;

static GOTO_LABEL_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);
static LOCAL_TMP_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);

pub struct LocalVariableInfo {
    global_name: String,
    defined_in_current_block: bool
}

fn make_unique_global_goto_label(label: &str) -> String
{
    let index = GOTO_LABEL_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let global_label = format!("goto_lbl_{}.{}", label, index);

    global_label
}

fn make_unique_global_name(var_name: &str) -> String
{
    let index = LOCAL_TMP_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let temp_name = format!("local.var.{}.{}", var_name, index);

    temp_name
}


fn resolve_expression(expr: &Expression, var_map: &mut HashMap<String, LocalVariableInfo>) -> Result<Expression, String>
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
            let var_info = var_map.get(var_name);

            if var_info.is_none() {
                return Err(format!("Use of undeclared variable '{}'", var_name));
            }

            Ok(Expression::Var(var_info.unwrap().global_name.clone()))
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


fn resolve_statement(stmnt: &Statement, var_map: &mut HashMap<String, LocalVariableInfo>, labels: &HashMap<String, String>) -> Result<Statement, String>
{
    match stmnt {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            let resolved_unlabled_stmnt = resolve_unlabeled_statement(unlabeled_stmnt, var_map, labels)?;
            return Ok(Statement::Stmnt(None, resolved_unlabled_stmnt));
        },
        Statement::Stmnt(Some(stmnt_labels), unlabeled_stmnt) => {
            let resolved_unlabled_stmnt = resolve_unlabeled_statement(unlabeled_stmnt, var_map, labels)?;
            let resolved_stmnt_labels = stmnt_labels.iter().map(|stmnt_label| labels.get(stmnt_label).unwrap().clone()).collect();

            return Ok(Statement::Stmnt(Some(resolved_stmnt_labels), resolved_unlabled_stmnt));
        }
    };
}

fn check_statement_goto_labels(stmnt: &Statement, labels: &mut HashMap<String, String>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            check_unlabeled_statement_goto_labels(unlabeled_stmnt, labels)?;
        },
        Statement::Stmnt(Some(stmnt_labels),unlabeled_stmnt ) => {
            for stmnt_label in stmnt_labels {
                if labels.contains_key(stmnt_label) {
                    return Err(format!("Duplicate label: '{stmnt_label}'"));
                }
                let global_label = make_unique_global_goto_label(stmnt_label);
                labels.insert(stmnt_label.clone(), global_label);
            }
            check_unlabeled_statement_goto_labels(unlabeled_stmnt, labels)?;
        }
    }

    Ok(())
}

fn resolve_unlabeled_statement(stmnt: &UnlabeledStatement, var_map: &mut HashMap<String, LocalVariableInfo>, labels: &HashMap<String, String>) -> Result<UnlabeledStatement, String>
{
    match stmnt {
        UnlabeledStatement::Return(expr) => {
            let resolved_expression = resolve_expression(expr, var_map)?;
            Ok(UnlabeledStatement::Return(resolved_expression))
        },
        UnlabeledStatement::Goto(label) => {
            if !labels.contains_key(label) {
                return Err(format!("Label '{}' not found", label));
            }
            Ok(UnlabeledStatement::Goto(labels.get(label).unwrap().clone()))
        },
        UnlabeledStatement::If(cond, then_stmnt , else_stmnt) => {
            let resolved_cond = resolve_expression(cond, var_map)?;
            let resolved_then_stmnt = resolve_statement(then_stmnt, var_map, labels)?;
            let resolved_else_stmnt = if let Some(else_stmnt) = else_stmnt {
                let resolved_else_stmnt = resolve_statement(else_stmnt, var_map, labels)?;
                Some(Box::new(resolved_else_stmnt))
            }
            else {
                None
            };
            Ok(UnlabeledStatement::If(resolved_cond, Box::new(resolved_then_stmnt), resolved_else_stmnt))
        },
        UnlabeledStatement::Compound(block) => {
            let mut new_var_map = HashMap::new();
            for (local_var_name, local_var_info) in var_map {
                new_var_map.insert(
                    local_var_name.clone(),
                    LocalVariableInfo {global_name: local_var_info.global_name.clone(), defined_in_current_block: false }
                );
            }

            let resolved_block = resolve_block(block, &mut new_var_map, labels)?;
            Ok(UnlabeledStatement::Compound(resolved_block))
        },
        UnlabeledStatement::Expr(expr) => {
            let resolved_expression = resolve_expression(expr, var_map)?;
            Ok(UnlabeledStatement::Expr(resolved_expression))
        },
        UnlabeledStatement::Null => Ok(UnlabeledStatement::Null),
        _ => { panic!("Semantic Analyzer: {:?} Not implemented yet!", stmnt); }
    }
}

fn check_unlabeled_statement_goto_labels(unlabeled_stmnt: &UnlabeledStatement, labels: &mut HashMap<String, String>) -> Result<(), String>
{
    match unlabeled_stmnt {
        UnlabeledStatement::If(_,then_stmnt , None) => {
            check_statement_goto_labels(then_stmnt, labels)?;
        },
        UnlabeledStatement::If(_,then_stmnt , Some(else_stmnt)) => {
            check_statement_goto_labels(then_stmnt, labels)?;
            check_statement_goto_labels(else_stmnt, labels)?;
        },
        UnlabeledStatement::Compound(Block::Blk(block_items)) => {
            for block_item in block_items {
                check_block_item_goto_labels(block_item, labels)?;
            }
        },
        _ => {}
    }

    Ok(())
}

fn resolve_declaration(decl: &Declaration, var_map: &mut HashMap<String, LocalVariableInfo>) -> Result<Declaration, String>
{
    match decl {
        Declaration::Declarant(var_name, initializer) => {
            if let Some(local_var_info) = var_map.get(var_name) && local_var_info.defined_in_current_block {
                return Err(format!("Variable '{}' is already defined in the current scope", var_name));
            }

            let temp_name = make_unique_global_name(var_name);

            var_map.insert(
                var_name.clone(),
                LocalVariableInfo { global_name: temp_name.clone(), defined_in_current_block: true }
            );

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


fn resolve_block_item(block_item: &BlockItem, var_map: &mut HashMap<String, LocalVariableInfo>, labels: &HashMap<String, String>) -> Result<BlockItem, String>
{
    match block_item {
        BlockItem::D(decl) => {
            let resolved_decl = resolve_declaration(decl, var_map)?;
            Ok(BlockItem::D(resolved_decl))
        },
        BlockItem::S(stmnt) => {
            let resolved_stmnt = resolve_statement(stmnt, var_map, labels)?;
            Ok(BlockItem::S(resolved_stmnt))
        }
    }
}


fn check_block_item_goto_labels(block_item: &BlockItem, labels: &mut HashMap<String, String>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            Ok(())
        },
        BlockItem::S(stmnt) => {
            check_statement_goto_labels(stmnt, labels)?;
            Ok(())
        }
    }
}

fn resolve_block(block: &Block, var_map: &mut HashMap<String, LocalVariableInfo>, labels: &HashMap<String, String>) -> Result<Block, String>
{
    let mut resolved_block_items = vec![];

    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                let resolved_block_item = resolve_block_item(&block_item, var_map, labels)?;
                resolved_block_items.push(resolved_block_item);
            }
        }
    }
    Ok(Block::Blk(resolved_block_items))
}

fn check_block_goto_labels(block: &Block, labels: &mut HashMap<String, String>) -> Result<(), String>
{
    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                check_block_item_goto_labels(block_item, labels)?;
            }
        }
    };

    Ok(())
}


fn resolve_function(func_def: &FunctionDefinition, var_map: &mut HashMap<String, LocalVariableInfo>) -> Result<FunctionDefinition, String>
{
    match func_def {
        FunctionDefinition::Function(name, block ) => {
            let mut labels = HashMap::new();
            check_block_goto_labels(&block, &mut labels)?;
            let resolved_block = resolve_block(block, var_map, &labels)?;
            Ok(FunctionDefinition::Function(name.clone(), resolved_block))
        }
    }
}


pub fn resolve_program(prog: &Program) -> Result<Program, String>
{
    match prog {
        Program::ProgramDefinition(func_def) => {
            let mut var_map = HashMap::new();
            let resolved_func_def = resolve_function(func_def, &mut var_map)?;
            Ok(Program::ProgramDefinition(resolved_func_def))
        }
    }
}
