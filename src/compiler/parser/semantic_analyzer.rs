use std::collections::HashMap;

use super::ast::*;

mod constant_expression_evaluator;
mod loop_labeling;
mod unique_global_labels;
mod goto_labels;
mod break_classifier;
mod switch_labeling;
mod resolver;
mod type_checker;
mod labels;


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


pub use type_checker::IdentifierAttrs;
pub use type_checker::SymbolInfo;
pub use type_checker::InitialValue;
pub use type_checker::StaticInit;


pub fn semantic_analysis(prog: &Program) -> Result<(Program, HashMap<String, type_checker::SymbolInfo>), String>
{
    let mut resolved_program = resolver::resolve_program(prog)?;
    let mut symbol_table = HashMap::new();
    type_checker::typecheck_program(&mut resolved_program, &mut symbol_table)?;

    labels::label_program(&mut resolved_program)?;

    Ok((resolved_program, symbol_table))
}
