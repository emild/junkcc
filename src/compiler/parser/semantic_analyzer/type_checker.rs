use std::collections::HashMap;

use env_logger::init;

use crate::compiler::parser::semantic_analyzer::switch_labeling::label_block_switch_statements;

use super::super::ast::*;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitialValue {
    Tentative,
    Initial(StaticInit),
    NoInitializer
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaticInit {
    IntInit(i32),
    LongInit(i64)
}

impl StaticInit
{
    pub fn convert_to(&self, typ: &Type) -> Self
    {
        assert!(!typ.is_func());
        match (self, typ) {
            (StaticInit::IntInit(_), Type::Int) |
            (StaticInit::LongInit(_), Type::Long) => self.clone(),
            (StaticInit::IntInit(c), Type::Long) => StaticInit::LongInit(i64::from(*c)),
            (StaticInit::LongInit(c), Type::Int) => StaticInit::IntInit((*c & 0xFFFFFFFF) as i32),
            _ => { panic!("Invalid StaticInit Conversion"); }
        }
    }
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


fn get_common_type(typ1: &Type, typ2: &Type) -> Type
{
    if typ1 == typ2 {
        typ1.clone()
    }
    else {
        Type::Long
    }
}


fn convert_to(typex: TypedExpression, target_typ: &Type) -> TypedExpression
{
    let typ = typex_get_type(&typex);
    if typ == *target_typ {
        typex
    }
    else {
        typex_set_type(Expression::Cast(target_typ.clone(), Box::new(typex)), target_typ.clone())
    }
}


fn typecheck_expr_function_call(func_name: &String, args: &Vec<TypedExpression>, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<TypedExpression, String>
{
    let func_type = symbol_table.get(func_name);
    let ret_val = match func_type {
        None => {
            return Err(format!("Function '{func_name}()' not declared in scope"));
        },
        Some(SymbolInfo{ typ: Type::FuncType(param_types, ret_type, _), attrs: _}) => {
            let ret_type = *ret_type.clone();

            if param_types.len() != args.len() {
                return Err(format!(
                    "Function '{}()' called with wrong number of arguments (function has {} parameters and is called with {} arguments)",
                    func_name,
                    param_types.len(),
                    args.len()
                ));
            }

            let param_types = param_types.clone();
            let mut converted_args = vec![];
            for (arg, param_type) in args.into_iter().zip(param_types) {
                let typed_arg = typecheck_expression(arg, symbol_table)?;
                let converted_arg = convert_to(typed_arg, &param_type);
                converted_args.push(converted_arg);
            }

            let call_expr = Expression::FunctionCall(func_name.clone(), converted_args);
            typex_set_type(call_expr, Type::Int)
        },
        Some(SymbolInfo{ typ: _, attrs: _}) => {
            return Err(format!("'{}' is not a function or a callable object", func_name));
        }
    };

    Ok(ret_val)

}



fn typecheck_expr_var(var_name: &String, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<TypedExpression, String>
{
    let sym_info = symbol_table.get(var_name);
    let checked_type = match sym_info {
        None => {
            return Err(format!("Variable '{}' not declared in scope", var_name));
        },
        Some(SymbolInfo{ typ: Type::Int, attrs: _}) => Type::Int,
        Some(SymbolInfo{ typ: Type::Long, attrs: _}) => Type::Long,
        _ => {
            return Err(format!(
                "Object '{}' has wrong type. Expected some integer type, actual type: '{:?}'",
                var_name,
                sym_info.as_ref().unwrap().typ
            ));
        }
    };

    Ok(typex_set_type(Expression::Var(var_name.clone()), checked_type))
}


fn typecheck_expr_assignment(binop: &Option<BinaryOperator>, left: &TypedExpression, right: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<TypedExpression, String>
{
    let typed_left = typecheck_expression(left, symbol_table)?;
    let typed_right = typecheck_expression(right, symbol_table)?;
    let typ_left = typex_get_type(&typed_left);
    let converted_right = convert_to(typed_right, &typ_left);

    match typed_left {
        TypedExpression::TypedExp(_, Expression::Var(_)) => { },
        _ => { return Err(format!("Assignment to non-lvalue")); }
    };

    let result_expr = match binop {
        Some(binop) => Expression::CompoundAssignment(binop.clone(), Box::new(typed_left), Box::new(converted_right)),
        None => Expression::Assignment(Box::new(typed_left), Box::new(converted_right))
    };

    Ok(typex_set_type(result_expr, typ_left))
}


fn typecheck_expr_binary(binop: &BinaryOperator, typed_left: &TypedExpression, typed_right: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<TypedExpression, String>
{
    let typed_left = typecheck_expression(typed_left, symbol_table)?;
    let typed_right = typecheck_expression(typed_right, symbol_table)?;

    let (result_expr, result_typ ) = match binop {
        BinaryOperator::LogicalAnd |
        BinaryOperator::LogicalOr => {
            (Expression::Binary(binop.clone(), Box::new(typed_left), Box::new(typed_right)), Type::Int)
        },

        BinaryOperator::Add |
        BinaryOperator::Subtract |
        BinaryOperator::Multiply |
        BinaryOperator::Divide |
        BinaryOperator::Remainder |
        BinaryOperator::BitwiseAnd |
        BinaryOperator::BitwiseOr |
        BinaryOperator::BitwiseXor => {
            let typ_left = typex_get_type(&typed_left);
            let typ_right = typex_get_type(&typed_right);
            let typ_common  = get_common_type(&typ_left, &typ_right);
            let converted_left = convert_to(typed_left, &typ_common);
            let converted_right = convert_to(typed_right, &typ_common);
            (Expression::Binary(binop.clone(), Box::new(converted_left), Box::new(converted_right)), typ_common)
        },

        BinaryOperator::LessThan |
        BinaryOperator::LessOrEqual |
        BinaryOperator::GreaterThan |
        BinaryOperator::GreaterOrEqual |
        BinaryOperator::Equal |
        BinaryOperator::NotEqual => {
            let typ_left = typex_get_type(&typed_left);
            let typ_right = typex_get_type(&typed_right);
            let typ_common  = get_common_type(&typ_left, &typ_right);
            let converted_left = convert_to(typed_left, &typ_common);
            let converted_right = convert_to(typed_right, &typ_common);
            (Expression::Binary(binop.clone(), Box::new(converted_left), Box::new(converted_right)), Type::Int)
        },


        BinaryOperator::ShiftLeft |
        BinaryOperator::ShiftRight => {
            let typ = typex_get_type(&typed_left);
            (Expression::Binary(binop.clone(), Box::new(typed_left), Box::new(typed_right)), typ)
        },

        BinaryOperator::Assign |
        BinaryOperator::AddAssign |
        BinaryOperator::BitwiseAndAssign |
        BinaryOperator::BitwiseOrAssign |
        BinaryOperator::BitwiseXorAssign |
        BinaryOperator::DivideAssign |
        BinaryOperator::MultiplyAssign |
        BinaryOperator::RemainderAssign |
        BinaryOperator::ShiftLeftAssign |
        BinaryOperator::ShiftRightAssign |
        BinaryOperator::SubtractAssign => {
            let typ_left = typex_get_type(&typed_left);
            let converted_right = convert_to(typed_right, &typ_left);
            (Expression::Binary(binop.clone(), Box::new(typed_left), Box::new(converted_right)), typ_left)
        },

        BinaryOperator::ConditionalMiddle => {
            panic!("Typecheck for ConditionalMiddle");
        }


    };

    Ok(typex_set_type(result_expr, result_typ))
}


fn typecheck_expr_conditional(cond: &TypedExpression, true_expr: &TypedExpression, false_expr: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<TypedExpression, String>
{
    let mut typed_cond = typecheck_expression(cond, symbol_table)?;

    let typ_cond = typex_get_type(&typed_cond);

    typed_cond = convert_to(typed_cond, &Type::Int);

    let mut typed_true = typecheck_expression(true_expr, symbol_table)?;
    let mut typed_false = typecheck_expression(false_expr, symbol_table)?;

    let typ_true = typex_get_type(&typed_true);
    let typ_false = typex_get_type(&typed_false);


    let result_typ = get_common_type(&typ_true, &typ_false);
    typed_true = convert_to(typed_true, &result_typ);
    typed_false = convert_to(typed_false, &result_typ);


    let ret_val = Expression::Conditional(Box::new(typed_cond), Box::new(typed_true), Box::new(typed_false));

    //TODO: need to check that true_expr and false_expr have both the same type

    Ok(typex_set_type(ret_val, result_typ))
}


fn typecheck_expr_inc_dec(typed_expr: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<TypedExpression, String>
{
    let TypedExpression::TypedExp(_, expr) = typed_expr;
    let ret_val = match expr {
        Expression::Var(var_name) => {
            typecheck_expr_var(var_name, symbol_table)?
        },
        _ => { return Err(format!("Non-lvalue for increment/decrement"));
        }
    };


    Ok(ret_val)
}


fn typecheck_expr_unary(unop: &UnaryOperator, typed_expr: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<TypedExpression, String>
{
    let checked_type_expr = typecheck_expression(typed_expr, symbol_table)?;
    let typ = match unop {
        UnaryOperator::LogicalNot => Type::Int,
        _ => typex_get_type(&checked_type_expr)
    };

    Ok(typex_set_type(Expression::Unary(unop.clone(), Box::new(checked_type_expr)), typ))
}


fn typecheck_expression(typed_expr: &TypedExpression, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<TypedExpression, String>
{
    let TypedExpression::TypedExp(_, expr) = typed_expr;
    let result_typed_expr = match expr {
        Expression::FunctionCall(func_name, args) => {
            typecheck_expr_function_call(func_name, args, symbol_table)?
        },
        Expression::Var(var_name) => {
            typecheck_expr_var(var_name, symbol_table)?
        },
        Expression::Assignment(left, right) => {
            typecheck_expr_assignment(&None, left, right, symbol_table)?
        },
        Expression::CompoundAssignment(binop,left , right) => {
            typecheck_expr_assignment(&Some(binop.clone()), left, right, symbol_table)?
        },
        Expression::Binary(binop, left , right) => {
            typecheck_expr_binary(binop, left, right, symbol_table)?
        },
        Expression::Conditional(cond, true_expr, false_expr ) => {
            typecheck_expr_conditional(cond, true_expr, false_expr, symbol_table)?
        },
        Expression::Constant(c) => {
            c.to_typex()
        },
        Expression::PostDecrement(expr) |
        Expression::PostIncrement(expr) |
        Expression::PreDecrement(expr) |
        Expression::PreIncrement(expr) => {
            typecheck_expr_inc_dec(expr, symbol_table)?
        },

        Expression::Unary(unop, expr ) => {
            typecheck_expr_unary(unop, expr, symbol_table)?
        },

        Expression::Cast(typ, expr) => {
            let typed_inner = typecheck_expression(expr, symbol_table)?;
            let checked_cast_expr = Expression::Cast(typ.clone(), Box::new(typed_inner));
            typex_set_type(checked_cast_expr, typ.clone())
        }

    };
    Ok(result_typed_expr)
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
                if old_decl.typ.is_func() {
                    return Err(format!("Function '{}()' is redeclared as variable", var_name));
                }
                if old_decl.typ != *typ {
                    return Err(format!("Conflicting redeclaration of variabale '{}' Initial type: '{:?}', redeclared as '{:?}'", var_name, old_decl.typ, typ));
                }
            }
            else {
                symbol_table.insert(
                    var_name.clone(),
                    SymbolInfo { typ: typ.clone(), attrs: IdentifierAttrs::StaticAttr(InitialValue::NoInitializer, true) }
                );
            }
        },

        Some(StorageClass::Static) => {
            let initial_value = match initializer {
                Some(TypedExpression::TypedExp(_, Expression::Constant(Const::ConstInt(init_val)))) => {
                    StaticInit::IntInit(*init_val)
                },
                Some(TypedExpression::TypedExp(_, Expression::Constant(Const::ConstLong(init_val)))) => {
                    StaticInit::LongInit(*init_val)
                },
                None => {
                    StaticInit::IntInit(0)
                },
                _ => {
                    return Err(format!("Non-constant initializer for local static variable '{}'", var_name));
                }
            };

            let initial_value = InitialValue::Initial(initial_value.convert_to(typ));

            symbol_table.insert(
                var_name.clone(),
                SymbolInfo { typ: typ.clone(), attrs: IdentifierAttrs::StaticAttr(initial_value, false) }
            );
        },

        _ => {
            symbol_table.insert(
                var_name.clone(),
                SymbolInfo{ typ: typ.clone(), attrs: IdentifierAttrs::LocalAttr }
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

        Some(TypedExpression::TypedExp(_, Expression::Constant(Const::ConstInt(init_val)))) => {
            InitialValue::Initial(StaticInit::IntInit(*init_val).convert_to(typ))
        },
        Some(TypedExpression::TypedExp(_, Expression::Constant(Const::ConstLong(init_val)))) => {
            InitialValue::Initial(StaticInit::LongInit(*init_val).convert_to(typ))
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
        if old_decl.typ.is_func() {
            return Err(format!("Function '{}()' redeclared as variable", var_name));
        }

        if old_decl.typ != *typ {
            return Err(format!("Conflicting types for variable '{}' Original type: '{:?}' redeclared as '{:?}'", var_name, old_decl.typ, typ));
        }


        let (old_init, old_global) = match &old_decl.attrs {
            IdentifierAttrs::StaticAttr(old_init_val, old_global ) => {
                (old_init_val, old_global)
            },
            _ => {
                return Err(format!("Wrong attribute for file scope variable '{}' Expected StaticAttrs, got '{:?}'.", var_name, old_decl.attrs));
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
        SymbolInfo { typ: typ.clone(), attrs }
    );

    Ok(())
}


fn type_str(typ: &Type) -> String
{
    match typ {
        Type::Int => String::from("int"),
        Type::Long => String::from("long"),
        Type::FuncType(param_types, ret_type, _) => {
            let mut param_types_str = vec![];
            for param_type in param_types {
                if let Type::FuncType(_,_,_) = param_type {
                    panic!("Function parameter cannot have function type");
                }
                else {
                    param_types_str.push(type_str(param_type));
                }
            }

            let ret_type_str = if let Type::FuncType(_,_,_) = **ret_type {
                panic!("Function return type cannot be function");
            }
            else {
                type_str(ret_type)
            };

            format!("{}({})", ret_type_str, param_types_str.join(", "))
        }
    }
}


fn are_function_types_compatible(func_type_1: &Type, func_type_2: &Type) -> bool
{
    let ret_val = match (func_type_1, func_type_2) {
        (Type::FuncType(param_types_1, ret_type_1, _),
        Type::FuncType(param_types_2, ret_type_2, _)) => {
            (*ret_type_1 == *ret_type_2) &&
            (param_types_1.len() == param_types_2.len()) &&
            *param_types_1 == *param_types_2
        },
        _ => {
            false
        }
    };

    ret_val
}


fn typecheck_function_declaration(func_decl: &mut FunctionDeclaration, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    let FunctionDeclaration::Declarant(func_name, param_names, body, func_type, stg_class ) = func_decl;
    let has_body = body.is_some();
    let mut already_defined = false;

    let mut global = *stg_class != Some(StorageClass::Static);

    if let Some(old_decl) = symbol_table.get(func_name) {
        match &old_decl.typ {
            Type::FuncType(_, _, old_defined) => {
                if !are_function_types_compatible(&old_decl.typ, func_type) {
                    return Err(format!("Incompatible redeclaration of function '{}()' ({} vs. {})", func_name, type_str(&old_decl.typ), type_str(func_type)));
                }

                already_defined = *old_defined;
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
            Type::Int |
            Type::Long => {
                return Err(format!("Function '{}()' redefines variable '{}", func_name, func_name));
            }
        }
    }


    let attrs = IdentifierAttrs::FuncAttr(already_defined || has_body, global);
    let (new_param_types, new_ret_type, new_defined) = match func_type {
        Type::FuncType(param_types, ret_type, _) =>
            (param_types.clone(), ret_type.clone(), already_defined||has_body),
        _ => { panic!("Type should be function"); }
    };

    let new_typ = Type::FuncType(new_param_types.clone(), new_ret_type.clone(), new_defined);
    symbol_table.insert(func_name.clone(), SymbolInfo { typ: new_typ, attrs });

    if has_body {
        for (param_type, param_name) in new_param_types.into_iter().zip(param_names) {
            symbol_table.insert(
                param_name.clone(),
                SymbolInfo { typ: param_type, attrs: IdentifierAttrs::LocalAttr}
            );
        }
        let body = body.as_mut().unwrap();
        typecheck_block(body, &new_ret_type, symbol_table)?;
        label_block_switch_statements(body, &None, &mut HashMap::new(), &mut None)?;
    }

    Ok(())


}


fn typecheck_local_declaration(decl: &mut Declaration, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match decl {
        Declaration::FunDecl(func_decl) => {
            typecheck_function_declaration(func_decl, symbol_table)?;
        },
        Declaration::VarDecl(var_decl) => {
            typecheck_local_variable_declaration(&var_decl, symbol_table)?;
        }
    };

    Ok(())
}


fn  typecheck_unlabeled_statement(unlabeled_statement: &mut UnlabeledStatement, func_ret_type: &Type, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match unlabeled_statement {
        UnlabeledStatement::Break(_,_) |
        UnlabeledStatement::Continue(_) |
        UnlabeledStatement::Goto(_) |
        UnlabeledStatement::Null => {},

        UnlabeledStatement::Expr(expr) => {
            *expr = typecheck_expression(expr, symbol_table)?;
        },

        UnlabeledStatement::Return(expr) => {
            let ret_expr = typecheck_expression(expr, symbol_table)?;
            *expr = convert_to(ret_expr, func_ret_type);
        },

        UnlabeledStatement::If(cond, then_stmnt, else_stmnt) => {
            *cond = typecheck_expression(cond, symbol_table)?;
            typecheck_statement(then_stmnt, func_ret_type, symbol_table)?;
            if let Some(else_stmnt) = else_stmnt {
                typecheck_statement(else_stmnt, func_ret_type, symbol_table)?;
            }
        },

        UnlabeledStatement::While(cond, body, _) |
        UnlabeledStatement::DoWhile(body, cond, _) => {
            *cond = typecheck_expression(cond, symbol_table)?;
            typecheck_statement(body, func_ret_type, symbol_table)?;
        },

        UnlabeledStatement::For(for_init, cond, post, body, _) => {
            match for_init {
                ForInit::InitDecl(var_decl) => {
                    typecheck_local_variable_declaration(var_decl, symbol_table)?;
                },
                ForInit::InitExp(Some(init_expr)) => {
                    *init_expr = typecheck_expression(init_expr, symbol_table)?;
                }
                ForInit::InitExp(None) => {}
            };

            if let Some(cond) = cond {
                *cond = typecheck_expression(cond, symbol_table)?;
            }

            if let Some(post) = post {
                *post = typecheck_expression(post, symbol_table)?;
            }

            typecheck_statement(body, func_ret_type, symbol_table)?;
        },

        UnlabeledStatement::Switch(cond, body, _, _, _) => {
            *cond = typecheck_expression(cond, symbol_table)?;
            typecheck_statement(body, func_ret_type, symbol_table)?;
        },

        UnlabeledStatement::Compound(block) => {
            typecheck_block(block, func_ret_type, symbol_table)?;
        }

    };

    Ok(())
}


fn typecheck_statement(stmnt: &mut Statement, func_ret_type: &Type, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(_, unlabeled_statement ) => {
            typecheck_unlabeled_statement(unlabeled_statement, func_ret_type, symbol_table)?;
        }
    };

    Ok(())
}


fn typecheck_block_item(block_item: &mut BlockItem, func_ret_type: &Type, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            typecheck_local_declaration(decl, symbol_table)?;
        },
        BlockItem::S(stmnt) => {
            typecheck_statement(stmnt, func_ret_type, symbol_table)?;
        }
    };

    Ok(())
}


fn typecheck_block(block: &mut Block, func_ret_type: &Type, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                typecheck_block_item(block_item, func_ret_type, symbol_table)?;
            }

            Ok(())
        }
    }
}


fn typecheck_file_scope_declaration(decl: &mut Declaration, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
{
    match decl {
        Declaration::FunDecl(func_decl) => {
            typecheck_function_declaration(func_decl, symbol_table)?;
        },
        Declaration::VarDecl(var_decl) => {
            typecheck_file_scope_variable_declaration(var_decl, symbol_table)?;
        }
    };

    Ok(())
}


pub fn typecheck_program(prog: &mut Program, symbol_table: &mut HashMap<String, SymbolInfo>) -> Result<(), String>
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
