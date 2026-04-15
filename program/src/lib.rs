mod instruction;

use std::io::Write;

pub use instruction::Instruction;
pub use instruction::InstructionDecoder;
pub use instruction::InstructionBuilder;

pub const BYTECODE_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Constant<'a> {
    I64(i64),
    F64(f64),
    String(&'a str),
}

impl Constant<'_> {
    pub fn discriminant(&self) -> u8 {
        match self {
            Constant::I64(_) => 1,
            Constant::F64(_) => 2,
            Constant::String(_) => 3
        }
    }
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
    pub code_offset: u32,
    pub code_length: u16,
    pub registers: u8,
    pub parameters: u8,
}

impl FunctionMetadata {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.write_all(&self.name_idx.to_le_bytes()).unwrap();
        buffer.write_all(&(self.function_kind as u8).to_le_bytes()).unwrap();
        buffer.write_all(&self.code_offset.to_le_bytes()).unwrap();
        buffer.write_all(&self.code_length.to_le_bytes()).unwrap();
        buffer.write_all(&self.registers.to_le_bytes()).unwrap();
        buffer.write_all(&self.parameters.to_le_bytes()).unwrap();

        buffer
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
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

pub struct ProgramHeader {
    pub magic_number: &'static [u8; 12],

    pub bytecode_version: u16,
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,

    pub instruction_count: u32,
    pub constant_count: u16,
    pub function_count: u16,
    pub type_count: u16,
}

impl ProgramHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.write_all(self.magic_number).unwrap();
        buffer.write_all(&self.version_major.to_le_bytes()).unwrap();
        buffer.write_all(&self.version_minor.to_le_bytes()).unwrap();
        buffer.write_all(&self.version_patch.to_le_bytes()).unwrap();

        buffer.write_all(&self.instruction_count.to_le_bytes()).unwrap();
        buffer.write_all(&self.constant_count.to_le_bytes()).unwrap();
        buffer.write_all(&self.function_count.to_le_bytes()).unwrap();
        buffer.write_all(&self.type_count.to_le_bytes()).unwrap();

        assert!(buffer.len() < 30);
        buffer.resize(30, 0);

        buffer
    }

    pub fn new(instruction_count: u32, constant_count: u16, function_count: u16, type_count: u16) -> Self {
        let magic_number = b"kirinprogram";

        let version_major: u8 = env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap();
        let version_minor: u8 = env!("CARGO_PKG_VERSION_MINOR").parse().unwrap();
        let version_patch: u8 = env!("CARGO_PKG_VERSION_PATCH").parse().unwrap();

        Self {
            magic_number,

            bytecode_version: BYTECODE_VERSION,

            version_major,
            version_minor,
            version_patch,

            instruction_count,
            constant_count,
            function_count,
            type_count
        }
    }
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

    pub fn write_bytes<W: Write>(&self, buffer: &mut W) {
        let header = ProgramHeader::new(
            self.instructions.len() as u32, 
            self.constants.len() as u16, 
            self.functions.len() as u16, 
            self.types.len() as u16
        );

        buffer.write_all(&header.to_bytes()).unwrap();

        // instructions
        for instruction in self.instructions.iter() {
            buffer.write_all(&instruction.to_le_bytes()).unwrap();
        }

        // constants
        for constant in self.constants.iter() {
            let discriminant = constant.discriminant();
            buffer.write_all(&discriminant.to_le_bytes()).unwrap();
            match constant {
                Constant::I64(int) => buffer.write_all(&int.to_le_bytes()).unwrap(),
                Constant::F64(float) => buffer.write_all(&float.to_le_bytes()).unwrap(),
                Constant::String(string) => {
                    let length = string.len() as u64;

                    buffer.write_all(&length.to_le_bytes()).unwrap();
                    buffer.write_all(string.as_bytes()).unwrap();
                }
            }
        }

        // functions
        for function in self.functions.iter() {
            buffer.write_all(&function.to_bytes()).unwrap()
        } 

        // types
        for type_info in self.types.iter() {
            buffer.write_all(&(type_info.kind as u8).to_le_bytes()).unwrap();
            buffer.write_all(&type_info.size.to_le_bytes()).unwrap();
        }
    }
}