use program::{FunctionKind, Instruction, InstructionDecoder};

use crate::vm::{FRAME_HEADER_LENGTH, FrameHeaderFlags, VM, VmContext};

pub trait VmControlOps {
    fn br_false(&mut self, instruction: Instruction);
    fn jump(&mut self, instruction: Instruction);
    fn call(&mut self, instruction: Instruction);
    fn ret_void(&mut self);
    fn ret(&mut self, instruction: Instruction);
}

impl VmControlOps for VM<'_> {
    fn br_false(&mut self, instruction: Instruction) {
        let dest = InstructionDecoder::decode_dest(instruction);
        let condition = self.get_register(dest);

        if condition == 1 {
            return;
        }

        self.jump(instruction);
    }

    fn jump(&mut self, instruction: Instruction) {
        let imm = InstructionDecoder::decode_imm19(instruction) as i64;

        if imm > 0 {
            self.instruction_ptr += imm as usize - 1;
        } else {
            self.instruction_ptr -= imm.unsigned_abs() as usize + 1;
        }
    }

    fn call(&mut self, instruction: Instruction) {
        let func_index = InstructionDecoder::decode_const13(instruction);

        let function_metadata = &self.functions[func_index as usize];

        let arg_start = InstructionDecoder::decode_src1(instruction);
        let ret_start = InstructionDecoder::decode_dest(instruction);

        match function_metadata.function_kind {
            FunctionKind::Bytecode => {
                let flags = FrameHeaderFlags::new(
                    ret_start as u8,
                    function_metadata.registers,
                    func_index as u16,
                );

                let frame = self.push_frame(function_metadata.code_offset as usize, flags);

                let arg_start = frame.prev_frame_ptr as usize
                    + FRAME_HEADER_LENGTH as usize
                    + arg_start as usize;

                let param_start = self.frame_ptr + FRAME_HEADER_LENGTH as usize;

                self.registers.copy_within(
                    arg_start..arg_start + (function_metadata.parameters as usize),
                    param_start,
                );
            }
            FunctionKind::Native => {
                let native_func = &self.native_functions[function_metadata.code_offset as usize];

                let arg_start = self.frame_ptr + FRAME_HEADER_LENGTH as usize + arg_start as usize;

                let ret_start = self.frame_ptr + FRAME_HEADER_LENGTH as usize + ret_start as usize;

                let (argument_slots, return_slots) = if ret_start
                    >= arg_start + function_metadata.parameters as usize
                {
                    let (arg_slice, ret_slice) = self.registers.split_at_mut(ret_start);

                    let argument_slots =
                        &arg_slice[arg_start..(arg_start + function_metadata.parameters as usize)];
                    let return_slots = &mut ret_slice[0..function_metadata.return_args as usize];

                    (argument_slots, return_slots)
                } else {
                    let (ret_slice, arg_slice) = self.registers.split_at_mut(arg_start);

                    let return_slots = &mut ret_slice
                        [ret_start..(ret_start + function_metadata.return_args as usize)];
                    let argument_slots = &arg_slice[0..function_metadata.parameters as usize];

                    (argument_slots, return_slots)
                };

                let mut ctx = VmContext {
                    constants: self.constants,
                    types: self.types,
                };

                let result = (native_func.function)(&mut ctx, argument_slots, return_slots);

                match result {
                    Ok(_) => {}

                    Err(err) => {
                        panic!(
                            "Error while executing native func: {} : \n{:?}",
                            native_func.name, err
                        );
                    }
                }
            }
        }
    }

    fn ret_void(&mut self) {
        _ = self.pop_frame();
    }

    fn ret(&mut self, instruction: Instruction) {
        let frame_header = self.get_frame_header();

        let function = &self.functions[frame_header.flags.function_index as usize];

        let ret_source_start = InstructionDecoder::decode_const19(instruction) as usize;

        let ret_source_start = self.frame_ptr + FRAME_HEADER_LENGTH as usize + ret_source_start;

        let ret_dest_start = frame_header.prev_frame_ptr
            + FRAME_HEADER_LENGTH
            + frame_header.flags.return_register as u64;

        self.registers.copy_within(
            ret_source_start..(ret_source_start + function.return_args as usize),
            ret_dest_start as usize,
        );

        _ = self.pop_frame();
    }
}
