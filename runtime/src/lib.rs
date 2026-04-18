pub mod native;

use program::{
    Constant, FunctionKind, FunctionMetadata, Instruction, InstructionBuilder, InstructionDecoder, Program, TypeInfo, opcode::*
};

#[cfg(debug_assertions)]
use program::debug_print_instruction;

use crate::native::NativeFunctionWrapper;

pub type Register = u64;

const FRAME_HEADER_LENGTH: Register = 3;


#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
struct FrameHeaderFlags {
    pub return_register: u8,
    pub frame_size: u8,
    pub function_index: u16,
}

impl FrameHeaderFlags {
    pub fn new(return_register: u8, frame_size: u8, function_index: u16) -> Self {
        Self {
            return_register,
            frame_size,
            function_index,
        }
    }

    #[inline(always)]
    pub fn to_register(self) -> u64 {
        (self.return_register as u64)
            | ((self.frame_size as u64) << 8)
            | ((self.function_index as u64) << 16)
    }

    #[inline(always)]
    pub fn from_register(reg: u64) -> Self {
        Self {
            return_register: (reg & 0xFF) as u8,
            frame_size: ((reg >> 8) & 0xFF) as u8,
            function_index: ((reg >> 16) & 0xFFFF) as u16,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FrameHeader {
    pub return_address: Register,
    pub prev_frame_ptr: Register,
    pub flags: FrameHeaderFlags,
}

pub struct VmContext<'a> {
    pub constants: &'a [Constant],
}

#[derive(Debug, Clone)]
pub struct VmError {
    pub message: String,
}

pub struct VM<'a> {
    registers: Vec<Register>,
    instruction_ptr: usize,
    frame_ptr: usize,
    is_running: bool,

    instructions: &'a [Instruction],
    functions: &'a [FunctionMetadata],
    constants: &'a [Constant],
    #[allow(unused)]
    types: &'a [TypeInfo],

    native_functions: &'a [NativeFunctionWrapper],
}

impl<'a> VM<'a> {
    pub fn new(program: &'a Program, native_functions: &'a [NativeFunctionWrapper]) -> Self {
        VM {
            registers: Vec::new(),
            instruction_ptr: 0,
            frame_ptr: 0,
            is_running: false,

            instructions: &program.instructions,
            functions: &program.functions,
            constants: &program.constants,
            types: &program.types,

            native_functions,
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        // find entry point
        let main_func_index = self.functions.iter().position(|f| {
            if f.function_kind != FunctionKind::Bytecode {
                return false;
            }

            let name_const = &self.constants[f.name_idx as usize];

            if let Constant::String(str) = name_const {
                return str == "main";
            }

            false
        });

        if main_func_index.is_none() {
            return Err("main function does not exist".to_string());
        }

        let main_func_index = main_func_index.unwrap();

        let main_func = &self.functions[main_func_index];

        if main_func.parameters > 0 {
            return Err("main function should have zero parameters".to_string());
        }

        let call_main_instruction =
            InstructionBuilder::new_format_b(OP_CALL, 0, 0, main_func_index as u32);

        self.is_running = true;

        self.execute(call_main_instruction)?;

        while self.is_running {
            let instruction = self.get_next_instruction();

            #[cfg(debug_assertions)]
            {
                print!("[{}] ", self.instruction_ptr - 1);
                debug_print_instruction(instruction);
            }

            self.execute(instruction)?;
        }

        Ok(())
    }

    fn execute(&mut self, instruction: Instruction) -> Result<(), String> {
        let opcode = InstructionDecoder::decode_opcode(instruction) as u8;

        match opcode {
            OP_NO_OP => {}

            OP_CONST_I64_IMM => self.const_i64_imm(instruction),
            OP_CONST_STR => self.const_str(instruction),

            OP_MOVE => self.move_inst(instruction),

            OP_ADD_I64 => self.add_i64(instruction),
            OP_SUB_I64 => self.sub_i64(instruction),
            OP_MUL_I64 => self.mul_i64(instruction),
            OP_DIV_I64 => self.div_i64(instruction),
            OP_MOD_I64 => self.mod_i64(instruction),
            OP_POW_I64 => self.pow_i64(instruction),
            OP_NEG_I64 => self.neg_i64(instruction),

            OP_CMP_LE_I64 => self.cmp_le_i64(instruction),
            OP_CMP_LT_I64 => self.cmp_lt_i64(instruction),

            OP_NOT => self.not(instruction),

            OP_BR_FALSE => self.br_false(instruction),
            OP_JUMP => self.jump(instruction),

            OP_CALL => self.call(instruction),

            OP_RET_VOID => self.ret_void(),
            OP_RET => self.ret(instruction),

            _ => return Err(format!("unknown opcode {} | {:x} ", opcode, opcode)),
        }

        Ok(())
    }

    fn const_i64_imm(&mut self, instruction: Instruction) {
        let dest = InstructionDecoder::decode_dest(instruction);
        let val = InstructionDecoder::decode_imm19(instruction);

        self.set_i64_in_register(dest, val as i64);
    }

    fn const_str(&mut self, instruction: Instruction) {
        let dest = InstructionDecoder::decode_dest(instruction);
        let index = InstructionDecoder::decode_const19(instruction);

        self.set_register(dest, index as Register);
    }

    fn move_inst(&mut self, instruction: Instruction) {
        let source = InstructionDecoder::decode_src1(instruction);
        let dest = InstructionDecoder::decode_dest(instruction);

        let source_value = self.get_register(source);
        self.set_register(dest, source_value);
    }

    // arithmetic
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


    // comparisons
    fn cmp_le_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_i64_in_register(src1);
        let val2 = self.get_i64_in_register(src2);

        let result = val1 <= val2;

        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_register(dest, result as Register);
    }

    fn cmp_lt_i64(&mut self, instruction: Instruction) {
        let src1 = InstructionDecoder::decode_src1(instruction);
        let src2 = InstructionDecoder::decode_src2(instruction);

        let val1 = self.get_i64_in_register(src1);
        let val2 = self.get_i64_in_register(src2);

        let result = val1 < val2;

        let dest = InstructionDecoder::decode_dest(instruction);

        self.set_register(dest, result as Register);
    }

    fn not(&mut self, instruction: Instruction) {
        let src = InstructionDecoder::decode_src1(instruction);
        let dest = InstructionDecoder::decode_dest(instruction);

        let val = self.get_register(src);
        self.set_register(dest, val ^ 1);
    }

    fn br_false(&mut self, instruction: Instruction) {
        let dest = InstructionDecoder::decode_dest(instruction);
        let condition = self.get_register(dest);

        if condition == 1 {
            return;
        }

        self.jump(instruction);
    }

    fn jump(&mut self, instruction: Instruction) {
        let imm = InstructionDecoder::decode_imm19(instruction);

        if imm > 0 {
            self.instruction_ptr += imm as usize - 1;
        } else {
            self.instruction_ptr -= imm as usize;
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
                    func_index as u16);

                let frame = self.push_frame(
                    function_metadata.code_offset as usize,
                    flags
                );

                let arg_start = frame.prev_frame_ptr as usize + FRAME_HEADER_LENGTH as usize + arg_start as usize;

                let param_start = self.frame_ptr + FRAME_HEADER_LENGTH as usize;
                
                self.registers.copy_within(arg_start..arg_start+(function_metadata.parameters as usize), param_start);
            }
            FunctionKind::Native => {
                let native_func = &self.native_functions[function_metadata.code_offset as usize];

                let arg_start = self.frame_ptr + FRAME_HEADER_LENGTH as usize + arg_start as usize;

                let args =
                    &self.registers[arg_start..(arg_start + function_metadata.parameters as usize)];

                let mut return_slots: Vec<Register> = vec![0; function_metadata.return_args as usize];

                let mut ctx = VmContext {
                    constants: self.constants,
                };

                let result = (native_func.function)(&mut ctx, args, &mut return_slots);

                match result {
                    Ok(_) => {
                        if !return_slots.is_empty() {
                            let ret_start =
                                self.frame_ptr + FRAME_HEADER_LENGTH as usize + ret_start as usize;

                            self.registers
                                [ret_start..(ret_start + function_metadata.return_args as usize)]
                                .copy_from_slice(&return_slots);
                        }
                    }
                    Err(err) => {
                        println!(
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
        
        let ret_dest_start = frame_header.prev_frame_ptr + FRAME_HEADER_LENGTH + frame_header.flags.return_register as u64;

        self.registers.copy_within(ret_source_start..(ret_source_start+function.return_args as usize), ret_dest_start as usize);

        _ = self.pop_frame();
    }

    fn push_frame(
        &mut self,
        target_instruction: usize,
        mut flags: FrameHeaderFlags
    ) -> FrameHeader {
        flags.frame_size += FRAME_HEADER_LENGTH as u8;

        let frame_header = FrameHeader {
            prev_frame_ptr: self.frame_ptr as Register,
            return_address: self.instruction_ptr as Register,
            flags,
        };

        let current_registers = self.registers.len();
        let target_registers = current_registers + frame_header.flags.frame_size as usize;

        self.instruction_ptr = target_instruction;
        self.frame_ptr = self.registers.len();

        self.registers.resize(target_registers, 0);

        self.registers[self.frame_ptr] = frame_header.return_address;
        self.registers[self.frame_ptr + 1] = frame_header.prev_frame_ptr;
        self.registers[self.frame_ptr + 2] = frame_header.flags.to_register();

        frame_header
    }

    fn pop_frame(&mut self) -> FrameHeader {
        let header = self.get_frame_header();

        self.instruction_ptr = header.return_address as usize;
        self.frame_ptr = header.prev_frame_ptr as usize;

        let current_registers = self.registers.len();
        let target_registers = current_registers - header.flags.frame_size as usize;

        self.registers.resize(target_registers, 0);

        
        if self.registers.len() < FRAME_HEADER_LENGTH as usize {
            self.is_running = false;
        }

        header
    }

    fn get_frame_header(&self) -> FrameHeader {
        let return_address = self.registers[self.frame_ptr];
        let prev_frame_ptr = self.registers[self.frame_ptr + 1];
        let flags = self.registers[self.frame_ptr + 2];

        let flags = FrameHeaderFlags::from_register(flags);

        FrameHeader {
            return_address,
            prev_frame_ptr,
            flags
        }
    }

    #[inline]
    fn set_i64_in_register(&mut self, index: Instruction, value: i64) {
        self.set_register(index, value as Register);
    }

    #[inline]
    fn get_i64_in_register(&mut self, index: Instruction) -> i64 {
        self.get_register(index) as i64
    }

    #[inline]
    fn get_register(&self, index: Instruction) -> Register {
        self.registers[self.frame_ptr + FRAME_HEADER_LENGTH as usize + index as usize]
    }

    #[inline]
    fn set_register(&mut self, index: Instruction, value: Register) {
        self.registers[self.frame_ptr + FRAME_HEADER_LENGTH as usize + index as usize] = value;
    }

    #[inline]
    fn get_next_instruction(&mut self) -> Instruction {
        self.instruction_ptr += 1;

        self.instructions[self.instruction_ptr - 1]
    }
}

#[cfg(test)]
mod test {
    use crate::FrameHeaderFlags;

    #[test]
    fn test_frame_header_flags_encoding() {
        let original_flags = FrameHeaderFlags::new(2, 20, 44);

        let encoded_flags = original_flags.to_register();

        let decoded_flags = FrameHeaderFlags::from_register(encoded_flags);

        assert_eq!(original_flags, decoded_flags)
    }
}