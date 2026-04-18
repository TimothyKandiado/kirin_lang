mod code_gen;
mod register_allocation;

pub use code_gen::build_program;

use crate::parser::ValueType;

// get size of a type in registers, a register is 64bits
impl ValueType {
    pub fn get_size(&self) -> usize {
        match self {
            ValueType::Undefined => panic!("undefined value type has no size"),
            ValueType::I64 => 1,
            ValueType::F64 => 1,
            ValueType::String => 1,
            ValueType::Bool => 1,
            ValueType::Void => 0,
            ValueType::Any => 2,
            ValueType::Fn(_) => 1,
        }
    }
}
