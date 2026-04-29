use super::ast::*;

fn pretty_print_typed_expression(typed_expr: &TypedExpression, indent: usize)
{
    let TypedExpression::TypedExp(typ, expr ) = typed_expr;
    match expr {
        Expression::Constant(Const::ConstInt(c)) => {
            println!("{}Constant(INT = {})", " ".repeat(indent), c);
        },
        Expression::Constant(Const::ConstLong(c)) => {
            println!("{}Constant(LONG = {})", " ".repeat(indent), c);
        },
        Expression::Var(var_name) => {
            println!("{}Var(TYPE='{}' NAME='{}')", " ".repeat(indent), opt_type_str(typ), var_name);
        },
        Expression::FunctionCall(func_name, args) => {
            pretty_print_function_call(func_name, args, indent);
        }
        Expression::Unary(unary_op, inner_expression) => {
            pretty_print_unary_operator(unary_op, inner_expression, indent);
        },
        Expression::PreIncrement(inner_expression) => {
            pretty_print_unary_operator(&UnaryOperator::PreIncrement, inner_expression, indent)
        },
        Expression::PreDecrement(inner_expression) => {
            pretty_print_unary_operator(&UnaryOperator::PreDecrement, inner_expression, indent)
        },
        Expression::PostIncrement(inner_expression) => {
            pretty_print_unary_operator(&UnaryOperator::PostIncrement, inner_expression, indent)
        },
        Expression::PostDecrement(inner_expression) => {
            pretty_print_unary_operator(&UnaryOperator::PostDecrement, inner_expression, indent)
        },
        Expression::Assignment(left, right) => {
            pretty_print_assignment(left, right, indent);
        },
        Expression::Binary(binary_op, left, right) => {
            pretty_print_binary_operator(binary_op, left, right, indent);
        },
        Expression::CompoundAssignment(binary_op, left, right) => {
            pretty_print_compound_assignment(binary_op, left, right, indent);
        },
        Expression::Conditional(cond, true_exp, false_exp) => {
            pretty_print_conditional(cond, true_exp, false_exp, indent);
        },
        Expression::Cast(typ, expr) => {
            pretty_print_cast(typ, expr, indent);
        }
    }
}


fn pretty_print_function_call(func_name: &String, args: &Vec<TypedExpression>, indent: usize)
{
    print!("{}CALL {}(", " ".repeat(indent), func_name);
    if args.is_empty() {
        println!(")");
    }
    else {
        println!("");
        pretty_print_typed_expression(&args[0], indent + 4);
        for i in 1..args.iter().len() {
            println!("{},", " ".repeat(indent + 4));
            pretty_print_typed_expression(&args[i], indent + 4);
        }
        println!("{})", " ".repeat(indent));
    }
}


fn pretty_print_unary_operator(unary_op: &UnaryOperator, inner_expression: &TypedExpression, indent: usize)
{
    match unary_op {
        UnaryOperator::Plus => {
            println!("{}UnaryPlus(", " ".repeat(indent));
        }
        UnaryOperator::Complement => {
            println!("{}BinaryNot(", " ".repeat(indent));
        },
        UnaryOperator::Negate => {
            println!("{}Minus(", " ".repeat(indent));
        },
        UnaryOperator::LogicalNot => {
            println!("{}LogicalNot(", " ".repeat(indent));
        },
        UnaryOperator::PreIncrement => {
            println!("{}PreIncrement(", " ".repeat(indent));
        },
        UnaryOperator::PreDecrement => {
            println!("{}PreDecrement(", " ".repeat(indent));
        },
        UnaryOperator::PostIncrement => {
            println!("{}PostIncrement(", " ".repeat(indent));
        },
        UnaryOperator::PostDecrement => {
            println!("{}PostDecrement(", " ".repeat(indent));
        }
    }

    pretty_print_typed_expression(inner_expression, indent + 4);
    println!("{})", " ".repeat(indent));
}


fn pretty_print_binary_operator(
    binary_op: &BinaryOperator,
    left: &TypedExpression,
    right: &TypedExpression,
    indent: usize)
{
    match binary_op {
        BinaryOperator::Add => {
            println!("{}Add(", " ".repeat(indent));
        },
        BinaryOperator::Subtract => {
            println!("{}Subtract(", " ".repeat(indent));
        },
        BinaryOperator::Multiply => {
            println!("{}Multiply(", " ".repeat(indent));
        },
        BinaryOperator::Divide => {
            println!("{}Divide(", " ".repeat(indent));
        },
        BinaryOperator::Remainder => {
            println!("{}Remainder(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseAnd => {
            println!("{}BitwiseAnd(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseOr => {
            println!("{}BitwiseOr(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseXor => {
            println!("{}BitwiseXor(", " ".repeat(indent));
        },
        BinaryOperator::ShiftLeft => {
            println!("{}ShiftLeft(", " ".repeat(indent));
        },
        BinaryOperator::ShiftRight => {
            println!("{}ShiftRight(", " ".repeat(indent));
        },
        BinaryOperator::LogicalOr => {
            println!("{}LogicalOr(", " ".repeat(indent));
        },
        BinaryOperator::LogicalAnd => {
            println!("{}LogicalAnd(", " ".repeat(indent));
        },
        BinaryOperator::Equal => {
            println!("{}Equal(", " ".repeat(indent));
        },
        BinaryOperator::NotEqual => {
            println!("{}NotEqual(", " ".repeat(indent));
        },
        BinaryOperator::LessThan => {
            println!("{}LessThan(", " ".repeat(indent));
        },
        BinaryOperator::LessOrEqual => {
            println!("{}LessOrEqual(", " ".repeat(indent));
        },
        BinaryOperator::GreaterThan => {
            println!("{}GreaterThan(", " ".repeat(indent));
        },
        BinaryOperator::GreaterOrEqual => {
            println!("{}GreaterOrEqual(", " ".repeat(indent));
        },
        BinaryOperator::Assign => {
            println!("{}Assign(", " ".repeat(indent));
        },
        BinaryOperator::AddAssign => {
            println!("{}AddAssign(", " ".repeat(indent));
        },
        BinaryOperator::SubtractAssign => {
            println!("{}SubtractAssign(", " ".repeat(indent));
        },
        BinaryOperator::MultiplyAssign => {
            println!("{}MultiplyAssign(", " ".repeat(indent));
        },
        BinaryOperator::DivideAssign => {
            println!("{}DivideAssign(", " ".repeat(indent));
        },
        BinaryOperator::RemainderAssign => {
            println!("{}RemainderAssign(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseAndAssign => {
            println!("{}BitwiseAndAssign(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseOrAssign => {
            println!("{}BitwiseOrAssign(", " ".repeat(indent));
        },
        BinaryOperator::BitwiseXorAssign => {
            println!("{}BitwiseXorAssign(", " ".repeat(indent));
        },
        BinaryOperator::ShiftLeftAssign => {
            println!("{}ShiftLeftAssign(", " ".repeat(indent));
        },
        BinaryOperator::ShiftRightAssign => {
            println!("{}ShiftRightAssign(", " ".repeat(indent));
        },
        BinaryOperator::ConditionalMiddle => {
            panic!("ConditionalMiddle shall NOT appear in the AST!");
        }
    };

    pretty_print_typed_expression(left, indent + 4);
    println!("{},", " ".repeat(indent + 4));
    pretty_print_typed_expression(right, indent + 4);
    println!("{})", " ".repeat(indent));

}

fn pretty_print_assignment(left: &TypedExpression, right: &TypedExpression, indent: usize)
{
    pretty_print_binary_operator(&BinaryOperator::Assign, left, right, indent);
}

fn pretty_print_compound_assignment(binary_op: &BinaryOperator, left: &TypedExpression, right: &TypedExpression, indent: usize)
{
    pretty_print_binary_operator(binary_op, left, right, indent);
}

fn pretty_print_conditional(cond: &TypedExpression, true_exp: &TypedExpression, false_exp: &TypedExpression, indent: usize)
{
    println!("{}Conditional(", " ".repeat(indent));
    println!("{}Cond=(", " ".repeat(indent + 4));
    pretty_print_typed_expression(cond, indent + 8);
    println!("{})", " ".repeat(indent + 4));
    println!("{}True_exp=(", " ".repeat(indent + 4));
    pretty_print_typed_expression(true_exp, indent + 8);
    println!("{})", " ".repeat(indent + 4));
    println!("{}False_exp=(", " ".repeat(indent + 4));
    pretty_print_typed_expression(false_exp, indent + 8);
    println!("{})", " ".repeat(indent + 4));
    println!("{})", " ".repeat(indent));
}


fn pretty_print_cast(typ: &Type, expr: &TypedExpression, indent: usize) {
    println!("{}CastTo({},", " ".repeat(indent), type_str(typ));
    pretty_print_typed_expression(expr, indent + 4);
    println!("{})", " ".repeat(indent));
}


fn pretty_print_labels(labels: &Vec<Label>, indent: usize)
{
    for label in labels {
        match label {
            Label::Goto(goto_label) => {
                println!("{}{}:", " ".repeat(indent), goto_label);
            },
            Label::Case(case_expr) => {
                println!("{}CASE (", " ".repeat(indent));
                pretty_print_typed_expression(case_expr, indent + 4);
                println!("{}): ", " ".repeat(indent));
            },
            Label::ResolvedCase(res_case_label) => {
                println!("{}{}:", " ".repeat(indent), res_case_label);
            }
            Label::Default => {
                println!("{}DEFAULT:", " ".repeat(indent));
            },
        }
    }
}

fn pretty_print_for_init(for_init: &ForInit, indent: usize)
{
    match for_init {
        ForInit::InitExp(None) => (),
        ForInit::InitExp(Some(expr)) => {
            pretty_print_typed_expression(expr, indent);
        },
        ForInit::InitDecl(var_decl) => {
            pretty_print_variable_declaration(var_decl, indent);
        }
    }
}

fn pretty_print_statement(s: &Statement, indent: usize)
{

    let (labels, unlabeled_stmnt) = match s {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            (&vec![], unlabeled_stmnt)
        },
        Statement::Stmnt(Some(labels), unlabeled_stmnt ) => {
            (labels, unlabeled_stmnt)
        }
    };

    pretty_print_labels(labels, 0);
    pretty_print_unlabeled_statement(unlabeled_stmnt, indent);
}


fn pretty_print_unlabeled_statement(s: &UnlabeledStatement, indent: usize)
{
    match s {
        UnlabeledStatement::Return(expr) => {
            println!("{}Return(", " ".repeat(indent));
            pretty_print_typed_expression(&expr, indent + 4);
            println!("{})", " ".repeat(indent));
        },
        UnlabeledStatement::Goto(label) => {
            println!("{}Goto {}", " ".repeat(indent), label);
        },
        UnlabeledStatement::If(cond,then_stmnt , else_stmnt) => {
            println!("{}If(", " ".repeat(indent));
            println!("{}Cond=(", " ".repeat(indent + 4));
            pretty_print_typed_expression(cond, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{}Then(", " ".repeat(indent + 4));
            pretty_print_statement(then_stmnt, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            if let Some(else_stmnt) = else_stmnt {
                println!("{}Else(", " ".repeat(indent + 4));
                pretty_print_statement(else_stmnt, indent + 8);
                println!("{})", " ".repeat(indent + 4));
            }
            println!("{})", " ".repeat(indent));
        },
        UnlabeledStatement::Compound(block) => {
            pretty_print_block(block, indent);
        },
        UnlabeledStatement::Break(break_type, label) => {
            println!("{}Break(type={:?}, label='{}')", " ".repeat(indent), break_type, label.clone().unwrap_or_default());
        },
        UnlabeledStatement::Continue(loop_label) => {
            println!("{}Continue(loop_label='{}')", " ".repeat(indent), loop_label.clone().unwrap_or_default());
        },
        UnlabeledStatement::While(cond, body, loop_label) => {
            println!("{}While(", " ".repeat(indent));
            println!("{}Label='{}'", " ".repeat(indent + 4), loop_label.clone().unwrap_or_default());
            println!("{}Cond=(", " ".repeat(indent + 4));
            pretty_print_typed_expression(cond, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{}Body=(", " ".repeat(indent + 4));
            pretty_print_statement(body, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{})", " ".repeat(indent));
        },
        UnlabeledStatement::DoWhile(body, cond, loop_label) => {
            println!("{}Do(", " ".repeat(indent));
            println!("{}Label='{}'", " ".repeat(indent + 4), loop_label.clone().unwrap_or_default());
            println!("{}Body=(", " ".repeat(indent + 4));
            pretty_print_statement(body, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{}While(", " ".repeat(indent + 4));
            println!("{}Cond=(", " ".repeat(indent + 8));
            pretty_print_typed_expression(cond, indent + 12);
            println!("{})", " ".repeat(indent + 8));
            println!("{})", " ".repeat(indent + 4));
            println!("{})", " ".repeat(indent));
        },
        UnlabeledStatement::For(for_init, cond, post, body, loop_label) => {
            println!("{}For(", " ".repeat(indent));
            println!("{}Label='{}'", " ".repeat(indent + 4), loop_label.clone().unwrap_or_default());
            println!("{}Init=(", " ".repeat(indent + 4));
            pretty_print_for_init(for_init, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{}Cond=(", " ".repeat(indent + 4));
            if let Some(expr)  = cond {
                pretty_print_typed_expression(expr, indent + 8);
            }
            println!("{})", " ".repeat(indent + 4));
            println!("{}Post=(", " ".repeat(indent + 4));
            if let Some(expr) = post {
                pretty_print_typed_expression(expr, indent + 8);
            }
            println!("{})", " ".repeat(indent + 4));
            println!("{}Body=(", " ".repeat(indent + 4));
            pretty_print_statement(body, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{})", " ".repeat(indent));
        },
        UnlabeledStatement::Switch(cond, body, switch_label, case_label_map, default_label) => {
            println!("{}Switch(", " ".repeat(indent));
            println!("{}Cond=(", " ".repeat(indent + 4));
            pretty_print_typed_expression(cond, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{}Label='{}'", " ".repeat(indent + 4), switch_label.clone().unwrap_or_default());
            if !case_label_map.is_empty() || default_label.is_some() {
                println!("{}Case Labels=(", " ".repeat(indent + 4));
                for (case_const, case_label) in case_label_map {
                    println!("{}CASE {} --> {}", " ".repeat(indent + 8), case_const.to_i64(), case_label);
                }

                if default_label.is_some() {
                    println!("{}DEFAULT --> {}", " ".repeat(indent + 8), default_label.clone().unwrap());
                }

                println!("{})", " ".repeat(indent + 4));
            }
            else {
                println!("{}NO Case Labels", " ".repeat(indent + 4));
            }
            println!("{}Body=(", " ".repeat(indent + 4));
            pretty_print_statement(body, indent + 8);
            println!("{})", " ".repeat(indent + 4));
            println!("{})", " ".repeat(indent));
        },
        UnlabeledStatement::Expr(expr) => {
            println!("{}Expr(", " ".repeat(indent));
            pretty_print_typed_expression(expr, indent + 4);
            println!("{})", " ".repeat(indent));
        },
        UnlabeledStatement::Null => {
            println!("{}NOP()", " ".repeat(indent));
        }
    }
}


fn type_str(typ: &Type) -> String
{
    return typ.to_string();
}

fn opt_type_str(opt_typ: &Option<Type>) -> String
{
    match opt_typ {
        None => String::from("NONE"),
        Some(typ) => type_str(typ)
    }
}

fn storage_class_str(stg_class: &Option<StorageClass>) -> &str
{
    match stg_class {
        None => "",
        Some(StorageClass::Extern) => "EXTERN",
        Some(StorageClass::Static) => "STATIC"
    }
}

fn pretty_print_variable_declaration(decl: &VariableDeclaration, indent: usize)
{
    match decl {
        VariableDeclaration::Declarant(var_name, Some(expr_init), typ, stg_class) => {
            println!("{}{}{} Var {} = (", " ".repeat(indent), storage_class_str(stg_class), type_str(typ), var_name);
            pretty_print_typed_expression(expr_init, indent + 4);
            println!("{})", " ".repeat(indent));
        },
        VariableDeclaration::Declarant(var_name,None, typ, stg_class ) => {
            println!("{}{}{} var {}", " ".repeat(indent),  storage_class_str(stg_class), type_str(typ), var_name);
        }
    }
}



fn pretty_print_function_declaration(func_decl: &FunctionDeclaration, indent: usize)
{
    match func_decl {
        FunctionDeclaration::Declarant(func_name, param_list, body, typ, stg_class) => {
            println!("{}{} Function(", " ".repeat(indent), storage_class_str(stg_class));
            println!("{}name={func_name}", " ".repeat(indent + 4));
            println!("{}type={}", " ".repeat(indent + 4), type_str(typ));
            println!("{}params=(", " ".repeat(indent + 4));
            if !param_list.is_empty() {
                print!("{}{}", " ".repeat(indent + 8), param_list[0]);
                for i in 1..param_list.len() {
                    println!(",");
                    print!("{}{}", " ".repeat(indent + 8), param_list[i]);
                }
                println!("");
            }
            println!("{})", " ".repeat(indent + 4));
            if let Some(body) = body {
                println!("{}body=(", " ".repeat(indent + 4));
                pretty_print_block(body, indent + 8);
                println!("{})", " ".repeat(indent + 4));
            }
            else {
                println!("{}NO BODY (DECLARATION ONLY)", " ".repeat(indent + 4));
            }
            println!("{})", " ".repeat(indent));
        }
    }
}


fn pretty_print_declaration(decl: &Declaration, indent: usize)
{
    match decl {
        Declaration::VarDecl(var_decl) => pretty_print_variable_declaration(var_decl, indent),
        Declaration::FunDecl(func_decl) => pretty_print_function_declaration(func_decl, indent)
    }
}


fn pretty_print_block_item(block_item: &BlockItem, indent: usize)
{
    match block_item {
        BlockItem::D(decl) => {
            pretty_print_declaration(decl, indent);
        },
        BlockItem::S(stmnt) => {
            pretty_print_statement(stmnt, indent);
        }
    }
}

fn pretty_print_block(b: &Block, indent: usize)
{
    println!("{}block=(", " ".repeat(indent));
    match b {
        Block::Blk(block_items) => {
            for block_item in block_items {
                pretty_print_block_item(block_item, indent + 4);
            }
        }
    };
    println!("{})", " ".repeat(indent));
}



fn pretty_print_program(p: &Program, indent: usize)
{
    println!("{}Program(", " ".repeat(indent));
    match p {
        Program::ProgramDefinition(decls) => {
            for decl in decls {
                pretty_print_declaration(&decl, indent + 4);
                println!("");
            }
        }
        _ => ()
    };

    println!("{})", " ".repeat(indent));
}


pub fn pretty_print_ast(p: &Program)
{
    pretty_print_program(p, 0);
}
