mod debug;
mod instruction;

pub mod opcode;

use std::error::Error;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;

use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
pub use debug::{debug_print_instruction, debug_program};
pub use instruction::Instruction;
pub use instruction::InstructionBuilder;
pub use instruction::InstructionDecoder;

pub const BYTECODE_VERSION: u16 = 2;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Constant {
    I64(i64),
    F64(f64),
    String(String),
}

const CONST_I64_DISCRIMINANT: u8 = 0;
const CONST_F64_DISCRIMINANT: u8 = 1;
const CONST_STRING_DISCRIMINANT: u8 = 2;

impl Constant {
    pub fn discriminant(&self) -> u8 {
        match self {
            Constant::I64(_) => CONST_I64_DISCRIMINANT,
            Constant::F64(_) => CONST_F64_DISCRIMINANT,
            Constant::String(_) => CONST_STRING_DISCRIMINANT,
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
    pub return_args: u8,
}

const FUNCTION_METADATA_SIZE: usize = 12;

impl FunctionMetadata {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.write_u16::<LittleEndian>(self.name_idx).unwrap();
        buffer.write_u8(self.function_kind as u8).unwrap();
        buffer.write_u32::<LittleEndian>(self.code_offset).unwrap();
        buffer.write_u16::<LittleEndian>(self.code_length).unwrap();
        buffer.write_u8(self.registers).unwrap();
        buffer.write_u8(self.parameters).unwrap();
        buffer.write_u8(self.return_args).unwrap();

        assert!(buffer.len() == FUNCTION_METADATA_SIZE);

        buffer
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let mut cursor = Cursor::new(data);

        let name_idx = cursor.read_u16::<LittleEndian>()?;
        let function_kind = cursor.read_u8()?;
        let code_offset = cursor.read_u32::<LittleEndian>()?;
        let code_length = cursor.read_u16::<LittleEndian>()?;
        let registers = cursor.read_u8()?;
        let parameters = cursor.read_u8()?;
        let return_args = cursor.read_u8()?;

        let function_kind = match function_kind {
            x if x == FunctionKind::Native as u8 => FunctionKind::Native,
            x if x == FunctionKind::Bytecode as u8 => FunctionKind::Bytecode,

            _ => return Err(format!("{} if not a valid function kind", function_kind).into()),
        };

        Ok(FunctionMetadata {
            name_idx,
            function_kind,
            code_offset,
            code_length,
            registers,
            parameters,
            return_args,
        })
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

const PROGRAM_HEADER_SIZE: usize = 30;

pub struct ProgramHeader {
    pub magic_number: [u8; 12],

    pub bytecode_version: u16,
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,

    pub instruction_count: u32,
    pub constant_count: u16,
    pub function_count: u16,
    pub type_count: u16,
}

const MAGIC_NUMBER_SIZE: usize = 12;

impl ProgramHeader {
    pub fn new(
        instruction_count: u32,
        constant_count: u16,
        function_count: u16,
        type_count: u16,
    ) -> Self {
        let magic_number: [u8; MAGIC_NUMBER_SIZE] = *b"kirinprogram";

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
            type_count,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.write_all(&self.magic_number).unwrap();

        buffer
            .write_u16::<LittleEndian>(self.bytecode_version)
            .unwrap();
        buffer.write_u8(self.version_major).unwrap();
        buffer.write_u8(self.version_minor).unwrap();
        buffer.write_u8(self.version_patch).unwrap();

        buffer
            .write_u32::<LittleEndian>(self.instruction_count)
            .unwrap();
        buffer
            .write_u16::<LittleEndian>(self.constant_count)
            .unwrap();
        buffer
            .write_u16::<LittleEndian>(self.function_count)
            .unwrap();
        buffer.write_u16::<LittleEndian>(self.type_count).unwrap();

        assert!(buffer.len() < PROGRAM_HEADER_SIZE);
        buffer.resize(PROGRAM_HEADER_SIZE, 0);

        buffer
    }

    pub fn from_bytes<R: Read>(mut data: R) -> Result<Self, Box<dyn Error>> {
        let mut magic_number = [0u8; MAGIC_NUMBER_SIZE];

        data.read_exact(&mut magic_number).unwrap();

        let bytecode_version = data.read_u16::<LittleEndian>()?;

        let version_major = data.read_u8()?;
        let version_minor = data.read_u8()?;
        let version_patch = data.read_u8()?;

        let instruction_count = data.read_u32::<LittleEndian>()?;
        let constant_count = data.read_u16::<LittleEndian>()?;
        let function_count = data.read_u16::<LittleEndian>()?;
        let type_count = data.read_u16::<LittleEndian>()?;

        Ok(Self {
            magic_number,
            bytecode_version,

            version_major,
            version_minor,
            version_patch,

            instruction_count,
            constant_count,
            function_count,
            type_count,
        })
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub types: Vec<TypeInfo>,
    pub functions: Vec<FunctionMetadata>,
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
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
            self.types.len() as u16,
        );

        buffer.write_all(&header.to_bytes()).unwrap();

        // instructions
        for instruction in self.instructions.iter() {
            buffer.write_u32::<LittleEndian>(*instruction).unwrap();
        }

        // constants
        for constant in self.constants.iter() {
            let discriminant = constant.discriminant();
            buffer.write_all(&discriminant.to_le_bytes()).unwrap();
            match constant {
                Constant::I64(int) => buffer.write_i64::<LittleEndian>(*int).unwrap(),
                Constant::F64(float) => buffer.write_f64::<LittleEndian>(*float).unwrap(),
                Constant::String(string) => {
                    let length = string.len() as u64;

                    buffer.write_u64::<LittleEndian>(length).unwrap();
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
            buffer.write_u8(type_info.kind as u8).unwrap();
            buffer.write_u8(type_info.size).unwrap();
        }
    }

    pub fn read_from_bytes(data: &[u8]) -> Result<Program, Box<dyn Error>> {
        let mut program = Program::new();

        let mut source = Cursor::new(data);

        let mut header_buffer = [0u8; PROGRAM_HEADER_SIZE];
        source.read_exact(&mut header_buffer)?;

        let header_data = Cursor::new(header_buffer);
        let header = ProgramHeader::from_bytes(header_data)?;

        for _ in 0..header.instruction_count {
            let instruction = source.read_u32::<LittleEndian>()?;
            program.instructions.push(instruction);
        }

        for _ in 0..header.constant_count {
            let discriminant = source.read_u8()?;

            match discriminant {
                CONST_I64_DISCRIMINANT => {
                    let val = source.read_i64::<LittleEndian>()?;
                    program.constants.push(Constant::I64(val));
                }
                CONST_F64_DISCRIMINANT => {
                    let val = source.read_f64::<LittleEndian>()?;
                    program.constants.push(Constant::F64(val));
                }
                CONST_STRING_DISCRIMINANT => {
                    let size = source.read_u64::<LittleEndian>()?;

                    let mut buffer = vec![0; size as usize];

                    source.read_exact(&mut buffer)?;
                    let string = str::from_utf8(&buffer)?.to_string();

                    program.constants.push(Constant::String(string));
                }

                _ => {
                    return Err(
                        format!("{} is not a valid constant discriminant", discriminant).into(),
                    );
                }
            }
        }

        for _ in 0..header.function_count {
            let mut buffer = [0u8; FUNCTION_METADATA_SIZE];

            source.read_exact(&mut buffer)?;

            let function = FunctionMetadata::from_bytes(&buffer)?;

            program.functions.push(function);
        }

        for _ in 0..header.type_count {
            let kind = source.read_u8()?;
            let size = source.read_u8()?;

            let kind = match kind {
                x if x == TypeKind::Bool as u8 => TypeKind::Bool,
                x if x == TypeKind::I64 as u8 => TypeKind::I64,
                x if x == TypeKind::F64 as u8 => TypeKind::F64,
                x if x == TypeKind::String as u8 => TypeKind::String,

                _ => return Err(format!("{} is not a valid type kind", kind).into()),
            };

            let type_info = TypeInfo { kind, size };
            program.types.push(type_info);
        }

        Ok(program)
    }
}
