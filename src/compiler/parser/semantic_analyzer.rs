use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::ast::*;

static GOTO_LABEL_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);
static LOOP_LABEL_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);
static SWITCH_LABEL_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);
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

#[derive(PartialEq, Eq, Hash)]
enum LoopType {
    While,
    DoWhile,
    For
}

fn make_unique_global_loop_label(loop_type: &LoopType) -> String
{
    let loop_type_str = HashMap::from([
        (LoopType::While, "while"),
        (LoopType::DoWhile, "do_while"),
        (LoopType::For, "for")
    ]);

    let index = LOOP_LABEL_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let global_label = format!("{}_loop.{}", loop_type_str.get(loop_type).unwrap(), index);

    global_label
}


fn make_unique_global_switch_label() -> String
{
    let index = SWITCH_LABEL_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let global_label = format!("switch_lbl_.{}", index);

    global_label
}


fn make_unique_case_label(switch_label: &String, case_value: i32) -> String
{
    let global_label = format!("{}_{:08X}", switch_label, case_value);

    global_label
}

fn make_global_default_label(switch_label: &String) -> String
{
    let global_label = format!("{}_default", switch_label);

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
                    }
                };
            }

            return Ok(Statement::Stmnt(Some(resolved_stmnt_labels), resolved_unlabled_stmnt));
        }
    };
}

fn check_statement_goto_labels(stmnt: &Statement, goto_labels: &mut HashMap<String, String>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            check_unlabeled_statement_goto_labels(unlabeled_stmnt, goto_labels)?;
        },
        Statement::Stmnt(Some(stmnt_labels),unlabeled_stmnt ) => {
            for stmnt_label in stmnt_labels {
                if let Label::Goto(stmnt_goto_label) = stmnt_label {
                    if goto_labels.contains_key(stmnt_goto_label) {
                        return Err(format!("Duplicate label: '{stmnt_goto_label}'"));
                    }
                    let global_label = make_unique_global_goto_label(stmnt_goto_label);
                    goto_labels.insert(stmnt_goto_label.clone(), global_label);
                }
            }
            check_unlabeled_statement_goto_labels(unlabeled_stmnt, goto_labels)?;
        }
    }

    Ok(())
}


fn label_statement_loops(stmnt: &mut Statement, loop_label: &Option<String>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(_, unlabeled_stmnt) => {
            label_unlabled_statement_loops(unlabeled_stmnt, loop_label)?;
        }
    };

    Ok(())
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
            let resolved_decl = resolve_declaration(decl, var_map)?;
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
        UnlabeledStatement::Switch(cond, body, switch_label, case_and_default_labels, case_label_map, default_label) => {
            let resolved_cond = resolve_expression(cond, var_map)?;
            let resolved_body = resolve_statement(body, var_map, goto_labels)?;
            Ok(UnlabeledStatement::Switch(resolved_cond, Box::new(resolved_body), switch_label.clone(), case_and_default_labels.clone(), case_label_map.clone(), default_label.clone()))
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



fn check_unlabeled_statement_goto_labels(unlabeled_stmnt: &UnlabeledStatement, goto_labels: &mut HashMap<String, String>) -> Result<(), String>
{
    match unlabeled_stmnt {
        UnlabeledStatement::If(_,then_stmnt , None) => {
            check_statement_goto_labels(then_stmnt, goto_labels)?;
        },
        UnlabeledStatement::If(_,then_stmnt , Some(else_stmnt)) => {
            check_statement_goto_labels(then_stmnt, goto_labels)?;
            check_statement_goto_labels(else_stmnt, goto_labels)?;
        },
        UnlabeledStatement::Compound(Block::Blk(block_items)) => {
            for block_item in block_items {
                check_block_item_goto_labels(block_item, goto_labels)?;
            }
        },
        UnlabeledStatement::DoWhile(body, _, _)    |
        UnlabeledStatement::While(_ , body, _ )    |
        UnlabeledStatement::For(_, _, _, body, _)  |
        UnlabeledStatement::Switch(_ , body ,_ ,_ ,_ ,_ ) => {
            check_statement_goto_labels(body, goto_labels)?;
        }

        _ => {}
    }

    Ok(())
}


fn label_unlabled_statement_loops(unlabeled_stmnt: &mut UnlabeledStatement, loop_label: &Option<String>) -> Result<(), String>
{
    match unlabeled_stmnt {
        UnlabeledStatement::Break(Some(BreakType::Loop), brk_loop_label) => {
            /*if loop_label.is_none() {
                return Err(format!("break  outside of loop"));
            }*/
            assert!(loop_label.is_some());

            brk_loop_label.replace(loop_label.clone().unwrap());
        },
        UnlabeledStatement::Continue(cont_loop_label) => {
            if loop_label.is_none() {
                return Err(format!("continue  outside of loop"));
            }

            cont_loop_label.replace(loop_label.clone().unwrap());
        },
        UnlabeledStatement::If(_, then_stmnt, None) => {
            label_statement_loops(then_stmnt, loop_label)?;
        },
        UnlabeledStatement::If(_, then_stmnt, Some(else_stmnt)) => {
            label_statement_loops(then_stmnt, loop_label)?;
            label_statement_loops(else_stmnt, loop_label)?;
        },
        UnlabeledStatement::While(_,body ,curr_while_loop_label_) => {
            let while_loop_label = make_unique_global_loop_label(&LoopType::While);
            curr_while_loop_label_.replace(while_loop_label.clone());
            label_statement_loops(body, &Some(while_loop_label))?;
        },
        UnlabeledStatement::DoWhile(body , _ , curr_do_while_loop_label) => {

            let do_while_loop_label = make_unique_global_loop_label(&LoopType::DoWhile);
            curr_do_while_loop_label.replace(do_while_loop_label.clone());
            label_statement_loops(body, &Some(do_while_loop_label))?;
        },
        UnlabeledStatement::For(_, _, _, body, curr_for_loop_label) => {
            let for_loop_label = make_unique_global_loop_label(&LoopType::For);
            curr_for_loop_label.replace(for_loop_label.clone());
            label_statement_loops(body, &Some(for_loop_label))?;
        },
        UnlabeledStatement::Switch(_ ,body ,_ ,_ ,_ ,_ ) => {
            label_statement_loops(body, loop_label)?;
        },
        UnlabeledStatement::Compound(block) => {
            label_block_loops(block, loop_label)?;
        },
        _ => {}
    };

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


fn check_block_item_goto_labels(block_item: &BlockItem, goto_labels: &mut HashMap<String, String>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            Ok(())
        },
        BlockItem::S(stmnt) => {
            check_statement_goto_labels(stmnt, goto_labels)?;
            Ok(())
        }
    }
}

fn label_block_item_loops(block_item: &mut BlockItem, loop_label: &Option<String>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            Ok(())
        },
        BlockItem::S(stmnt) => {
            label_statement_loops(stmnt, loop_label)?;
            Ok(())
        }
    }
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

fn check_block_goto_labels(block: &Block, goto_labels: &mut HashMap<String, String>) -> Result<(), String>
{
    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                check_block_item_goto_labels(block_item, goto_labels)?;
            }
        }
    };

    Ok(())
}

fn label_block_loops(block: &mut Block, loop_label: &Option<String>) -> Result<(), String>
{
     match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                label_block_item_loops(block_item, loop_label)?;
            }
        }
    };

    Ok(())
}


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


fn evaluate_constant_expression(expr: &Expression) -> Result<i32, String>
{
    let val = match expr {
        Expression::Assignment(_,_) |
        Expression::CompoundAssignment(_,_,_) |
        Expression::PostDecrement(_) |
        Expression::PostIncrement(_) |
        Expression::PreDecrement(_) |
        Expression::PreIncrement(_) |
        Expression::Var(_) => {
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


fn switch_end_label(switch_label: &Option<String>) -> String
{
    return format!("{}_exit", switch_label.clone().unwrap());
}




fn label_unlabeled_statement_switch_statements(unlabeled_stmnt: &mut UnlabeledStatement, switch_label: &Option<String>, case_and_default_labels: &mut Vec<Label>, case_labels_map: &mut HashMap<i32, String>, default_label: &mut Option<String>) -> Result<(), String>
{
    match unlabeled_stmnt {
        UnlabeledStatement::Break(break_type, break_label) => {
            match break_type {
                Some(BreakType::Switch) => {
                    assert!(break_label.is_none());
                    if switch_label.is_none() {
                        return Err(format!("break statement outside loop or switch"));
                    }

                    break_label.replace(switch_end_label(switch_label));
                },
                _ => {}
            }
        },
        UnlabeledStatement::Compound(block) => {
            label_block_switch_statements(block, switch_label, case_and_default_labels, case_labels_map, default_label)?;
        },
        UnlabeledStatement::Switch(_, body, switch_stmnt_label, switch_stmnt_case_and_default_labels, switch_stmnt_case_label_map, switch_stmnt_default_label) => {
            let switch_stmnt_global_label = make_unique_global_switch_label();
            switch_stmnt_label.replace(switch_stmnt_global_label.clone());
            label_statement_switch_statements(body, &Some(switch_stmnt_global_label), switch_stmnt_case_and_default_labels, switch_stmnt_case_label_map, switch_stmnt_default_label)?;
        },
        UnlabeledStatement::DoWhile(body, _, _) |
        UnlabeledStatement::While(_, body, _) |
        UnlabeledStatement::For(_, _, _, body, _) => {
            label_statement_switch_statements(body, switch_label, case_and_default_labels, case_labels_map, default_label)?;
        },
        UnlabeledStatement::If(_, then_stmnt, else_stmnt) => {
            label_statement_switch_statements(then_stmnt, switch_label, case_and_default_labels, case_labels_map, default_label)?;
            if let Some(else_stmnt) = else_stmnt {
                label_statement_switch_statements(else_stmnt, switch_label, case_and_default_labels, case_labels_map, default_label)?;
            }
        },
        UnlabeledStatement::Expr(_) |
        UnlabeledStatement::Return(_) |
        UnlabeledStatement::Continue(_) |
        UnlabeledStatement::Goto(_) |
        UnlabeledStatement::Null => {}
    }

    Ok(())
}


fn label_statement_switch_statements(stmnt: &mut Statement, switch_label: &Option<String>, case_and_default_labels: &mut Vec<Label>,  case_labels_map: &mut HashMap<i32, String>, default_label: &mut Option<String>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            label_unlabeled_statement_switch_statements(unlabeled_stmnt, switch_label, case_and_default_labels, case_labels_map, default_label)?;
        },
        Statement::Stmnt(Some(labels), unlabeled_stmnt) => {
            for label in labels {
                match label {
                    Label::Case(expr) => {
                        if switch_label.is_none() {
                            return Err(format!("Case label outside switch statement"));
                        }
                        let case_value = evaluate_constant_expression(expr)?;

                        if case_labels_map.contains_key(&case_value) {
                            return Err(format!("Error: Duplicate case value"));
                        }

                        let global_case_label = make_unique_case_label(&switch_label.clone().unwrap(), case_value);
                        case_labels_map.insert(case_value, global_case_label);
                        case_and_default_labels.push(Label::Case(Expression::IntConstant(case_value)));

                    },
                    Label::Default => {
                        if switch_label.is_none() {
                            return Err(format!("default: label outside switch statement"));
                        }
                        if default_label.is_some() {
                            return Err(format!("Error: Duplicate default: label"));
                        }
                        let global_default_label = make_global_default_label(&switch_label.clone().unwrap());
                        default_label.replace(global_default_label);
                        case_and_default_labels.push(Label::Default);
                    },
                    Label::Goto(_) => {}
                }
            }

            label_unlabeled_statement_switch_statements(unlabeled_stmnt, switch_label, case_and_default_labels, case_labels_map, default_label)?;
        }
    };

    Ok(())
}


fn label_block_item_switch_statements(block_item: &mut BlockItem, switch_label: &Option<String>, case_and_default_labels: &mut Vec<Label>, case_labels_map: &mut HashMap<i32, String>, default_label: &mut Option<String>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            Ok(())
        },
        BlockItem::S(stmnt) => {
            label_statement_switch_statements(stmnt, switch_label, case_and_default_labels, case_labels_map, default_label)?;
            Ok(())
        }
    }
}


fn label_block_switch_statements(block: &mut Block, switch_label: &Option<String>, case_and_default_labels: &mut Vec<Label>, case_labels_map: &mut HashMap<i32, String>, default_label: &mut Option<String>) -> Result<(), String>
{
     match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                label_block_item_switch_statements(block_item, switch_label, case_and_default_labels, case_labels_map, default_label)?;
            }
        }
    };

    Ok(())
}


fn check_and_classify_unlabeled_statement_break_statements(
    unlabeled_stmnt: &mut UnlabeledStatement,
    break_type: &Option<BreakType>) -> Result<(), String>
{
    match unlabeled_stmnt {
        UnlabeledStatement::Break(brk_stmnt_type, _) => {
            if break_type.is_none() {
                return Err(format!("Error: break cannot be outside loop or switch statements"));
            }
            brk_stmnt_type.replace(break_type.clone().unwrap());
        },
        UnlabeledStatement::Compound(block) => {
            check_and_classify_block_break_statements(block, break_type)?;
        },
        UnlabeledStatement::DoWhile(body,_,_)   |
        UnlabeledStatement::While(_, body,_)    |
        UnlabeledStatement::For(_,_,_,body,_) => {
            check_and_classify_statement_break_statements(body, &Some(BreakType::Loop))?;
        },
        UnlabeledStatement::Switch(_, body,_,_,_,_) => {
            check_and_classify_statement_break_statements(body, &Some(BreakType::Switch))?;
        },
        UnlabeledStatement::If(_,then_stmnt , else_stmnt) => {
            check_and_classify_statement_break_statements(then_stmnt, break_type)?;
            if let Some(else_stmnt) = else_stmnt {
                check_and_classify_statement_break_statements(else_stmnt, break_type)?;
            }
        },
        _ => {}

    }

    Ok(())
}


fn check_and_classify_statement_break_statements(
    stmnt: &mut Statement,
    break_type: &Option<BreakType>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(_, unlabeled_stmnt)
            => check_and_classify_unlabeled_statement_break_statements(
                unlabeled_stmnt,
                break_type)
    }
}


fn check_and_classify_block_item_break_statements(
    block_item: &mut BlockItem,
    break_type: &Option<BreakType>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(_)
            => Ok(()),
        BlockItem::S(stmnt)
            => check_and_classify_statement_break_statements(stmnt, break_type)
    }
}

fn check_and_classify_block_break_statements(
    block: &mut Block,
    break_type: &Option<BreakType>) -> Result<(), String>
{
    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                check_and_classify_block_item_break_statements(block_item, break_type)?;
            }
        }
    }

    Ok(())
}



fn resolve_function(func_def: &FunctionDefinition, var_map: &mut HashMap<String, LocalVariableInfo>) -> Result<FunctionDefinition, String>
{
    match func_def {
        FunctionDefinition::Function(name, block ) => {
            let mut goto_labels = HashMap::new();
            check_block_goto_labels(&block, &mut goto_labels)?;
            let mut resolved_block = resolve_block(block, var_map, &goto_labels)?;
            check_and_classify_block_break_statements(&mut resolved_block, &None)?;
            label_block_loops(&mut resolved_block, &None)?;
            label_block_switch_statements(&mut resolved_block, &None, &mut vec![], &mut HashMap::new(), &mut None)?;
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
