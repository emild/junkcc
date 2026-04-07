use std::collections::HashMap;

use super::super::ast::*;

#[derive(Debug)]
pub enum Type {
    Int,
    FuncType(usize /* number of parameters */, bool /* has_body, i.e. defined */)
}


fn typecheck_expr_function_call(func_name: &String, args: &Vec<Expression>, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    let func_type = symbol_table.get(func_name);
    match func_type {
        None => {
            return Err(format!("Function '{func_name}()' not declared in scope"));
        },
        Some(Type::FuncType(num_params, _)) => {
            if *num_params != args.len() {
                return Err(format!(
                    "Function '{}()' called with wrong number of arguments (function has {} parameters and is called with {} arguments)",
                    func_name,
                    num_params,
                    args.len()
                ));
            }
        },
        Some(Type::Int) => {
            return Err(format!("'{}' is not a function or a callable object", func_name));
        }
    };

    for arg in args {
        typecheck_expression(arg, symbol_table)?;
    }

    Ok(())

}


fn typecheck_expr_var(var_name: &String, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    let var_type = symbol_table.get(var_name);
    match var_type {
        None => {
            return Err(format!("Variable '{}' not declared in scope", var_name));
        },
        Some(Type::Int) => {},
        _ => {
            return Err(format!(
                "Object '{}' has wrong type. Expected: '{:?}', actual type: '{:?}'",
                var_name,
                Type::Int,
                var_type
            ));
        }
    }

    Ok(())
}


fn typecheck_expr_assignment(left: &Expression, right: &Expression, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match left {
        Expression::Var(var_name) => {
            typecheck_expr_var(var_name, symbol_table)?;
        }
        _ => { return Err(format!("Assignment to non-lvalue")); }
    }

    typecheck_expression(right, symbol_table)?;

    Ok(())
}


fn typecheck_expr_binary(left: &Expression, right: &Expression, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{

    typecheck_expression(left, symbol_table)?;
    typecheck_expression(right, symbol_table)?;

    //TODO: Check that both left and right have the same type

    Ok(())
}


fn typecheck_expr_conditional(cond: &Expression, true_expr: &Expression, false_expr: &Expression, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    typecheck_expression(cond, symbol_table)?;
    typecheck_expression(true_expr, symbol_table)?;
    typecheck_expression(false_expr, symbol_table)?;

    //TODO: need to check that true_expr and false_expr have both the same type

    Ok(())
}


fn typecheck_expr_inc_dec(expr: &Expression, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match expr {
        Expression::Var(var_name) => {
            typecheck_expr_var(var_name, symbol_table)?;
        },
        _ => { return Err(format!("Non-lvalue for increment/decrement"));
        }
    };


    Ok(())
}


fn typecheck_expr_unary(expr: &Expression, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    typecheck_expression(expr, symbol_table)?;

    Ok(())
}


fn typecheck_expression(expr: &Expression, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match expr {
        Expression::FunctionCall(func_name, args) => {
            typecheck_expr_function_call(func_name, args, symbol_table)?;
        },
        Expression::Var(var_name) => {
            typecheck_expr_var(var_name, symbol_table)?;
        },
        Expression::Assignment(left, right) |
        Expression::CompoundAssignment(_,left , right) => {
            typecheck_expr_assignment(left, right, symbol_table)?;
        },
        Expression::Binary(_,left , right) => {
            typecheck_expr_binary(left, right, symbol_table)?;
        },
        Expression::Conditional(cond, true_expr, false_expr ) => {
            typecheck_expr_conditional(cond, true_expr, false_expr, symbol_table)?;
        },
        Expression::IntConstant(_) => {

        },
        Expression::PostDecrement(expr) |
        Expression::PostIncrement(expr) |
        Expression::PreDecrement(expr) |
        Expression::PreIncrement(expr) => {
            typecheck_expr_inc_dec(expr, symbol_table)?;
        }

        Expression::Unary(_,expr ) => {
            typecheck_expr_unary(expr, symbol_table)?;
        }

    }
    Ok(())
}


fn typecheck_variable_declaration(var_decl: &VariableDeclaration, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match var_decl {
        VariableDeclaration::Declarant(var_name, initializer ) => {
            symbol_table.insert(var_name.clone(), Type::Int);
            if let Some(initializer) = initializer {
                typecheck_expression(initializer, symbol_table)?;
            }

            Ok(())
        }
    }
}


fn typecheck_function_declaration(func_decl: &FunctionDeclaration, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match func_decl {
        FunctionDeclaration::Declarant(func_name, params, body) => {
            let func_type = params.len();
            let has_body = body.is_some();
            let mut already_defined = false;

            if let Some(old_decl) = symbol_table.get(func_name) {
                match old_decl {
                    Type::FuncType(old_type, old_defined) => {
                        if *old_type != func_type {
                            return Err(format!("Incompatible redeclaration of function '{}()' ({} vs. {} parameters)", func_name, func_type, old_type));
                        }

                        already_defined = *old_defined;
                        if already_defined && has_body {
                            return Err(format!("Redefinition of function '{}()'. Function '{}()' already has a body", func_name, func_name));
                        }
                    },
                    Type::Int => {
                        return Err(format!("Function '{}()' redefines variable '{}", func_name, func_name));
                    }
                }
            }

            symbol_table.insert(func_name.clone(), Type::FuncType(params.len(), already_defined || has_body));

            if has_body {
                for param in params {
                    symbol_table.insert(param.clone(), Type::Int);
                }

                typecheck_block(&body.as_ref().unwrap(), symbol_table)?;
            }

            Ok(())
        }
    }
}


fn typecheck_declaration(decl: &Declaration, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match decl {
        Declaration::FunDecl(func_decl) => {
            typecheck_function_declaration(&func_decl, symbol_table)?;
        },
        Declaration::VarDecl(var_decl) => {
            typecheck_variable_declaration(&var_decl, symbol_table)?;
        }
    };

    Ok(())
}


fn  typecheck_unlabeled_statement(unlabeled_statement: &UnlabeledStatement, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match unlabeled_statement {
        UnlabeledStatement::Break(_,_) |
        UnlabeledStatement::Continue(_) |
        UnlabeledStatement::Goto(_) |
        UnlabeledStatement::Null => {},

        UnlabeledStatement::Expr(expr) |
        UnlabeledStatement::Return(expr) => {
            typecheck_expression(expr, symbol_table)?;
        },

        UnlabeledStatement::If(cond, then_stmnt, else_stmnt) => {
            typecheck_expression(cond, symbol_table)?;
            typecheck_statement(then_stmnt, symbol_table)?;
            if let Some(else_stmnt) = else_stmnt {
                typecheck_statement(else_stmnt, symbol_table)?;
            }
        },

        UnlabeledStatement::While(cond, body, _) |
        UnlabeledStatement::DoWhile(body, cond, _) => {
            typecheck_expression(cond, symbol_table)?;
            typecheck_statement(body, symbol_table)?;
        },

        UnlabeledStatement::For(for_init, cond, post, body, _) => {
            match for_init {
                ForInit::InitDecl(var_decl) => {
                    typecheck_variable_declaration(var_decl, symbol_table)?;
                },
                ForInit::InitExp(Some(init_expr)) => {
                    typecheck_expression(init_expr, symbol_table)?;
                }
                ForInit::InitExp(None) => {}
            };

            if let Some(cond) = cond {
                typecheck_expression(cond, symbol_table)?;
            }

            if let Some(post) = post {
                typecheck_expression(post, symbol_table)?;
            }

            typecheck_statement(body, symbol_table)?;
        },

        UnlabeledStatement::Switch(cond, body, _, _, _) => {
            typecheck_expression(cond, symbol_table)?;
            typecheck_statement(body, symbol_table)?;
        },

        UnlabeledStatement::Compound(block) => {
            typecheck_block(block, symbol_table)?;
        }

    };

    Ok(())
}


fn typecheck_statement(stmnt: &Statement, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(_, unlabeled_statement ) => {
            typecheck_unlabeled_statement(unlabeled_statement, symbol_table)?;
        }
    };

    Ok(())
}


fn typecheck_block_item(block_item: &BlockItem, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            typecheck_declaration(decl, symbol_table)?;
        },
        BlockItem::S(stmnt) => {
            typecheck_statement(stmnt, symbol_table)?;
        }
    };

    Ok(())
}


fn typecheck_block(block: &Block, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                typecheck_block_item(block_item, symbol_table)?;
            }

            Ok(())
        }
    }
}

pub fn typecheck_program(prog: &Program, symbol_table: &mut HashMap<String, Type>) -> Result<(), String>
{
    match prog {
        Program::ProgramDefinition(func_decls) => {
            for func_decl in func_decls {
                typecheck_function_declaration(func_decl, symbol_table)?;
            }

            Ok(())
        }
    }
}