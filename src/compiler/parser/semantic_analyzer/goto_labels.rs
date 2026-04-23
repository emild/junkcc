use std::collections::HashMap;

use super::unique_global_labels::make_unique_global_goto_label;
use super::super::super::parser::ast::*;


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
        UnlabeledStatement::Switch(_ , body ,_ ,_ ,_ ) => {
            check_statement_goto_labels(body, goto_labels)?;
        }

        _ => {}
    }

    Ok(())
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


pub fn check_block_goto_labels(block: &Block, goto_labels: &mut HashMap<String, String>) -> Result<(), String>
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

//-----------------------------------------------------------------------------------------

fn apply_global_goto_labels_to_unlabeled_statement(unlabeled_stmnt: &mut UnlabeledStatement, goto_labels: &HashMap<String, String>) -> Result<(), String>
{
    match unlabeled_stmnt {
        UnlabeledStatement::Goto(goto_label) => {
            if let Some(global_goto_label) = goto_labels.get(goto_label) {
                *goto_label = global_goto_label.clone();
            }
            else {
                panic!("Cannot find global label for goto target label '{}'", goto_label);
            }
        },
        UnlabeledStatement::If(_,then_stmnt , None) => {
            apply_global_goto_labels_to_statement(then_stmnt, goto_labels)?;
        },
        UnlabeledStatement::If(_,then_stmnt , Some(else_stmnt)) => {
            apply_global_goto_labels_to_statement(then_stmnt, goto_labels)?;
            apply_global_goto_labels_to_statement(else_stmnt, goto_labels)?;
        },
        UnlabeledStatement::Compound(Block::Blk(block_items)) => {
            for block_item in block_items {
                apply_global_goto_labels_to_block_item(block_item, goto_labels)?;
            }
        },
        UnlabeledStatement::DoWhile(body, _, _)    |
        UnlabeledStatement::While(_ , body, _ )    |
        UnlabeledStatement::For(_, _, _, body, _)  |
        UnlabeledStatement::Switch(_ , body ,_ ,_ ,_ ) => {
            apply_global_goto_labels_to_statement(body, goto_labels)?;
        }

        _ => {}
    }

    Ok(())
}


fn apply_global_goto_labels_to_statement(stmnt: &mut Statement, goto_labels: &HashMap<String, String>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(None, unlabeled_stmnt) => {
            apply_global_goto_labels_to_unlabeled_statement(unlabeled_stmnt, goto_labels)?;
        },
        Statement::Stmnt(Some(stmnt_labels),unlabeled_stmnt ) => {
            for stmnt_label in stmnt_labels {
                if let Label::Goto(stmnt_goto_label) = stmnt_label {
                    if let Some(global_goto_label) = goto_labels.get(stmnt_goto_label) {
                        *stmnt_goto_label = global_goto_label.clone();
                    }
                    else {
                        panic!("Cannot find global label for label '{}'", stmnt_goto_label);
                    }
                }
            }
            apply_global_goto_labels_to_unlabeled_statement(unlabeled_stmnt, goto_labels)?;
        }
    }

    Ok(())
}




fn apply_global_goto_labels_to_block_item(block_item: &mut BlockItem, goto_labels: &HashMap<String, String>) -> Result<(), String>
{
    match block_item {
        BlockItem::D(decl) => {
            Ok(())
        },
        BlockItem::S(stmnt) => {
            apply_global_goto_labels_to_statement(stmnt, goto_labels)?;
            Ok(())
        }
    }
}



fn apply_global_goto_labels_to_block(block: &mut Block, goto_labels: &HashMap<String, String>) -> Result<(), String>
{
    match block {
        Block::Blk(block_items) => {
            for block_item in block_items {
                apply_global_goto_labels_to_block_item(block_item, goto_labels)?;
            }
        }
    };

    Ok(())
}



pub fn resolve_program_goto_labels(prog: &mut Program) -> Result<(), String>
{
    let Program::ProgramDefinition(decls) = prog;

    for decl in decls {
        if let Declaration::FunDecl(FunctionDeclaration::Declarant(_,_, Some(body) ,_,_)) = decl {
            let mut goto_labels = HashMap::new();
            check_block_goto_labels(body, &mut goto_labels)?;
            apply_global_goto_labels_to_block(body, &goto_labels)?;
        }
    }

    Ok(())
}
