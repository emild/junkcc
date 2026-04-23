

use super::super::ast::*;

pub fn label_program(prog: &mut Program) -> Result<(), String>
{
    super::goto_labels::resolve_program_goto_labels(prog)?;
    super::loop_labeling::label_program_loops(prog)?;
    super::break_classifier::check_and_classify_program_break_statements(prog)?;
    super::switch_labeling::label_program_switch_statements(prog)?;

    Ok(())
}
