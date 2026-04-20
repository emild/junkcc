use std::collections::HashMap;

use env_logger::init;

use super::super::ast::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    FuncType(usize /* number of parameters */, bool /* has_body, i.e. defined */)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitialValue {
    Tentative,
    Initial(i32),
    NoInitializer
}

#[derive(Debug, PartialEq, Eq)]
pub enum IdentifierAttrs {
    FuncAttr(bool /* defined */, bool /* global */),        /* Functions */
    StaticAttr(InitialValue /* init */, bool /* global */), /* Variables with static duration */
    LocalAttr                                               /* Parameters or automatic variables */
}

#[derive(Debug)]
pub struct SymbolInfo {
    pub typ: Type,
    pub attrs: IdentifierAttrs
}


fn typecheck_expr_function_call(func_name: &String, args: &Vec<TypedExpression>, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    let func_type = symbol_table.get(func_name);
    match func_type {
        None => {
            return Err(format!("Function '{func_name}()' not declared in scope"));
        },
        Some(SymbolInfo{ typ: Type::FuncType(num_params, _), attrs: _}) => {
            if *num_params != args.len() {
                return Err(format!(
                    "Function '{}()' called with wrong number of arguments (function has {} parameters and is called with {} arguments)",
                    func_name,
                    num_params,
                    args.len()
                ));
            }
        },
        Some(SymbolInfo{ typ: Type::Int, attrs: _}) => {
            return Err(format!("'{}' is not a function or a callable object", func_name));
        }
    };

    for arg in args {
        typecheck_expression(arg, symbol_table)?;
    }

    Ok(())

}


fn typecheck_expr_var(var_name: &String, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    let var_type = symbol_table.get(var_name);
    match var_type {
        None => {
            return Err(format!("Variable '{}' not declared in scope", var_name));
        },
        Some(SymbolInfo{ typ: Type::Int, attrs: _} ) => {},
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


fn typecheck_expr_assignment(typed_left: &TypedExpression, typed_right: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    let TypedExpression::TypedExp(_, left) = typed_left;
    match left {
        Expression::Var(var_name) => {
            typecheck_expr_var(var_name, symbol_table)?;
        }
        _ => { return Err(format!("Assignment to non-lvalue")); }
    }

    typecheck_expression(typed_right, symbol_table)?;

    Ok(())
}


fn typecheck_expr_binary(typed_left: &TypedExpression, typed_right: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    typecheck_expression(typed_left, symbol_table)?;
    typecheck_expression(typed_right, symbol_table)?;

    //TODO: Check that both left and right have the same type

    Ok(())
}


fn typecheck_expr_conditional(cond: &TypedExpression, true_expr: &TypedExpression, false_expr: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    typecheck_expression(cond, symbol_table)?;
    typecheck_expression(true_expr, symbol_table)?;
    typecheck_expression(false_expr, symbol_table)?;

    //TODO: need to check that true_expr and false_expr have both the same type

    Ok(())
}


fn typecheck_expr_inc_dec(typed_expr: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    let TypedExpression::TypedExp(_, expr) = typed_expr;
    match expr {
        Expression::Var(var_name) => {
            typecheck_expr_var(var_name, symbol_table)?;
        },
        _ => { return Err(format!("Non-lvalue for increment/decrement"));
        }
    };


    Ok(())
}


fn typecheck_expr_unary(typed_expr: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    typecheck_expression(typed_expr, symbol_table)?;

    Ok(())
}


fn typecheck_expression(typed_expr: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    let TypedExpression::TypedExp(_, expr) = typed_expr;
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
        Expression::Constant(_) => {

        },
        Expression::PostDecrement(expr) |
        Expression::PostIncrement(expr) |
        Expression::PreDecrement(expr) |
        Expression::PreIncrement(expr) => {
            typecheck_expr_inc_dec(expr, symbol_table)?;
        }

        Expression::Unary(_,expr ) => {
            typecheck_expr_unary(expr, symbol_table)?;
        },
        Expression::Cast(_,_) => {
            panic!("EMIL: Cast expression not implemented [YET]");
        }

    }
    Ok(())
}


fn typecheck_local_variable_declaration(var_decl: &VariableDeclaration, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    let VariableDeclaration::Declarant(var_name, initializer, typ, stg_class) = var_decl;

    match stg_class {
        Some(StorageClass::Extern) => {
            if !initializer.is_none() {
                return Err(format!("Local Extern variable declaration of '{}' cannot have initializer", var_name));
            }

            if let Some(old_decl) = symbol_table.get(var_name) {
                if old_decl.typ != Type::Int {
                    return Err(format!("Function '{}()' is redeclared as variable", var_name));
                }
            }
            else {
                symbol_table.insert(
                    var_name.clone(),
                    SymbolInfo { typ: Type::Int, attrs: IdentifierAttrs::StaticAttr(InitialValue::NoInitializer, true) }
                );
            }
        },

        Some(StorageClass::Static) => {
            let initial_value = match initializer {
                //TODO: Handle Long constants
                Some(TypedExpression::TypedExp(_, Expression::Constant(Const::ConstInt(init_val)))) => {
                    InitialValue::Initial(*init_val)
                },
                None => {
                    InitialValue::Initial(0)
                },
                _ => {
                    return Err(format!("Non-constant initializer for local static variable '{}'", var_name));
                }
            };

            symbol_table.insert(
                var_name.clone(),
                SymbolInfo { typ: Type::Int, attrs: IdentifierAttrs::StaticAttr(initial_value, false) }
            );
        },

        _ => {
            symbol_table.insert(
                var_name.clone(),
                SymbolInfo{ typ: Type::Int, attrs: IdentifierAttrs::LocalAttr }
            );

            if let Some(initializer) = initializer {
                typecheck_expression(initializer, symbol_table)?;
            }
        }
    };

    Ok(())

}


fn typecheck_file_scope_variable_declaration(var_decl: &VariableDeclaration, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    let VariableDeclaration::Declarant(var_name, initializer, typ, stg_class) = var_decl;
    let mut initial_value = match initializer {
        //TODO: Handle long constants
        Some(TypedExpression::TypedExp(_, Expression::Constant(Const::ConstInt(init_val)))) => {
            InitialValue::Initial(*init_val)
        },
        None => {
            if *stg_class == Some(StorageClass::Extern) {
                InitialValue::NoInitializer
            }
            else {
                InitialValue::Tentative
            }
        },
        _ => {
            return Err(format!("Non-constant initializer for variable '{}'", var_name));
        }
    };

    let mut global = *stg_class != Some(StorageClass::Static);

    if let Some(old_decl) = symbol_table.get(var_name) {
        match old_decl.typ {
            Type::Int => {
            },
            _ => {
                return Err(format!("Function '{}()' redeclared as variable", var_name));
            }
        };

        let (old_init, old_global) = match &old_decl.attrs {
            IdentifierAttrs::StaticAttr(old_init_val, old_global ) => {
                (old_init_val, old_global)
            },
            _ => {
                return Err(format!("Wrong attribute for file scope Svariable '{}' Expected StaticAttrs, got '{:?}'.", var_name, old_decl.attrs));
            }
        };

        if *stg_class == Some(StorageClass::Extern) {
            global = *old_global;
        }
        else if global != *old_global {
            return Err(format!("Conflicting linkage type for file scope variable '{}'", var_name));
        }

        if let InitialValue::Initial(_) = old_init {
            if let InitialValue::Initial(_) = initial_value {
                return Err(format!("Conflicting initializers for file scope variable '{}'", var_name));
            }
            else {
                initial_value = old_init.clone();
            }
        }
        else if let InitialValue::Initial(_) = initial_value {

        }
        else if let IdentifierAttrs::StaticAttr(InitialValue::Tentative, _) = old_decl.attrs {
            initial_value = InitialValue::Tentative;
        }

    }


    let attrs = IdentifierAttrs::StaticAttr(initial_value, global);
    symbol_table.insert(
        var_name.clone(),
        SymbolInfo { typ: Type::Int, attrs }
    );

    Ok(())
}


fn typecheck_function_declaration(func_decl: &FunctionDeclaration, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match func_decl {
        FunctionDeclaration::Declarant(func_name, params, body, typ, stg_class ) => {
            let func_type = params.len();
            let has_body = body.is_some();
            let mut already_defined = false;

            let mut global = *stg_class != Some(StorageClass::Static);

            if let Some(old_decl) = symbol_table.get(func_name) {
                match old_decl.typ {
                    Type::FuncType(old_type, old_defined) => {
                        if old_type != func_type {
                            return Err(format!("Incompatible redeclaration of function '{}()' ({} vs. {} parameters)", func_name, func_type, old_type));
                        }

                        already_defined = old_defined;
                        if already_defined && has_body {
                            return Err(format!("Redefinition of function '{}()'. Function '{}()' already has a body", func_name, func_name));
                        }

                        global = match old_decl.attrs {
                            IdentifierAttrs::FuncAttr(_, true) => {
                                if *stg_class == Some(StorageClass::Static) {
                                    return Err(format!("Static function declaration follows a non-static one"));
                                }
                                true
                            },
                            IdentifierAttrs::FuncAttr(_, false) => {
                                false
                            },
                            _ => {
                                return Err(format!("Function '{}()'has non FuncAttrs '{:?}'", func_name, old_decl.attrs));
                            }
                        };
                    },
                    Type::Int => {
                        return Err(format!("Function '{}()' redefines variable '{}", func_name, func_name));
                    }
                }
            }


            let attrs = IdentifierAttrs::FuncAttr(already_defined || has_body, global);
            let typ = Type::FuncType(params.len(), already_defined || has_body);

            symbol_table.insert(func_name.clone(), SymbolInfo { typ, attrs });

            if has_body {
                for param in params {
                    symbol_table.insert(
                        param.clone(),
                        SymbolInfo { typ: Type::Int, attrs: IdentifierAttrs::LocalAttr}
                    );
                }

                typecheck_block(&body.as_ref().unwrap(), symbol_table)?;
            }

            Ok(())
        }
    }
}


fn typecheck_local_declaration(decl: &Declaration, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match decl {
        Declaration::FunDecl(func_decl) => {
            typecheck_function_declaration(&func_decl, symbol_table)?;
        },
        Declaration::VarDecl(var_decl) => {
            typecheck_local_variable_declaration(&var_decl, symbol_table)?;
        }
    };

    Ok(())
}


fn  typecheck_unlabeled_statement(unlabeled_statement: &UnlabeledStatement, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
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
                    typecheck_local_variable_declaration(var_decl, symbol_table)?;
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


fn typecheck_statement(stmnt: &Statement, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(_, unlabeled_statement ) => {
            typecheck_unlabeled_statement(unlabeled_statement, symbol_table)?;
        }
    };

    Ok(())
}


fn typecheck_block_item(block_item: &BlockItem, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            typecheck_local_declaration(decl, symbol_table)?;
        },
        BlockItem::S(stmnt) => {
            typecheck_statement(stmnt, symbol_table)?;
        }
    };

    Ok(())
}


fn typecheck_block(block: &Block, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
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


fn typecheck_file_scope_declaration(decl: &Declaration, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match decl {
        Declaration::FunDecl(func_decl) => {
            typecheck_function_declaration(&func_decl, symbol_table)?;
        },
        Declaration::VarDecl(var_decl) => {
            typecheck_file_scope_variable_declaration(&var_decl, symbol_table)?;
        }
    };

    Ok(())
}


pub fn typecheck_program(prog: &Program, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match prog {
        Program::ProgramDefinition(decls) => {
            for decl in decls {
                typecheck_file_scope_declaration(decl, symbol_table)?;
            }

            Ok(())
        }
    }
}
