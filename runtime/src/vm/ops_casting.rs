use program::{Instruction, InstructionDecoder};

use crate::{Register, vm::VM};

pub trait VmCastingOps {
    fn i64_to_f64(&mut self, instruction: Instruction);
    fn f64_to_i64(&mut self, instruction: Instruction);
    fn box_value(&mut self, instruction: Instruction);
    fn unbox_value(&mut self, instruction: Instruction);
}

impl VmCastingOps for VM<'_> {
    fn i64_to_f64(&mut self, instruction: Instruction) {
        let src = InstructionDecoder::decode_src1(instruction);
        let value = self.get_i64_in_register(src);

        let result = value as f64;

        let dest = InstructionDecoder::decode_dest(instruction);
        self.set_f64_in_register(dest, result);
    }

    fn f64_to_i64(&mut self, instruction: Instruction) {
        let src = InstructionDecoder::decode_src1(instruction);
        let value = self.get_f64_in_register(src);

        let result = value as i64;

        let dest = InstructionDecoder::decode_dest(instruction);
        self.set_i64_in_register(dest, result);
    }

    fn box_value(&mut self, instruction: Instruction) {
        let src = InstructionDecoder::decode_src1(instruction);
        let type_index = InstructionDecoder::decode_const13(instruction);

        let value = self.get_register(src);

        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_register(dest, type_index as Register);
        self.set_register(dest + 1, value);
    }

    fn unbox_value(&mut self, instruction: Instruction) {
        todo!()
    }
}
