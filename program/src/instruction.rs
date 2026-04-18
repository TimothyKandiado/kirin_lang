pub type Instruction = u32;
pub type Opcode = u8;

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

    pub fn set_opcode(mut self, opcode: Opcode) -> Self {
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
        opcode: Opcode,
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
        opcode: Opcode,
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

    pub fn new_format_c(opcode: Opcode, dest: Instruction, const19: Instruction) -> Instruction {
        InstructionBuilder::new()
            .set_opcode(opcode)
            .set_dest(dest)
            .set_const19(const19)
            .build()
    }
}

pub struct InstructionDecoder {}

impl InstructionDecoder {
    #[inline]
    pub fn decode_opcode(instruction: Instruction) -> Instruction {
        (instruction >> OPCODE_SHIFT) & OPCODE_MASK
    }

    #[inline]
    pub fn decode_dest(instruction: Instruction) -> Instruction {
        (instruction >> RD_SHIFT) & REG_MASK
    }

    #[inline]
    pub fn decode_src1(instruction: Instruction) -> Instruction {
        (instruction >> RA_SHIFT) & REG_MASK
    }

    #[inline]
    pub fn decode_src2(instruction: Instruction) -> Instruction {
        (instruction >> RB_SHIFT) & REG_MASK
    }

    #[inline]
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

    #[inline]
    pub fn decode_const19(instruction: Instruction) -> Instruction {
        (instruction) & CONST19_MASK
    }

    #[inline]
    pub fn decode_const13(instruction: Instruction) -> Instruction {
        (instruction) & CONST13_MASK
    }
}

#[cfg(test)]
mod tests {
    use crate::instruction::{InstructionBuilder, InstructionDecoder};

    #[test]
    fn test_opcode_encoding() {
        let actual_opcode = 10;

        let instruction = InstructionBuilder::new().set_opcode(actual_opcode).build();

        let decoded_opcode = InstructionDecoder::decode_opcode(instruction) as u8;

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
        let actual_opcode = 12;
        let actual_dest = 10;
        let actual_src1 = 11;
        let actual_src2 = 12;

        let instruction =
            InstructionBuilder::new_format_a(actual_opcode, actual_dest, actual_src1, actual_src2);

        let decoded_opcode = InstructionDecoder::decode_opcode(instruction) as u8;
        let decoded_dest = InstructionDecoder::decode_dest(instruction);
        let decoded_src1 = InstructionDecoder::decode_src1(instruction);
        let decoded_src2 = InstructionDecoder::decode_src2(instruction);

        assert_eq!(actual_opcode, decoded_opcode);
        assert_eq!(actual_dest, decoded_dest);
        assert_eq!(actual_src1, decoded_src1);
        assert_eq!(actual_src2, decoded_src2);
    }
}
