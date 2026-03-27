use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::compiler::parser::semantic_analyzer::LoopType;

static GOTO_LABEL_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);
static LOOP_LABEL_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);
static SWITCH_LABEL_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);
static LOCAL_TMP_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);


pub fn make_unique_global_goto_label(label: &str) -> String
{
    let index = GOTO_LABEL_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let global_label = format!("goto_lbl_{}.{}", label, index);

    global_label
}


pub fn make_unique_global_loop_label(loop_type: &LoopType) -> String
{
    let loop_type_str = HashMap::from([
        (LoopType::While, "while"),
        (LoopType::DoWhile, "do_while"),
        (LoopType::For, "for")
    ]);

    let index = LOOP_LABEL_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let global_label = format!("{}_loop.{}", loop_type_str.get(loop_type).unwrap(), index);

    global_label
}


pub fn make_unique_global_switch_label() -> String
{
    let index = SWITCH_LABEL_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let global_label = format!("switch_lbl_.{}", index);

    global_label
}


pub fn make_unique_case_label(switch_label: &String, case_value: i32) -> String
{
    let global_label = format!("{}_{:08X}", switch_label, case_value);

    global_label
}


pub fn make_global_default_label(switch_label: &String) -> String
{
    let global_label = format!("{}_default", switch_label);

    global_label
}


pub fn make_unique_global_name(var_name: &str) -> String
{
    let index = LOCAL_TMP_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    let temp_name = format!("local.var.{}.{}", var_name, index);

    temp_name
}