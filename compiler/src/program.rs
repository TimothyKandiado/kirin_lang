mod code_gen;
mod debug;
mod register_allocation;

pub use code_gen::build_program;
pub use debug::debug_program;

use crate::{instruction::Instruction, parser::ValueType};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Constant<'a> {
    I64(i64),
    F64(f64),
    String(&'a str),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum FunctionKind {
    Bytecode,
    Native,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FunctionMetadata {
    pub name_idx: u16,
    pub function_kind: FunctionKind,
    pub code_offset: usize,
    pub code_length: u16,
    pub registers: u8,
    pub parameters: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TypeKind {
    I64,
    F64,
    Bool,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct TypeInfo {
    pub kind: TypeKind,
    pub size: u8,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Program<'a> {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant<'a>>,
    pub types: Vec<TypeInfo>,
    pub functions: Vec<FunctionMetadata>,
}

impl Default for Program<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Program<'_> {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            types: Vec::new(),
            functions: Vec::new(),
        }
    }
}

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
