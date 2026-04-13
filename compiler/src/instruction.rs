pub type Instruction = u32;


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum OpCode {
    NoOp = 0x0,

    // [opcode: 7bits, imm: ]
    ConstI64Imm,
    ConstI64,
    ConstF64,
    ConstTrue,
    ConstFalse,
    ConstStr,

    Move,
    Swap,

    AddI64,
    SubI64,
    MulI64,
    DivI64,
    ModI64,
    PowI64,
    NegI64,

    CmpLtI64,
    CmpLeI64,

    CmpEq,

    Not,
    And,
    Or,

    BrFalse,
    Jump,

    Call,   // direct call
    Invoke, // indirect call
    Ret,
    RetVoid,
    Halt,
}

impl OpCode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            x if x == OpCode::NoOp as u32 => OpCode::NoOp,
            x if x == OpCode::ConstI64Imm as u32 => OpCode::ConstI64Imm,
            x if x == OpCode::ConstI64 as u32 => OpCode::ConstI64,
            x if x == OpCode::ConstF64 as u32 => OpCode::ConstF64,
            x if x == OpCode::ConstTrue as u32 => OpCode::ConstTrue,
            x if x == OpCode::ConstFalse as u32 => OpCode::ConstFalse,
            x if x == OpCode::ConstStr as u32 => OpCode::ConstStr,

            x if x == OpCode::Move as u32 => OpCode::Move,
            x if x == OpCode::Swap as u32 => OpCode::Swap,

            x if x == OpCode::AddI64 as u32 => OpCode::AddI64,
            x if x == OpCode::SubI64 as u32 => OpCode::SubI64,
            x if x == OpCode::MulI64 as u32 => OpCode::MulI64,
            x if x == OpCode::DivI64 as u32 => OpCode::DivI64,
            x if x == OpCode::ModI64 as u32 => OpCode::ModI64,
            x if x == OpCode::PowI64 as u32 => OpCode::PowI64,
            x if x == OpCode::NegI64 as u32 => OpCode::NegI64,

            x if x == OpCode::CmpLtI64 as u32 => OpCode::CmpLtI64,
            x if x == OpCode::CmpLeI64 as u32 => OpCode::CmpLeI64,
            x if x == OpCode::CmpEq as u32 => OpCode::CmpEq,

            x if x == OpCode::Not as u32 => OpCode::Not,
            x if x == OpCode::And as u32 => OpCode::And,
            x if x == OpCode::Or as u32 => OpCode::Or,

            x if x == OpCode::BrFalse as u32 => OpCode::BrFalse,
            x if x == OpCode::Jump as u32 => OpCode::Jump,

            x if x == OpCode::Call as u32 => OpCode::Call,
            x if x == OpCode::Invoke as u32 => OpCode::Invoke,

            x if x == OpCode::Ret as u32 => OpCode::Ret,
            x if x == OpCode::RetVoid as u32 => OpCode::RetVoid,
            x if x == OpCode::Halt as u32 => OpCode::Halt,

            _ => panic!(
                "invalid OpCode value: decimal={} hex={:#X} binary={:#b}",
                value, value, value
            ),
        }
    }
}

// const OPCODE_BITS: u32 = 7;
// const REG_BITS: u32 = 6;
// const IMM19_BITS: u32 = 19;

const OPCODE_SHIFT: u32 = 25;
const RD_SHIFT: u32 = 19; // dest register
const RA_SHIFT: u32 = 13; // source 1 register
const RB_SHIFT: u32 = 7; // source 2 register

const OPCODE_MASK: u32 = 0b1111111;
const REG_MASK: u32 = 0b111111;
const CONST19_MASK: u32 = 0x7FFFF; // 19 bits
const IMM19_MASK: u32 = 0x3FFFF; // 18 bits
const CONST13_MASK: u32 = 0x1FFF; // 13 bits

pub struct InstructionBuilder {
    instruction: Instruction,
}

impl Default for InstructionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl InstructionBuilder {
    pub fn new() -> Self {
        Self { instruction: 0 }
    }

    pub fn build(self) -> Instruction {
        self.instruction
    }

    pub fn set_opcode(mut self, opcode: OpCode) -> Self {
        let opcode = opcode as Instruction;
        assert!(opcode < OPCODE_MASK);
        let shifted = opcode << OPCODE_SHIFT;
        self.instruction |= shifted;
        self
    }

    pub fn set_dest(mut self, destination: Instruction) -> Self {
        assert!(destination < REG_MASK);

        let shifted = destination << RD_SHIFT;
        self.instruction |= shifted;
        self
    }

    pub fn set_src1(mut self, source: Instruction) -> Self {
        assert!(source < REG_MASK);

        let shifted = source << RA_SHIFT;
        self.instruction |= shifted;
        self
    }

    pub fn set_src2(mut self, source: Instruction) -> Self {
        assert!(source < 63);

        let shifted = source << RB_SHIFT;
        self.instruction |= shifted;
        self
    }

    pub fn set_const19(mut self, value: Instruction) -> Self {
        assert!(value < CONST19_MASK);
        self.instruction |= value;
        self
    }

    pub fn set_const13(mut self, value: Instruction) -> Self {
        assert!(value < CONST13_MASK);
        self.instruction |= value;
        self
    }

    pub fn set_imm19(mut self, value: i32) -> Self {
        assert!((-262144..=262143).contains(&value));

        let sign: u32 = if value < 0 { 1 } else { 0 };
        let value = value.unsigned_abs();

        self.instruction |= value & IMM19_MASK;
        self.instruction |= sign << 18;
        self
    }

    /// create new instruction of the format Opcode | dest | src1 | src2
    pub fn new_format_a(
        opcode: OpCode,
        dest: Instruction,
        src1: Instruction,
        src2: Instruction,
    ) -> Instruction {
        InstructionBuilder::new()
            .set_opcode(opcode)
            .set_dest(dest)
            .set_src1(src1)
            .set_src2(src2)
            .build()
    }

    pub fn new_format_b(
        opcode: OpCode,
        dest: Instruction,
        src1: Instruction,
        const13: Instruction,
    ) -> Instruction {
        InstructionBuilder::new()
            .set_opcode(opcode)
            .set_dest(dest)
            .set_src1(src1)
            .set_const13(const13)
            .build()
    }

    pub fn new_format_c(opcode: OpCode, dest: Instruction, const19: Instruction) -> Instruction {
        InstructionBuilder::new()
            .set_opcode(opcode)
            .set_dest(dest)
            .set_const19(const19)
            .build()
    }
}

pub struct InstructionDecoder {}

impl InstructionDecoder {
    pub fn decode_opcode(instruction: Instruction) -> Instruction {
        (instruction >> OPCODE_SHIFT) & OPCODE_MASK
    }

    pub fn decode_dest(instruction: Instruction) -> Instruction {
        (instruction >> RD_SHIFT) & REG_MASK
    }

    pub fn decode_src1(instruction: Instruction) -> Instruction {
        (instruction >> RA_SHIFT) & REG_MASK
    }

    pub fn decode_src2(instruction: Instruction) -> Instruction {
        (instruction >> RB_SHIFT) & REG_MASK
    }

    pub fn decode_imm19(instruction: Instruction) -> i32 {
        let raw = instruction & IMM19_MASK;
        let sign = (instruction >> 18) & 1;

        if sign == 1 {
            // Negative number
            let value: i32 = raw as i32;
            -value
        } else {
            raw as i32
        }
    }

    pub fn decode_const19(instruction: Instruction) -> Instruction {
        (instruction) & CONST19_MASK
    }

    pub fn decode_const13(instruction: Instruction) -> Instruction {
        (instruction) & CONST13_MASK
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::{InstructionBuilder, InstructionDecoder, OpCode};

    #[test]
    fn test_opcode_encoding() {
        let actual_opcode = OpCode::Call;

        let instruction = InstructionBuilder::new().set_opcode(actual_opcode).build();

        let decoded_opcode = OpCode::from_u32(InstructionDecoder::decode_opcode(instruction));

        assert_eq!(actual_opcode, decoded_opcode)
    }

    #[test]
    fn test_dest_encoding() {
        let actual_dest = 13 as u32;

        let instruction = InstructionBuilder::new().set_dest(actual_dest).build();

        let decoded_dest = InstructionDecoder::decode_dest(instruction);

        assert_eq!(actual_dest, decoded_dest)
    }

    #[test]
    fn test_src1_encoding() {
        let actual_src = 33 as u32;

        let instruction = InstructionBuilder::new().set_src1(actual_src).build();

        let decoded_src = InstructionDecoder::decode_src1(instruction);

        assert_eq!(actual_src, decoded_src)
    }

    #[test]
    fn test_src2_encoding() {
        let actual_src = 13 as u32;

        let instruction = InstructionBuilder::new().set_src2(actual_src).build();

        let decoded_src = InstructionDecoder::decode_src2(instruction);

        assert_eq!(actual_src, decoded_src)
    }

    #[test]
    fn test_const19_encoding() {
        let actual_value = 113 as u32;

        let instruction = InstructionBuilder::new().set_const19(actual_value).build();

        let decoded_value = InstructionDecoder::decode_const19(instruction);

        assert_eq!(actual_value, decoded_value)
    }

    #[test]
    fn test_const13_encoding() {
        let actual_value = 113 as u32;

        let instruction = InstructionBuilder::new().set_const13(actual_value).build();

        let decoded_value = InstructionDecoder::decode_const13(instruction);

        assert_eq!(actual_value, decoded_value)
    }

    #[test]
    fn test_imm19_encoding() {
        let actual_value = -113 as i32;

        let instruction = InstructionBuilder::new().set_imm19(actual_value).build();

        let decoded_value = InstructionDecoder::decode_imm19(instruction);

        assert_eq!(actual_value, decoded_value)
    }

    #[test]
    fn test_format_a_encoding() {
        let actual_opcode = OpCode::CmpEq;
        let actual_dest = 10;
        let actual_src1 = 11;
        let actual_src2 = 12;

        let instruction =
            InstructionBuilder::new_format_a(actual_opcode, actual_dest, actual_src1, actual_src2);

        let decoded_opcode = OpCode::from_u32(InstructionDecoder::decode_opcode(instruction));
        let decoded_dest = InstructionDecoder::decode_dest(instruction);
        let decoded_src1 = InstructionDecoder::decode_src1(instruction);
        let decoded_src2 = InstructionDecoder::decode_src2(instruction);

        assert_eq!(actual_opcode, decoded_opcode);
        assert_eq!(actual_dest, decoded_dest);
        assert_eq!(actual_src1, decoded_src1);
        assert_eq!(actual_src2, decoded_src2);
    }
}
