use std::collections::HashMap;

use super::super::super::parser::ast::*;

use super::break_classifier::check_and_classify_block_break_statements;
use super::goto_labels::check_block_goto_labels;
use super::loop_labeling::label_block_loops;
use super::switch_labeling::label_block_switch_statements;
use super::IdentifierInfo;
use super::constant_expression_evaluator::evaluate_constant_expression;
use super::unique_global_labels::make_unique_global_name_for_local_variable;
use super::unique_global_labels::make_unique_global_name_for_parameter;


fn resolve_expression(expr: &Expression, identifier_map: &mut HashMap<String, IdentifierInfo>) -> Result<Expression, String>
{
    match expr {
        Expression::Assignment(left, right ) => {
            match **left {
                Expression::Var(_) => {
                    let resolved_left = resolve_expression(&left, identifier_map)?;
                    let resolved_right = resolve_expression(right, identifier_map)?;

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
                    let resolved_left = resolve_expression(&left, identifier_map)?;
                    let resolved_right = resolve_expression(right, identifier_map)?;

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
                    let resolved_expr = resolve_expression(expr, identifier_map)?;
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
                    let resolved_expr = resolve_expression(expr, identifier_map)?;
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
                    let resolved_expr = resolve_expression(expr, identifier_map)?;
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
                    let resolved_expr = resolve_expression(expr, identifier_map)?;
                    Ok(Expression::PostDecrement(Box::new(resolved_expr)))
                },
                _ => {
                    return Err(format!("Non lval in post-decrement"));
                }
            }
        },

        Expression::Var(var_name) => {
            let var_info = identifier_map.get(var_name);

            if var_info.is_none() {
                return Err(format!("Use of undeclared variable '{}'", var_name));
            }

            Ok(Expression::Var(var_info.unwrap().global_name.clone()))
        },

        Expression::FunctionCall(func_name, args) => {
            let func_info = identifier_map.get(func_name);

            if func_info.is_none() {
                return Err(format!("Calling undeclared function '{}'", func_name));
            }

            let new_func_name = func_info.unwrap().global_name.clone();
            let mut new_args = vec![];

            for arg in args {
                let new_arg = resolve_expression(arg, identifier_map)?;
                new_args.push(new_arg);
            }

            Ok(Expression::FunctionCall(new_func_name, new_args))
        },

        Expression::Conditional(cond, true_exp, false_exp) => {
            let resolved_cond = resolve_expression(cond, identifier_map)?;
            let resolved_true_exp = resolve_expression(true_exp, identifier_map)?;
            let resolved_false_exp = resolve_expression(false_exp, identifier_map)?;

            Ok(Expression::Conditional(Box::new(resolved_cond), Box::new(resolved_true_exp), Box::new(resolved_false_exp)))
        },

        Expression::Binary(binary_op,left, right , ) => {
            let resolved_left = resolve_expression(left, identifier_map)?;
            let resolved_right = resolve_expression(right, identifier_map)?;

            Ok(Expression::Binary(binary_op.clone(), Box::new(resolved_left), Box::new(resolved_right)))
        },

        Expression::Unary(unary_op, expr) => {
            let resolved_expr = resolve_expression(expr, identifier_map)?;

            Ok(Expression::Unary(unary_op.clone(), Box::new(resolved_expr)))
        },

        Expression::IntConstant(c) => {
            Ok(Expression::IntConstant(*c))
        }

    }
}


fn resolve_statement(stmnt: &Statement, identifier_map: &mut HashMap<String, IdentifierInfo>, goto_labels: &HashMap<String, String>) -> Result<Statement, String>
{
    match stmnt {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            let resolved_unlabled_stmnt = resolve_unlabeled_statement(unlabeled_stmnt, identifier_map, goto_labels)?;
            return Ok(Statement::Stmnt(None, resolved_unlabled_stmnt));
        },
        Statement::Stmnt(Some(stmnt_labels), unlabeled_stmnt) => {
            let resolved_unlabled_stmnt = resolve_unlabeled_statement(unlabeled_stmnt, identifier_map, goto_labels)?;
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
fn copy_identifier_map(identifier_map: &HashMap<String, IdentifierInfo>) -> HashMap<String, IdentifierInfo>
{
    let mut new_identifier_map = HashMap::new();
    for (identifier_name, identifier_info) in identifier_map {
        new_identifier_map.insert(
            identifier_name.clone(),
            IdentifierInfo {global_name: identifier_info.global_name.clone(), from_current_scope: false, has_linkage: identifier_info.has_linkage }
        );
    }

    new_identifier_map
}


fn resolve_for_init(for_init: &ForInit, identifier_map: &mut HashMap<String, IdentifierInfo>) -> Result<ForInit, String>
{
    let resolved_for_init = match for_init {
        ForInit::InitExp(None) => ForInit::InitExp(None),
        ForInit::InitExp(Some(expr)) => {
            let resolved_expr = resolve_expression(expr, identifier_map)?;
            ForInit::InitExp(Some(resolved_expr))
        },
        ForInit::InitDecl(decl) => {
            let resolved_decl = resolve_variable_declaration(decl, identifier_map)?;
            ForInit::InitDecl(resolved_decl)
        }
    };

    Ok(resolved_for_init)
}


fn resolve_unlabeled_statement(stmnt: &UnlabeledStatement, identifier_map: &mut HashMap<String, IdentifierInfo>, goto_labels: &HashMap<String, String>) -> Result<UnlabeledStatement, String>
{
    match stmnt {
        UnlabeledStatement::Return(expr) => {
            let resolved_expression = resolve_expression(expr, identifier_map)?;
            Ok(UnlabeledStatement::Return(resolved_expression))
        },
        UnlabeledStatement::Goto(label) => {
            if !goto_labels.contains_key(label) {
                return Err(format!("Label '{}' not found", label));
            }
            Ok(UnlabeledStatement::Goto(goto_labels.get(label).unwrap().clone()))
        },
        UnlabeledStatement::If(cond, then_stmnt , else_stmnt) => {
            let resolved_cond = resolve_expression(cond, identifier_map)?;
            let resolved_then_stmnt = resolve_statement(then_stmnt, identifier_map, goto_labels)?;
            let resolved_else_stmnt = if let Some(else_stmnt) = else_stmnt {
                let resolved_else_stmnt = resolve_statement(else_stmnt, identifier_map, goto_labels)?;
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
            let resolved_cond = resolve_expression(cond, identifier_map)?;
            let resolved_body = resolve_statement(body, identifier_map, goto_labels)?;
            Ok(UnlabeledStatement::While(resolved_cond, Box::new(resolved_body), loop_label.clone()))
        },
        UnlabeledStatement::DoWhile(body, cond, loop_label) => {
            let resolved_cond = resolve_expression(cond, identifier_map)?;
            let resolved_body = resolve_statement(body, identifier_map, goto_labels)?;
            Ok(UnlabeledStatement::DoWhile(Box::new(resolved_body), resolved_cond, loop_label.clone()))
        },
        UnlabeledStatement::For(for_init, cond, post, body, loop_label) => {
            let mut new_identifier_map = copy_identifier_map(identifier_map);
            let resolved_for_init = resolve_for_init(for_init, &mut new_identifier_map)?;
            let mut resolved_cond = None;
            if let Some(expr) = cond {
                let resolved_expr = resolve_expression(expr, &mut new_identifier_map)?;
                resolved_cond = Some(resolved_expr);
            }
            let mut resolved_post = None;
            if let Some(expr) = post {
                let resolved_expr = resolve_expression(expr, &mut new_identifier_map)?;
                resolved_post = Some(resolved_expr);
            }
            let resolved_body = resolve_statement(body, &mut new_identifier_map, goto_labels)?;
            Ok(UnlabeledStatement::For(resolved_for_init, resolved_cond, resolved_post, Box::new(resolved_body), loop_label.clone()))
        },
        UnlabeledStatement::Switch(cond, body, switch_label, case_label_map, default_label) => {
            let resolved_cond = resolve_expression(cond, identifier_map)?;
            let resolved_body = resolve_statement(body, identifier_map, goto_labels)?;
            Ok(UnlabeledStatement::Switch(resolved_cond, Box::new(resolved_body), switch_label.clone(), case_label_map.clone(), default_label.clone()))
        }
        UnlabeledStatement::Compound(block) => {
            let mut new_identifier_map = copy_identifier_map(identifier_map);
            let resolved_block = resolve_block(block, &mut new_identifier_map, goto_labels)?;
            Ok(UnlabeledStatement::Compound(resolved_block))
        },
        UnlabeledStatement::Expr(expr) => {
            let resolved_expression = resolve_expression(expr, identifier_map)?;
            Ok(UnlabeledStatement::Expr(resolved_expression))
        },
        UnlabeledStatement::Null => Ok(UnlabeledStatement::Null),
        _ => { panic!("Semantic Analyzer: {:?} Not implemented yet!", stmnt); }
    }
}




fn resolve_variable_declaration(decl: &VariableDeclaration, identifier_map: &mut HashMap<String, IdentifierInfo>) -> Result<VariableDeclaration, String>
{
    match decl {
        VariableDeclaration::Declarant(var_name, initializer) => {
            if let Some(local_var_info) = identifier_map.get(var_name) && local_var_info.from_current_scope {
                return Err(format!("Variable '{}' is already defined in the current scope", var_name));
            }

            let temp_name = make_unique_global_name_for_local_variable(var_name);

            identifier_map.insert(
                var_name.clone(),
                IdentifierInfo { global_name: temp_name.clone(), from_current_scope: true, has_linkage: false }
            );

            let resolved_initializer = match initializer {
                Some(init_expression) => {
                    let resolved_init_expression = resolve_expression(init_expression, identifier_map)?;
                    Some(resolved_init_expression)
                },
                None => None
            };

            Ok(VariableDeclaration::Declarant(temp_name, resolved_initializer))
        }
    }
}


fn resolve_param(param: &String, identifier_map: &mut HashMap<String, IdentifierInfo>) -> Result<String, String>
{

    if let Some(param_info) = identifier_map.get(param) && param_info.from_current_scope {
        return Err(format!("Parameter '{}' is already defined in the current scope", param));
    }

    let resolved_param = make_unique_global_name_for_parameter(param);

    identifier_map.insert(
        param.clone(),
        IdentifierInfo { global_name: resolved_param.clone(), from_current_scope: true, has_linkage: false }
    );


    Ok(resolved_param)
}


fn resolve_function_declaration(func_decl: &FunctionDeclaration, identifier_map: &mut HashMap<String, IdentifierInfo>) -> Result<FunctionDeclaration, String>
{
    match func_decl {
        FunctionDeclaration::Declarant(func_name, params, body) => {
            let prev_entry = identifier_map.get(func_name);
            if let Some(prev_entry) = prev_entry {
                if prev_entry.from_current_scope && !prev_entry.has_linkage {
                    return Err(format!("Duplicate declaration: '{}'", func_name));
                }
            }
            identifier_map.insert(func_name.clone(), IdentifierInfo { global_name: func_name.clone(), from_current_scope: true, has_linkage: true });
            let mut inner_map = copy_identifier_map(identifier_map);
            let mut new_params = vec![];
            for param in params {
                let new_param = resolve_param(param, &mut inner_map)?;
                new_params.push(new_param);
            }

            let mut new_body = None;

            if let Some(body) = body {
                let mut goto_labels = HashMap::new();
                check_block_goto_labels(&body, &mut goto_labels)?;
                let mut resolved_body = resolve_block(body, &mut inner_map, &mut goto_labels)?;
                check_and_classify_block_break_statements(&mut resolved_body, &None)?;
                label_block_loops(&mut resolved_body, &None)?;
                label_block_switch_statements(&mut resolved_body, &None, &mut HashMap::new(), &mut None)?;
                new_body.replace(resolved_body);
            }

            Ok(FunctionDeclaration::Declarant(func_name.clone(), new_params, new_body))
        }
    }

}


fn resolve_local_declaration(decl: &Declaration, identifier_map: &mut HashMap<String, IdentifierInfo>) -> Result<Declaration, String>
{
    match decl {
        Declaration::VarDecl(VariableDeclaration::Declarant(var_name, initializer )) => {
            let resolved_var_decl = resolve_variable_declaration(&VariableDeclaration::Declarant(var_name.clone(), initializer.clone()), identifier_map)?;
            Ok(Declaration::VarDecl(resolved_var_decl))
        },
        Declaration::FunDecl(func_decl) => {
            match func_decl {
                FunctionDeclaration::Declarant(_,_,Some(_)) => {
                    return Err(format!("Local function definitions are not allowed"));
                },
                _ => ()
            }
            let resolved_func_decl = resolve_function_declaration(func_decl, identifier_map)?;
            Ok(Declaration::FunDecl(resolved_func_decl))
        }
    }
}


fn resolve_block_item(block_item: &BlockItem, identifier_map: &mut HashMap<String, IdentifierInfo>, labels: &HashMap<String, String>) -> Result<BlockItem, String>
{
    match block_item {
        BlockItem::D(decl) => {
            let resolved_decl = resolve_local_declaration(decl, identifier_map)?;
            Ok(BlockItem::D(resolved_decl))
        },
        BlockItem::S(stmnt) => {
            let resolved_stmnt = resolve_statement(stmnt, identifier_map, labels)?;
            Ok(BlockItem::S(resolved_stmnt))
        }
    }
}




fn resolve_block(block: &Block, identifier_map: &mut HashMap<String, IdentifierInfo>, goto_labels: &HashMap<String, String>) -> Result<Block, String>
{
    let mut resolved_block_items = vec![];

    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                let resolved_block_item = resolve_block_item(&block_item, identifier_map, goto_labels)?;
                resolved_block_items.push(resolved_block_item);
            }
        }
    }
    Ok(Block::Blk(resolved_block_items))
}



pub fn resolve_program(prog: &Program) -> Result<Program, String>
{
    match prog {
        Program::ProgramDefinition(func_decls) => {
            let mut identifier_map = HashMap::new();
            let mut resolved_func_decls = vec![];
            for func_decl in func_decls {
                let resolved_func_decl = resolve_function_declaration(func_decl, &mut identifier_map)?;
                resolved_func_decls.push(resolved_func_decl);
            }
            Ok(Program::ProgramDefinition(resolved_func_decls))
        }
    }
}