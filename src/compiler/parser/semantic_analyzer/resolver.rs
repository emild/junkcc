use std::collections::HashMap;

use super::super::super::parser::ast::*;

use super::break_classifier::check_and_classify_block_break_statements;
use super::goto_labels::check_block_goto_labels;
use super::loop_labeling::label_block_loops;
use super::switch_labeling::label_block_switch_statements;
use super::LocalVariableInfo;
use super::constant_expression_evaluator::evaluate_constant_expression;
use super::unique_global_labels::make_unique_global_name;


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

        Expression::FunctionCall(func_name, args) => {
            //let mut resolved_args = vec![];
            panic!("Function call not supported/implemented yet");

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


fn resolve_statement(stmnt: &Statement, var_map: &mut HashMap<String, LocalVariableInfo>, goto_labels: &HashMap<String, String>) -> Result<Statement, String>
{
    match stmnt {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            let resolved_unlabled_stmnt = resolve_unlabeled_statement(unlabeled_stmnt, var_map, goto_labels)?;
            return Ok(Statement::Stmnt(None, resolved_unlabled_stmnt));
        },
        Statement::Stmnt(Some(stmnt_labels), unlabeled_stmnt) => {
            let resolved_unlabled_stmnt = resolve_unlabeled_statement(unlabeled_stmnt, var_map, goto_labels)?;
            let mut resolved_stmnt_labels = vec![];
            for stmnt_label in stmnt_labels {
                match stmnt_label {
                    Label::Goto(stmnt_goto_label) => {
                        let resolved_stmnt_goto_label = goto_labels.get(stmnt_goto_label).unwrap();
                        resolved_stmnt_labels.push(Label::Goto(resolved_stmnt_goto_label.clone()));
                    },
                    Label::Case(expr) => {
                        let case_val = evaluate_constant_expression(expr)?;
                        resolved_stmnt_labels.push(Label::Case(Expression::IntConstant(case_val)));
                    },
                    Label::Default => {
                        resolved_stmnt_labels.push(Label::Default);
                    },
                    _ => {
                        panic!("Unexpected label: {:?}", stmnt_label);
                    }
                };
            }

            return Ok(Statement::Stmnt(Some(resolved_stmnt_labels), resolved_unlabled_stmnt));
        }
    };
}





// Copies a var map. but resets defined_in_current_block to false
fn copy_var_map(var_map: &HashMap<String, LocalVariableInfo>) -> HashMap<String, LocalVariableInfo>
{
    let mut new_var_map = HashMap::new();
    for (local_var_name, local_var_info) in var_map {
        new_var_map.insert(
            local_var_name.clone(),
            LocalVariableInfo {global_name: local_var_info.global_name.clone(), defined_in_current_block: false }
        );
    }

    new_var_map
}


fn resolve_for_init(for_init: &ForInit, var_map: &mut HashMap<String, LocalVariableInfo>) -> Result<ForInit, String>
{
    let resolved_for_init = match for_init {
        ForInit::InitExp(None) => ForInit::InitExp(None),
        ForInit::InitExp(Some(expr)) => {
            let resolved_expr = resolve_expression(expr, var_map)?;
            ForInit::InitExp(Some(resolved_expr))
        },
        ForInit::InitDecl(decl) => {
            let resolved_decl = resolve_variable_declaration(decl, var_map)?;
            ForInit::InitDecl(resolved_decl)
        }
    };

    Ok(resolved_for_init)
}


fn resolve_unlabeled_statement(stmnt: &UnlabeledStatement, var_map: &mut HashMap<String, LocalVariableInfo>, goto_labels: &HashMap<String, String>) -> Result<UnlabeledStatement, String>
{
    match stmnt {
        UnlabeledStatement::Return(expr) => {
            let resolved_expression = resolve_expression(expr, var_map)?;
            Ok(UnlabeledStatement::Return(resolved_expression))
        },
        UnlabeledStatement::Goto(label) => {
            if !goto_labels.contains_key(label) {
                return Err(format!("Label '{}' not found", label));
            }
            Ok(UnlabeledStatement::Goto(goto_labels.get(label).unwrap().clone()))
        },
        UnlabeledStatement::If(cond, then_stmnt , else_stmnt) => {
            let resolved_cond = resolve_expression(cond, var_map)?;
            let resolved_then_stmnt = resolve_statement(then_stmnt, var_map, goto_labels)?;
            let resolved_else_stmnt = if let Some(else_stmnt) = else_stmnt {
                let resolved_else_stmnt = resolve_statement(else_stmnt, var_map, goto_labels)?;
                Some(Box::new(resolved_else_stmnt))
            }
            else {
                None
            };
            Ok(UnlabeledStatement::If(resolved_cond, Box::new(resolved_then_stmnt), resolved_else_stmnt))
        },
        UnlabeledStatement::Break(break_type, loop_or_switch_label) => {
           Ok(UnlabeledStatement::Break(break_type.clone(), loop_or_switch_label.clone()))
        },
        UnlabeledStatement::Continue(loop_label) => {
            Ok(UnlabeledStatement::Continue(loop_label.clone()))
        },
        UnlabeledStatement::While(cond, body, loop_label) => {
            let resolved_cond = resolve_expression(cond, var_map)?;
            let resolved_body = resolve_statement(body, var_map, goto_labels)?;
            Ok(UnlabeledStatement::While(resolved_cond, Box::new(resolved_body), loop_label.clone()))
        },
        UnlabeledStatement::DoWhile(body, cond, loop_label) => {
            let resolved_cond = resolve_expression(cond, var_map)?;
            let resolved_body = resolve_statement(body, var_map, goto_labels)?;
            Ok(UnlabeledStatement::DoWhile(Box::new(resolved_body), resolved_cond, loop_label.clone()))
        },
        UnlabeledStatement::For(for_init, cond, post, body, loop_label) => {
            let mut new_var_map = copy_var_map(var_map);
            let resolved_for_init = resolve_for_init(for_init, &mut new_var_map)?;
            let mut resolved_cond = None;
            if let Some(expr) = cond {
                let resolved_expr = resolve_expression(expr, &mut new_var_map)?;
                resolved_cond = Some(resolved_expr);
            }
            let mut resolved_post = None;
            if let Some(expr) = post {
                let resolved_expr = resolve_expression(expr, &mut new_var_map)?;
                resolved_post = Some(resolved_expr);
            }
            let resolved_body = resolve_statement(body, &mut new_var_map, goto_labels)?;
            Ok(UnlabeledStatement::For(resolved_for_init, resolved_cond, resolved_post, Box::new(resolved_body), loop_label.clone()))
        },
        UnlabeledStatement::Switch(cond, body, switch_label, case_label_map, default_label) => {
            let resolved_cond = resolve_expression(cond, var_map)?;
            let resolved_body = resolve_statement(body, var_map, goto_labels)?;
            Ok(UnlabeledStatement::Switch(resolved_cond, Box::new(resolved_body), switch_label.clone(), case_label_map.clone(), default_label.clone()))
        }
        UnlabeledStatement::Compound(block) => {
            let mut new_var_map = copy_var_map(var_map);
            let resolved_block = resolve_block(block, &mut new_var_map, goto_labels)?;
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




fn resolve_variable_declaration(decl: &VariableDeclaration, var_map: &mut HashMap<String, LocalVariableInfo>) -> Result<VariableDeclaration, String>
{
    match decl {
        VariableDeclaration::Declarant(var_name, initializer) => {
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

            Ok(VariableDeclaration::Declarant(temp_name, resolved_initializer))
        }
    }
}


fn resolve_block_item(block_item: &BlockItem, var_map: &mut HashMap<String, LocalVariableInfo>, labels: &HashMap<String, String>) -> Result<BlockItem, String>
{
    panic!("resolve_block_item(): NO LONGER SUPPORTED/IMPLEMENTED");
    /*
    match block_item {
        BlockItem::D(decl) => {
            let resolved_decl = resolve_variable_declaration(decl, var_map)?;
            Ok(BlockItem::D(resolved_decl))
        },
        BlockItem::S(stmnt) => {
            let resolved_stmnt = resolve_statement(stmnt, var_map, labels)?;
            Ok(BlockItem::S(resolved_stmnt))
        }
    }
    */
}




fn resolve_block(block: &Block, var_map: &mut HashMap<String, LocalVariableInfo>, goto_labels: &HashMap<String, String>) -> Result<Block, String>
{
    let mut resolved_block_items = vec![];

    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                let resolved_block_item = resolve_block_item(&block_item, var_map, goto_labels)?;
                resolved_block_items.push(resolved_block_item);
            }
        }
    }
    Ok(Block::Blk(resolved_block_items))
}


fn resolve_function(func_def: &FunctionDeclaration, var_map: &mut HashMap<String, LocalVariableInfo>) -> Result<FunctionDeclaration, String>
{
    panic!("resolve_function(): No longer implemented/supported");
    /*
    match func_def {
        FunctionDeclaration::Function(name, block ) => {
            let mut goto_labels = HashMap::new();
            check_block_goto_labels(&block, &mut goto_labels)?;
            let mut resolved_block = resolve_block(block, var_map, &goto_labels)?;
            check_and_classify_block_break_statements(&mut resolved_block, &None)?;
            label_block_loops(&mut resolved_block, &None)?;
            label_block_switch_statements(&mut resolved_block, &None, &mut HashMap::new(), &mut None)?;
            Ok(FunctionDeclaration::Function(name.clone(), resolved_block))
        }
    }
    */
}


pub fn resolve_program(prog: &Program) -> Result<Program, String>
{
    panic!("resolve_program(): No longer implemented/supported");
    /*
    match prog {
        Program::ProgramDefinition(func_def) => {
            let mut var_map = HashMap::new();
            let resolved_func_def = resolve_function(func_def, &mut var_map)?;
            Ok(Program::ProgramDefinition(resolved_func_def))
        }
    }
    */
}