use super::super::super::parser::ast::*;


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
        UnlabeledStatement::Switch(_, body,_,_,_) => {
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



pub fn check_and_classify_block_break_statements(
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