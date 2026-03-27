use super::super::super::parser::ast::*;
use super::LoopType;
use super::unique_global_labels::make_unique_global_loop_label;




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



fn label_statement_loops(stmnt: &mut Statement, loop_label: &Option<String>) -> Result<(), String>
{
    match stmnt {
        Statement::Stmnt(_, unlabeled_stmnt) => {
            label_unlabled_statement_loops(unlabeled_stmnt, loop_label)?;
        }
    };

    Ok(())
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



pub fn label_block_loops(block: &mut Block, loop_label: &Option<String>) -> Result<(), String>
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