mod code_gen;
mod register_allocation;

use crate::{
    instruction::Instruction,
    ir::{IrFunction, IrModule},
    parser::ValueType,
    program::register_allocation::RegisterAllocator,
};

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

struct ProgramBuilder<'a> {
    instructions: Vec<Instruction>,
    constants: Vec<Constant<'a>>,
    types: Vec<TypeInfo>,
    functions: Vec<FunctionMetadata>,
}

impl<'a> ProgramBuilder<'a> {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            types: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn build_module(mut self, ir_module: IrModule<'a>) -> Program<'a> {
        // load function symbols
        for function in ir_module.functions.iter() {
            match function {
                IrFunction::Bytecode {
                    name,
                    params,
                    ret_type: _,
                    blocks: _,
                    reg_count: _,
                    reg_types: _,
                } => {
                    let constant = Constant::String(name);
                    let name_idx = self.push_constant(constant) as u16;

                    let func_metadata = FunctionMetadata {
                        name_idx,
                        code_length: 0,
                        code_offset: 0,
                        function_kind: FunctionKind::Bytecode,
                        registers: 0,
                        parameters: params.len() as u8,
                    };

                    self.functions.push(func_metadata);
                }
                IrFunction::Native {
                    name,
                    params,
                    ret_type: _,
                } => {
                    let constant = Constant::String(name);
                    let name_idx = self.push_constant(constant) as u16;

                    let func_metadata = FunctionMetadata {
                        name_idx,
                        code_length: 0,
                        code_offset: 0,
                        function_kind: FunctionKind::Native,
                        registers: 0,
                        parameters: params.len() as u8,
                    };

                    self.functions.push(func_metadata);
                }
            }
        }

        for function in ir_module.functions.iter() {
            self.build_function(function);
        }

        Program {
            instructions: self.instructions,
            constants: self.constants,
            types: self.types,
            functions: self.functions,
        }
    }

    fn build_function(&mut self, function: &IrFunction<'a>) {
        let (total_registers, reg_allocations) = RegisterAllocator::allocate_for_function(function);

        let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type: _,
            blocks: _,
            reg_count: _,
            reg_types: _,
        } = function
        else {
            return;
        };

        println!("=== Register Allocation ===");
        println!("Total registers: {}", total_registers);
        for (idx, alloc) in reg_allocations.iter().enumerate() {
            println!("VReg: [{}] ; Allocation: {:?}", idx, alloc)
        }
    }

    fn push_constant(&mut self, new: Constant<'a>) -> usize {
        let existing = self.constants.iter().position(|constant| constant == &new);

        if let Some(index) = existing {
            index
        } else {
            self.constants.push(new);
            self.constants.len() - 1
        }
    }
}

pub fn build_program<'a>(ir_module: IrModule<'a>) -> Program<'a> {
    ProgramBuilder::new().build_module(ir_module)
}

// get size of a type in registers, a register is 64bits
fn get_type_size(val_type: &ValueType) -> usize {
    match val_type {
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
