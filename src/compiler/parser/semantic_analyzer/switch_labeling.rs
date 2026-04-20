use std::collections::HashMap;


use super::super::super::parser::ast::*;
use super::unique_global_labels::make_unique_global_switch_label;
use super::unique_global_labels::make_unique_case_label;
use super::unique_global_labels::make_global_default_label;
use super::constant_expression_evaluator::evaluate_constant_expression;




fn label_unlabeled_statement_switch_statements(unlabeled_stmnt: &mut UnlabeledStatement, switch_label: &Option<String>, case_labels_map: &mut HashMap<Const, String>, default_label: &mut Option<String>) -> Result<(), String>
{
    match unlabeled_stmnt {
        UnlabeledStatement::Break(break_type, break_label) => {
            match break_type {
                Some(BreakType::Switch) => {
                    assert!(break_label.is_none());
                    if switch_label.is_none() {
                        return Err(format!("break statement outside loop or switch"));
                    }

                    break_label.replace(switch_label.clone().unwrap());
                },
                _ => {}
            }
        },
        UnlabeledStatement::Compound(block) => {
            label_block_switch_statements(block, switch_label, case_labels_map, default_label)?;
        },
        UnlabeledStatement::Switch(_, body, switch_stmnt_label, switch_stmnt_case_label_map, switch_stmnt_default_label) => {
            let switch_stmnt_global_label = make_unique_global_switch_label();
            switch_stmnt_label.replace(switch_stmnt_global_label.clone());
            label_statement_switch_statements(body, &Some(switch_stmnt_global_label), switch_stmnt_case_label_map, switch_stmnt_default_label)?;
        },
        UnlabeledStatement::DoWhile(body, _, _) |
        UnlabeledStatement::While(_, body, _) |
        UnlabeledStatement::For(_, _, _, body, _) => {
            label_statement_switch_statements(body, switch_label, case_labels_map, default_label)?;
        },
        UnlabeledStatement::If(_, then_stmnt, else_stmnt) => {
            label_statement_switch_statements(then_stmnt, switch_label, case_labels_map, default_label)?;
            if let Some(else_stmnt) = else_stmnt {
                label_statement_switch_statements(else_stmnt, switch_label, case_labels_map, default_label)?;
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


fn label_statement_switch_statements(stmnt: &mut Statement, switch_label: &Option<String>, case_labels_map: &mut HashMap<Const, String>, default_label: &mut Option<String>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            label_unlabeled_statement_switch_statements(unlabeled_stmnt, switch_label, case_labels_map, default_label)?;
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

                        let global_case_label = make_unique_case_label(&switch_label.clone().unwrap(), case_value.to_i64());
                        case_labels_map.insert(case_value, global_case_label.clone());
                        *label = Label::ResolvedCase(global_case_label.clone());

                    },
                    Label::Default => {
                        if switch_label.is_none() {
                            return Err(format!("default: label outside switch statement"));
                        }
                        if default_label.is_some() {
                            return Err(format!("Error: Duplicate default: label"));
                        }
                        let global_default_label = make_global_default_label(&switch_label.clone().unwrap());
                        default_label.replace(global_default_label.clone());

                        *label = Label::ResolvedCase(global_default_label.clone());
                    },
                    Label::Goto(_) => {},
                    _ => {
                        panic!("Unexpected label: {:?}", label);
                    }
                }
            }

            label_unlabeled_statement_switch_statements(unlabeled_stmnt, switch_label, case_labels_map, default_label)?;
        }
    };

    Ok(())
}


fn label_block_item_switch_statements(block_item: &mut BlockItem, switch_label: &Option<String>, case_labels_map: &mut HashMap<Const, String>, default_label: &mut Option<String>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            Ok(())
        },
        BlockItem::S(stmnt) => {
            label_statement_switch_statements(stmnt, switch_label, case_labels_map, default_label)?;
            Ok(())
        }
    }
}


pub fn label_block_switch_statements(block: &mut Block, switch_label: &Option<String>, case_labels_map: &mut HashMap<Const, String>, default_label: &mut Option<String>) -> Result<(), String>
{
     match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                label_block_item_switch_statements(block_item, switch_label, case_labels_map, default_label)?;
            }
        }
    };

    Ok(())
}