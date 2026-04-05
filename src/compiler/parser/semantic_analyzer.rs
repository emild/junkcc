use std::collections::HashMap;

use constant_expression_evaluator::evaluate_constant_expression;
use goto_labels::check_block_goto_labels;
use break_classifier::check_and_classify_block_break_statements;
use loop_labeling::label_block_loops;

use super::ast::*;

mod constant_expression_evaluator;
mod loop_labeling;
mod unique_global_labels;
mod goto_labels;
mod break_classifier;
mod switch_labeling;
mod resolver;
mod type_checker;


pub struct IdentifierInfo {
    global_name: String,
    from_current_scope: bool,
    has_linkage: bool
}



#[derive(PartialEq, Eq, Hash)]
enum LoopType {
    While,
    DoWhile,
    For
}



pub fn semantic_analysis(prog: &Program) -> Result<Program, String>
{
    let resolved_program = resolver::resolve_program(prog)?;
    let mut symbol_table = HashMap::new();
    type_checker::typecheck_program(&resolved_program, &mut symbol_table)?;

    Ok(resolved_program)
}
