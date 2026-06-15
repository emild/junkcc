use std::{collections::HashMap, sync::{LazyLock, Mutex, atomic::{AtomicUsize, Ordering}}};

use crate::compiler::parser::StaticInit;

use super::super::parser::ast::Type;


static DOUBLE_CONSTANT_NAME_INDEX: AtomicUsize = AtomicUsize::new(0);

static DOUBLE_CONSTANTS: LazyLock<Mutex<HashMap<u64, String> > > = LazyLock::new(|| Mutex::new(HashMap::new()));


fn new_double_constant_name() -> String
{
    let indx = DOUBLE_CONSTANT_NAME_INDEX.fetch_add(1, Ordering::SeqCst);
    format!("CONST_DBL_{}", indx)
}

pub fn make_double_constant(val: f64) -> String
{
    let bits = val.to_bits();
    let constant_label = DOUBLE_CONSTANTS.lock().unwrap().entry(bits).or_insert(new_double_constant_name()).clone();
    format!(".L_{}", constant_label)
}



pub fn get_double_constant_top_level_items() -> Vec<super::ast::TopLevel>
{
    let double_constants = DOUBLE_CONSTANTS.lock().unwrap();
    const NEGATIVE_ZERO: u64 = (-0.0f64).to_bits();

    let mut result = vec![];

    for (val, label) in double_constants.iter() {
        // Hack for using xorpd instruction to implement unary minus for doubles
        let alignment = if *val == NEGATIVE_ZERO {
            16
        }
        else {
            Type::Double.alignment()
        };

        result.push(super::ast::TopLevel::StaticConstant(label.clone(), alignment, StaticInit::DoubleInit(f64::from_bits(*val))));
    }

    result
}