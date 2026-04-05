use std::{sync::atomic::{AtomicUsize, Ordering}};

pub mod ast;
mod pretty_print;

use ast::*;

use super::parser;


static TMP_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);

static TMP_LABEL_INDEX: AtomicUsize = AtomicUsize::new(0);

fn make_temp_name() -> String
{
    let index = TMP_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let temp_name = format!("tmp.{}", index);

    temp_name
}

fn make_temp_label(prefix: &str) -> String
{
    let index = TMP_LABEL_INDEX.fetch_add(1, Ordering::SeqCst);
    let label = format!("l_{}_{}", prefix, index);

    label
}


fn emit_tacky_unary_operator(unop: &parser::ast::UnaryOperator) -> Result<UnaryOperator, String>
{
    match unop {
        parser::ast::UnaryOperator::Plus => Ok(UnaryOperator::Plus),
        parser::ast::UnaryOperator::Complement => Ok(UnaryOperator::Complement),
        parser::ast::UnaryOperator::Negate => Ok(UnaryOperator::Negate),
        parser::ast::UnaryOperator::LogicalNot => Ok(UnaryOperator::LogicalNot),
        _ => { return Err(format!("TACKY Conversion: Expected unary operator, got '{:?}'", unop)); }
    }
}


fn emit_tacky_binary_operator(binop: &parser::ast::BinaryOperator) -> Result<BinaryOperator, String>
{
    match binop {
        parser::ast::BinaryOperator::Add            => Ok(BinaryOperator::Add),
        parser::ast::BinaryOperator::Subtract       => Ok(BinaryOperator::Subtract),
        parser::ast::BinaryOperator::Multiply       => Ok(BinaryOperator::Multiply),
        parser::ast::BinaryOperator::Divide         => Ok(BinaryOperator::Divide),
        parser::ast::BinaryOperator::Remainder      => Ok(BinaryOperator::Remainder),
        parser::ast::BinaryOperator::BitwiseOr      => Ok(BinaryOperator::BitwiseOr),
        parser::ast::BinaryOperator::BitwiseAnd     => Ok(BinaryOperator::BitwiseAnd),
        parser::ast::BinaryOperator::BitwiseXor     => Ok(BinaryOperator::BitwiseXor),
        parser::ast::BinaryOperator::ShiftLeft      => Ok(BinaryOperator::ShiftLeft),
        parser::ast::BinaryOperator::ShiftRight     => Ok(BinaryOperator::ShiftRight),
        parser::ast::BinaryOperator::Equal          => Ok(BinaryOperator::Equal),
        parser::ast::BinaryOperator::NotEqual       => Ok(BinaryOperator::NotEqual),
        parser::ast::BinaryOperator::LessThan       => Ok(BinaryOperator::LessThan),
        parser::ast::BinaryOperator::LessOrEqual    => Ok(BinaryOperator::LessOrEqual),
        parser::ast::BinaryOperator::GreaterThan    => Ok(BinaryOperator::GreaterThan),
        parser::ast::BinaryOperator::GreaterOrEqual => Ok(BinaryOperator::GreaterOrEqual),

        _ => { return Err(format!("TACKY Conversion: Expected binary operator, got '{:?}'", binop)); }
    }
}

fn get_noncompound_operator(compond_binary_operator: &parser::ast::BinaryOperator) -> Result<parser::ast::BinaryOperator, String>
{
    let noncompund_binop = match compond_binary_operator {
        parser::ast::BinaryOperator::AddAssign => parser::ast::BinaryOperator::Add,
        parser::ast::BinaryOperator::SubtractAssign => parser::ast::BinaryOperator::Subtract,
        parser::ast::BinaryOperator::MultiplyAssign => parser::ast::BinaryOperator::Multiply,
        parser::ast::BinaryOperator::DivideAssign => parser::ast::BinaryOperator::Divide,
        parser::ast::BinaryOperator::RemainderAssign => parser::ast::BinaryOperator::Remainder,
        parser::ast::BinaryOperator::BitwiseAndAssign => parser::ast::BinaryOperator::BitwiseAnd,
        parser::ast::BinaryOperator::BitwiseOrAssign => parser::ast::BinaryOperator::BitwiseOr,
        parser::ast::BinaryOperator::BitwiseXorAssign => parser::ast::BinaryOperator::BitwiseXor,
        parser::ast::BinaryOperator::ShiftLeftAssign => parser::ast::BinaryOperator::ShiftLeft,
        parser::ast::BinaryOperator::ShiftRightAssign => parser::ast::BinaryOperator::ShiftRight,
        _ => { return Err(format!("Expected compound assignment operator. got: '{:?}'", compond_binary_operator)); }
    };

    Ok(noncompund_binop)
}


fn get_inc_dec_operator(expr: &parser::ast::Expression) -> Result<BinaryOperator, String>
{
    let bin_op = match expr {
        parser::ast::Expression::PostIncrement(_) |
        parser::ast::Expression::PreIncrement(_) => BinaryOperator::Add,

        parser::ast::Expression::PostDecrement(_) |
        parser::ast::Expression::PreDecrement(_) => BinaryOperator::Subtract,

        _ => { return Err(format!("Expected ++ or --, got {:?}", expr)); }
    };

    Ok(bin_op)
}

fn emit_tacky_pre_inc_dec(bin_op: BinaryOperator, var_name: &String, instructions: &mut Vec<Instruction>) -> Result<Val, String>
{
    let dst = Val::Var(var_name.clone());
    let src = Val::IntConstant(1);
    instructions.push(Instruction::Binary(bin_op, dst.clone(), src, dst.clone()));

    Ok(dst)
}


fn emit_tacky_post_inc_dec(bin_op: BinaryOperator, var_name: &String, instructions: &mut Vec<Instruction>) -> Result<Val, String>
{
    let dst = Val::Var(var_name.clone());
    let new_dst_name = make_temp_name();
    let new_dst = Val::Var(new_dst_name);
    instructions.push(Instruction::Copy(dst.clone(), new_dst.clone()));

    let src = Val::IntConstant(1);
    instructions.push(Instruction::Binary(bin_op, dst.clone(), src, dst.clone()));

    Ok(new_dst)
}


fn emit_tacky_expression(expr: &parser::ast::Expression, instructions: &mut Vec<Instruction>) -> Result<Val, String>
{
    let val = match expr {
        parser::ast::Expression::IntConstant(c) => {
            Val::IntConstant(*c)
        },
        parser::ast::Expression::Var(var_name) => {
            Val::Var(var_name.clone())
        },
        parser::ast::Expression::Assignment(dst, src ) => {
            match &**dst {
                parser::ast::Expression::Var(var_name) => {
                    let tacky_src = emit_tacky_expression(&*src, instructions)?;
                    instructions.push(Instruction::Copy(tacky_src, Val::Var(var_name.clone())));
                    Val::Var(var_name.clone())
                },
                _ => { return Err(format!("Tacky: non-lvalue on the left of '='")); }
            }
        },
        parser::ast::Expression::CompoundAssignment(binary_operator, dst, src ) => {
            match &**dst {
                parser::ast::Expression::Var(var_name) => {
                    let tacky_src = emit_tacky_expression(&*src, instructions)?;
                    let tacky_dst = Val::Var(var_name.clone());
                    let noncompound_operator = get_noncompound_operator(binary_operator)?;
                    let tmp_dst_name = make_temp_name();
                    let tmp_dst = Val::Var(tmp_dst_name);
                    let tacky_bin_op = emit_tacky_binary_operator(&noncompound_operator)?;
                    instructions.push(Instruction::Binary(tacky_bin_op, tacky_dst.clone(), tacky_src, tmp_dst.clone()));
                    instructions.push(Instruction::Copy(tmp_dst, tacky_dst.clone()));

                    tacky_dst
                },
                _ => { return Err(format!("Tacky: non-lvalue on the left of '{:?}='", binary_operator)); }
            }
        },
        parser::ast::Expression::PreIncrement(inner_expression) |
        parser::ast::Expression::PreDecrement(inner_expression) => {
            let inc_dec_binary_op = get_inc_dec_operator(expr)?;
            let dst = emit_tacky_expression(inner_expression, instructions)?;
            match &dst {
                Val::Var(var_name) => {
                    emit_tacky_pre_inc_dec(inc_dec_binary_op, var_name, instructions)?;
                },
                _ => {
                    { return Err(format!("Tacky: non-lvalue argument for pre-increment/pre-decrement")); }
                }
            };

            dst
        },
        parser::ast::Expression::PostIncrement(inner_expression) |
        parser::ast::Expression::PostDecrement(inner_expression) => {
            let inc_dec_binary_op = get_inc_dec_operator(expr)?;
            let dst = emit_tacky_expression(inner_expression, instructions)?;
            let new_dst = match &dst {
                Val::Var(var_name) => {
                    emit_tacky_post_inc_dec(inc_dec_binary_op, var_name, instructions)?
                },
                _ => {
                    { return Err(format!("Tacky: non-lvalue argument for pre-increment/pre-decrement")); }
                }
            };

            new_dst
        },
        parser::ast::Expression::Unary(unop, inner_expression) => {
            let src = emit_tacky_expression(&inner_expression, instructions)?;
            let dst_name = make_temp_name();
            let dst = Val::Var(dst_name);
            let tacky_un_op = emit_tacky_unary_operator(unop)?;
            instructions.push(Instruction::Unary(tacky_un_op, src, dst.clone()));

            dst
        },
        parser::ast::Expression::Binary(parser::ast::BinaryOperator::LogicalAnd, expr1, expr2) => {
            let result_name = make_temp_name();
            let result = Val::Var(result_name);
            let lbl_expr_is_false = make_temp_label("and_expr_false");
            let lbl_expr_end = make_temp_label("and_expr_end");

            let left = emit_tacky_expression(expr1, instructions)?;
            instructions.push(Instruction::JumpIfZero(left.clone(), lbl_expr_is_false.clone()));
            let right = emit_tacky_expression(expr2, instructions)?;
            instructions.push(Instruction::JumpIfZero(right.clone(), lbl_expr_is_false.clone()));
            instructions.push(Instruction::Copy(Val::IntConstant(1), result.clone()));
            instructions.push(Instruction::Jump(lbl_expr_end.clone()));
            instructions.push(Instruction::Label(lbl_expr_is_false.clone()));
            instructions.push(Instruction::Copy(Val::IntConstant(0), result.clone()));
            instructions.push(Instruction::Label(lbl_expr_end.clone()));

            result
        },
        parser::ast::Expression::Binary(parser::ast::BinaryOperator::LogicalOr, expr1, expr2) => {
            let result_name = make_temp_name();
            let result = Val::Var(result_name);
            let lbl_expr_is_true = make_temp_label("or_expr_true");
            let lbl_expr_end = make_temp_label("or_expr_end");

            let left = emit_tacky_expression(expr1, instructions)?;
            instructions.push(Instruction::JumpIfNotZero(left.clone(), lbl_expr_is_true.clone()));
            let right = emit_tacky_expression(expr2, instructions)?;
            instructions.push(Instruction::JumpIfNotZero(right.clone(), lbl_expr_is_true.clone()));
            instructions.push(Instruction::Copy(Val::IntConstant(0), result.clone()));
            instructions.push(Instruction::Jump(lbl_expr_end.clone()));
            instructions.push(Instruction::Label(lbl_expr_is_true.clone()));
            instructions.push(Instruction::Copy(Val::IntConstant(1), result.clone()));
            instructions.push(Instruction::Label(lbl_expr_end.clone()));

            result
        },
        parser::ast::Expression::Binary(binop, expr1, expr2 ) => {
            let src1 = emit_tacky_expression(expr1, instructions)?;
            let src2 = emit_tacky_expression(expr2, instructions)?;
            let dst_name = make_temp_name();
            let dst = Val::Var(dst_name);
            let tacky_bin_op = emit_tacky_binary_operator(binop)?;
            instructions.push(Instruction::Binary(tacky_bin_op, src1, src2, dst.clone()));

            dst
        },
        parser::ast::Expression::Conditional(cond, true_exp, false_exp) => {
            let result_name = make_temp_name();
            let result = Val::Var(result_name);
            let cond_val = emit_tacky_expression(cond, instructions)?;
            let lbl_cond_zero = make_temp_label("l_cond_zero");
            let lbl_cond_end = make_temp_label("l_cond_end");
            instructions.push(Instruction::JumpIfZero(cond_val, lbl_cond_zero.clone()));
            let true_val = emit_tacky_expression(true_exp, instructions)?;
            instructions.push(Instruction::Copy(true_val, result.clone()));
            instructions.push(Instruction::Jump(lbl_cond_end.clone()));
            instructions.push(Instruction::Label(lbl_cond_zero));
            let false_val = emit_tacky_expression(false_exp, instructions)?;
            instructions.push(Instruction::Copy(false_val, result.clone()));
            instructions.push(Instruction::Label(lbl_cond_end));

            result
        },
        parser::ast::Expression::FunctionCall(func_name, args ) => {
            let mut tacky_args = vec![];

            for arg in args {
                let tacky_arg = emit_tacky_expression(arg, instructions)?;
                tacky_args.push(tacky_arg);
            }

            let ret_val_name = make_temp_name();
            let ret_val = Val::Var(ret_val_name);

            instructions.push(Instruction::FuncCall(func_name.clone(), tacky_args, ret_val.clone()));

            ret_val
        }
        //,
        //  _ => { panic!("TACKY Conversion: unsupported/unimplemented expression, got '{:?}'", expr); }
    };

    Ok(val)
}


fn emit_tacky_statement(stmnt: &parser::ast::Statement, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    match stmnt {
        parser::ast::Statement::Stmnt(None, unlabeled_stmnt) => {
            emit_tacky_unlabeled_statement(unlabeled_stmnt, instructions)?;
        },
        parser::ast::Statement::Stmnt(Some(labels), unlabeled_stmnt) => {
            for label in labels {
                match label {
                    parser::ast::Label::Goto(goto_label) => {
                        instructions.push(Instruction::Label(goto_label.clone()));
                    },
                    parser::ast::Label::ResolvedCase(resolved_case_label) => {
                        instructions.push(Instruction::Label(resolved_case_label.clone()));
                    },
                    parser::ast::Label::Case(_case_const) => {
                        panic!("Tacky generation: case labels not implemented");
                    },
                    parser::ast::Label::Default => {
                        panic!("Tacky generation: case default label not implemented");
                    }
                }

            }
            emit_tacky_unlabeled_statement(unlabeled_stmnt, instructions)?;
        }
    };
    Ok(())
}


fn emit_tacky_for_init(for_init: &parser::ast::ForInit, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    match for_init {
        parser::ast::ForInit::InitDecl(decl) => {
            emit_tacky_variable_declaration(decl, instructions)?;
        },
        parser::ast::ForInit::InitExp(Some(expr)) => {
            emit_tacky_expression(expr, instructions)?;
        },
        parser::ast::ForInit::InitExp(None) => {}
    };

    Ok(())
}


fn start_loop_label(loop_label: &Option<String>) -> String
{
    return format!("{}_start", loop_label.clone().unwrap());
}


fn continue_loop_label(loop_label: &Option<String>) -> String
{
    return format!("{}_continue", loop_label.clone().unwrap());
}

fn break_loop_label(loop_label: &Option<String>) -> String
{
    return format!("{}_break", loop_label.clone().unwrap());
}


fn break_switch_label(switch_label: &Option<String>) -> String
{
    return format!("{}_break", switch_label.clone().unwrap());
}


fn emit_tacky_unlabeled_statement(stmnt: &parser::ast::UnlabeledStatement, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    match stmnt {
        parser::ast::UnlabeledStatement::Return(expr) => {
            let val = emit_tacky_expression(&expr, instructions)?;
            instructions.push(Instruction::Return(val));
        },
        parser::ast::UnlabeledStatement::Goto(label) => {
            instructions.push(Instruction::Jump(label.clone()));
        }
        parser::ast::UnlabeledStatement::If(cond, then_stmnt, None) => {
            //if without else
            let cond_val = emit_tacky_expression(cond, instructions)?;
            let lbl_zero = make_temp_label("l_zero");
            instructions.push(Instruction::JumpIfZero(cond_val, lbl_zero.clone()));
            emit_tacky_statement(then_stmnt, instructions)?;
            instructions.push(Instruction::Label(lbl_zero));
        },
        parser::ast::UnlabeledStatement::If(cond, then_stmnt, Some(else_stmnt)) => {
            //if with else
            let cond_val = emit_tacky_expression(cond, instructions)?;
            let lbl_zero = make_temp_label("l_zero");
            let lbl_end = make_temp_label("l_end");
            instructions.push(Instruction::JumpIfZero(cond_val, lbl_zero.clone()));
            emit_tacky_statement(then_stmnt, instructions)?;
            instructions.push(Instruction::Jump(lbl_end.clone()));
            instructions.push(Instruction::Label(lbl_zero));
            emit_tacky_statement(else_stmnt, instructions)?;
            instructions.push(Instruction::Label(lbl_end));
        },
        parser::ast::UnlabeledStatement::Break(break_type, break_label) => {
            assert!(break_type.is_some());
            match break_type {
                Some(parser::ast::BreakType::Loop) => {
                    instructions.push(Instruction::Jump(break_loop_label(break_label)));
                },
                Some(parser::ast::BreakType::Switch) => {
                    instructions.push(Instruction::Jump(break_switch_label(break_label)));
                },
                None => {
                    panic!("Bug: Got break with None type during tacky generation");
                }
            };
        },
        parser::ast::UnlabeledStatement::Continue(loop_label) => {
            instructions.push(Instruction::Jump(continue_loop_label(loop_label)));
        },
        parser::ast::UnlabeledStatement::While(cond, body, loop_label) => {
            instructions.push(Instruction::Label(continue_loop_label(loop_label)));
            let cond_val = emit_tacky_expression(cond, instructions)?;
            instructions.push(Instruction::JumpIfZero(cond_val, break_loop_label(loop_label)));
            emit_tacky_statement(body, instructions)?;
            instructions.push(Instruction::Jump(continue_loop_label(loop_label)));
            instructions.push(Instruction::Label(break_loop_label(loop_label)));
        },
        parser::ast::UnlabeledStatement::DoWhile(body, cond, loop_label) => {
            instructions.push(Instruction::Label(start_loop_label(loop_label)));
            emit_tacky_statement(body, instructions)?;
            instructions.push(Instruction::Label(continue_loop_label(loop_label)));
            let cond_val = emit_tacky_expression(cond, instructions)?;
            instructions.push(Instruction::JumpIfNotZero(cond_val, start_loop_label(loop_label)));
            instructions.push(Instruction::Label(break_loop_label(loop_label)));
        },
        parser::ast::UnlabeledStatement::For(for_init, cond , post, body, loop_label) => {
            emit_tacky_for_init(for_init, instructions)?;
            instructions.push(Instruction::Label(start_loop_label(loop_label)));
            if let Some(cond) = cond {
                let cond_val = emit_tacky_expression(cond, instructions)?;
                instructions.push(Instruction::JumpIfZero(cond_val, break_loop_label(loop_label)));
            }
            emit_tacky_statement(body, instructions)?;
            instructions.push(Instruction::Label(continue_loop_label(loop_label)));
            if let Some(post) = post {
                emit_tacky_expression(post, instructions)?;
            }
            instructions.push(Instruction::Jump(start_loop_label(loop_label)));
            instructions.push(Instruction::Label(break_loop_label(loop_label)));
        },
        parser::ast::UnlabeledStatement::Switch(cond, body, switch_label , case_labels_map, default_label) => {
            let val = emit_tacky_expression(cond, instructions)?;
            let cmp_result_name = make_temp_name();
            let cmp_result = Val::Var(cmp_result_name);
            let switch_end_label = break_switch_label(switch_label);

            for (case_val, target) in case_labels_map {
                instructions.push(Instruction::Binary(BinaryOperator::Equal, val.clone(), Val::IntConstant(*case_val), cmp_result.clone()));
                instructions.push(Instruction::JumpIfNotZero(cmp_result.clone(), target.clone()));
            }

            // If there is no match, jump to the default label if there is one
            // or after the body if there is no default label
            if let Some(default_label) = default_label {
                instructions.push(Instruction::Jump(default_label.clone()));
            }
            else {
                instructions.push(Instruction::Jump(switch_end_label.clone()));
            }

            emit_tacky_statement(body, instructions)?;
            instructions.push(Instruction::Label(switch_end_label.clone()));
        }

        parser::ast::UnlabeledStatement::Compound(block) => {
            emit_tacky_block(block, instructions)?;
        },
        parser::ast::UnlabeledStatement::Expr(expr) => {
            emit_tacky_expression(expr, instructions)?;
        },
        parser::ast::UnlabeledStatement::Null => {},
        _ => { panic!("emit_tacky_statement: Not implemented for '{:?}' !", stmnt); }
    }

    Ok(())
}


fn emit_tacky_variable_declaration(decl: &parser::ast::VariableDeclaration, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    match decl {
        parser::ast::VariableDeclaration::Declarant(var_name, Some(init_expr) ) => {
            let init_val = emit_tacky_expression(init_expr, instructions)?;
            instructions.push(Instruction::Copy(init_val, Val::Var(var_name.clone())));
        },
        parser::ast::VariableDeclaration::Declarant(_, None) => {}
    };

    Ok(())
}


fn emit_tacky_block_item(block_item: &parser::ast::BlockItem, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    match block_item {
        parser::ast::BlockItem::S(stmnt) => {
            emit_tacky_statement(stmnt, instructions)?;
        },
        parser::ast::BlockItem::D(decl) => {
            match decl {
                parser::ast::Declaration::VarDecl(var_decl) => {
                    emit_tacky_variable_declaration(var_decl, instructions)?;
                },
                parser::ast::Declaration::FunDecl(parser::ast::FunctionDeclaration::Declarant(_ , _, None)) => {
                    /* Nothing to emit */
                },
                parser::ast::Declaration::FunDecl(parser::ast::FunctionDeclaration::Declarant(_ , _, Some(_))) => {
                    panic!("BUG: Local function definitions are not supported (the semantic analyzer should have caught this)");
                }
            };
        }
    };

    Ok(())
}

fn emit_tacky_block(block: &parser::ast::Block, instructions: &mut Vec<Instruction>) -> Result<(), String>
{
    match block {
        parser::ast::Block::Blk(block_items) => {
            for block_item in block_items {
                emit_tacky_block_item(block_item, instructions)?;
            }
        }
    }

    Ok(())
}


fn emit_tacky_function_definition(func_def: &parser::ast::FunctionDeclaration) -> Result<FunctionDefinition, String>
{
     match func_def {
        parser::ast::FunctionDeclaration::Declarant(func_name, params, Some(block)) => {
            let mut instructions = vec![];

            emit_tacky_block(block, &mut instructions)?;

            //Force the function to return, in case control reaches the end of its body
            instructions.push(Instruction::Return(Val::IntConstant(0)));
            Ok(FunctionDefinition::Function(func_name.clone(), params.clone(), instructions))
        },
        _ => { return Err(format!("TACKY Conversion: expected function definition, got '{:?}'", *func_def)); }
    }
}


pub fn emit_tacky_program(program: &parser::ast::Program) -> Result<Program, String>
{
    match program {
        parser::ast::Program::ProgramDefinition(func_defs) => {
            let mut tacky_func_defs = vec![];
            for func_def in func_defs {
                if let parser::ast::FunctionDeclaration::Declarant(_, _, Some(_)) = func_def {
                    //Emit tacky only for function declarations that are definitions (i.e.  have bodies)
                    let tacky_func_def = emit_tacky_function_definition(func_def)?;
                    tacky_func_defs.push(tacky_func_def);
                }
            }
            Ok(Program::ProgramDefinition(tacky_func_defs))
        },
        _ => { return Err(format!("Tacky conversion: expected ProgramDefinition, got '{:?}'", program)); }
    }

}



pub use self::emit_tacky_program as generate_tacky_ast;
pub use pretty_print::pretty_print_tacky_ast;