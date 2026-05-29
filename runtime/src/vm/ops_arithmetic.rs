use program::{Instruction, InstructionDecoder};

use crate::vm::VM;

pub trait VmArithmeticOps {
    fn add_i64(&mut self, instruction: Instruction);
    fn sub_i64(&mut self, instruction: Instruction);
    fn mul_i64(&mut self, instruction: Instruction);
    fn div_i64(&mut self, instruction: Instruction);
    fn mod_i64(&mut self, instruction: Instruction);
    fn pow_i64(&mut self, instruction: Instruction);
    fn neg_i64(&mut self, instruction: Instruction);

    fn add_f64(&mut self, instruction: Instruction);
    fn sub_f64(&mut self, instruction: Instruction);
    fn mul_f64(&mut self, instruction: Instruction);
    fn div_f64(&mut self, instruction: Instruction);
    fn mod_f64(&mut self, instruction: Instruction);
    fn pow_f64(&mut self, instruction: Instruction);
    fn neg_f64(&mut self, instruction: Instruction);
}

impl VmArithmeticOps for VM<'_> {
    fn add_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_i64_in_register(src1);
        let val2 = self.get_i64_in_register(src2);

        let result = val1 + val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_i64_in_register(dest, result);
    }

    fn sub_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_i64_in_register(src1);
        let val2 = self.get_i64_in_register(src2);

        let result = val1 - val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_i64_in_register(dest, result);
    }

    fn mul_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_i64_in_register(src1);
        let val2 = self.get_i64_in_register(src2);

        let result = val1 * val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_i64_in_register(dest, result);
    }

    fn div_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_i64_in_register(src1);
        let val2 = self.get_i64_in_register(src2);

        let result = val1 / val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_i64_in_register(dest, result);
    }

    fn mod_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_i64_in_register(src1);
        let val2 = self.get_i64_in_register(src2);

        let result = val1 % val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_i64_in_register(dest, result);
    }

    fn pow_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_i64_in_register(src1) as f64;
        let val2 = self.get_i64_in_register(src2) as f64;

        let result = val1.powf(val2) as i64;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_i64_in_register(dest, result);
    }

    fn neg_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let val1 = self.get_i64_in_register(src1);
        let result = -val1;

        let dest = InstructionDecoder::decode_dest(instruction);
        self.set_i64_in_register(dest, result);
    }

    fn add_f64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_f64_in_register(src1);
        let val2 = self.get_f64_in_register(src2);

        let result = val1 + val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_f64_in_register(dest, result);
    }

    fn sub_f64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_f64_in_register(src1);
        let val2 = self.get_f64_in_register(src2);

        let result = val1 - val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_f64_in_register(dest, result);
    }

    fn mul_f64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_f64_in_register(src1);
        let val2 = self.get_f64_in_register(src2);

        let result = val1 * val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_f64_in_register(dest, result);
    }

    fn div_f64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_f64_in_register(src1);
        let val2 = self.get_f64_in_register(src2);

        let result = val1 / val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_f64_in_register(dest, result);
    }

    fn mod_f64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_f64_in_register(src1);
        let val2 = self.get_f64_in_register(src2);

        let result = val1 % val2;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_f64_in_register(dest, result);
    }

    fn pow_f64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_f64_in_register(src1);
        let val2 = self.get_f64_in_register(src2);

        let result = val1.powf(val2);
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_f64_in_register(dest, result);
    }

    fn neg_f64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let val1 = self.get_f64_in_register(src1);

        let result = -val1;
        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_f64_in_register(dest, result);
    }
}
