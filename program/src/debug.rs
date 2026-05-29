use std::sync::LazyLock;

use crate::opcode::OP_ADD_I64;
use crate::{Instruction, InstructionDecoder, Program};

use crate::opcode::*;

pub fn debug_program(program: &Program) {
    println!("==== Program ====");
    println!("=== Constants ===");
    for (idx, constant) in program.constants.iter().enumerate() {
        println!("[{}] => {:?}", idx, constant)
    }

    println!("=== Functions ===");
    for (idx, func) in program.functions.iter().enumerate() {
        println!("[{}] => {:?}", idx, func)
    }

    println!("=== Instructions ===");
    for (idx, &instruction) in program.instructions.iter().enumerate() {
        print!("[{}] ", idx);

        debug_print_instruction(instruction);
    }
    println!("===================");
}

static FORMAT_A_OPS : LazyLock<Vec<u8>> = LazyLock::new(|| {
    vec![
        OP_ADD_I64 , OP_SUB_I64 , OP_MUL_I64 , OP_DIV_I64 , OP_MOD_I64 , OP_POW_I64, 
        OP_ADD_F64 , OP_SUB_F64 , OP_MUL_F64 , OP_DIV_F64 , OP_MOD_F64 , OP_POW_F64,
        OP_CMP_LE_I64, OP_CMP_LT_I64, OP_CMP_LE_F64, OP_CMP_LT_F64, OP_CMP_EQ
    ]
});

static FORMAT_C_OPS: LazyLock<Vec<u8>> = LazyLock::new(|| {
    vec![
        OP_CONST_I64, OP_CONST_I64_IMM, OP_CONST_F64, OP_CONST_FALSE, OP_CONST_TRUE, OP_CONST_STR
    ]
});

pub fn debug_print_instruction(instruction: Instruction) {
    let opcode = InstructionDecoder::decode_opcode(instruction) as u8;
    match opcode {
        OP_CONST_I64_IMM => {
            let imm = InstructionDecoder::decode_imm19(instruction);
            let dest = InstructionDecoder::decode_dest(instruction);

            println!("OP_CONST_I64_IMM {} | {}", dest, imm);
        }

        OP_MOVE | OP_SWAP => {
            let dest = InstructionDecoder::decode_dest(instruction);
            let src = InstructionDecoder::decode_src1(instruction);

            let opcode = opcode_name(opcode);

            println!("{} dest:{} | src:{}", opcode, dest, src)
        },

        OP_BR_FALSE => {
            let offset = InstructionDecoder::decode_imm19(instruction);
            let cond = InstructionDecoder::decode_dest(instruction);

            println!("OP_BR_FALSE cond:{} | off:{}", cond, offset);
        }
        OP_JUMP => {
            let offset = InstructionDecoder::decode_imm19(instruction);

            println!("OP_JUMP off:{}", offset);
        }

        OP_CALL | OP_INVOKE => print_call(instruction),

        x if FORMAT_A_OPS.contains(&x) => {
            print_format_a(instruction);
        }
        
        
        x if FORMAT_C_OPS.contains(&x) => print_format_c(instruction),

        OP_F64_TO_I64 | OP_I64_TO_F64 => {
            let opcode = opcode_name(opcode);

            let dest = InstructionDecoder::decode_dest(instruction);
            let src = InstructionDecoder::decode_src1(instruction);

            println!("{}  dest:{} | src:{}", opcode, dest, src)
        }

        OP_BOX | OP_UNBOX => {
            print_box_op(instruction)
        }

        OP_RET => {
            let opcode = opcode_name(opcode);

            let const19 = InstructionDecoder::decode_const19(instruction);

            println!("{}  {}", opcode, const19);
        },

        _ => {
            let opcode = opcode_name(opcode);
            println!("{}", opcode)
        },
    }
}

fn print_format_a(instruction: Instruction) {
    let opcode = InstructionDecoder::decode_opcode(instruction) as u8;
    let opcode = opcode_name(opcode);

    let dest = InstructionDecoder::decode_dest(instruction);
    let src1 = InstructionDecoder::decode_src1(instruction);
    let src2 = InstructionDecoder::decode_src2(instruction);

    println!("{}  dest:{} | src1:{} | src2:{}", opcode, dest, src1, src2)
}

fn print_box_op(instruction: Instruction) {
    let opcode = InstructionDecoder::decode_opcode(instruction) as u8;
    let opcode = opcode_name(opcode);

    let dest = InstructionDecoder::decode_dest(instruction);
    let src1 = InstructionDecoder::decode_src1(instruction);
    let const13 = InstructionDecoder::decode_const13(instruction);

    println!("{}  dest:{} | src:{} | type:{}", opcode, dest, src1, const13)
}

fn print_call(instruction: Instruction) {
    let opcode = InstructionDecoder::decode_opcode(instruction) as u8;
    let opcode = opcode_name(opcode);

    let dest = InstructionDecoder::decode_dest(instruction);
    let src1 = InstructionDecoder::decode_src1(instruction);
    let const13 = InstructionDecoder::decode_const13(instruction);

    println!("{}  ret:{} | params:{} | idx:{}", opcode, dest, src1, const13)
}

fn print_format_c(instruction: Instruction) {
    let opcode = InstructionDecoder::decode_opcode(instruction) as u8;
    let opcode = opcode_name(opcode);

    let dest = InstructionDecoder::decode_dest(instruction);
    let const19 = InstructionDecoder::decode_const19(instruction);

    println!("{} | {} | {}", opcode, dest, const19);
}
