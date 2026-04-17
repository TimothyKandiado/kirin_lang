use crate::opcode::OP_ADD_I64;
use crate::{Instruction, InstructionDecoder, Program};

use crate::opcode::*;

pub fn debug_program(program: &Program) {
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
        let opcode = InstructionDecoder::decode_opcode(instruction) as u8;

        print!("[{}] ", idx);

        match opcode {
            OP_ADD_I64
            | OP_SUB_I64
            | OP_MUL_I64
            | OP_DIV_I64
            | OP_MOD_I64
            | OP_POW_I64
            | OP_CMP_EQ
            | OP_CMP_LE_I64
            | OP_CMP_LT_I64
            | OP_AND
            | OP_OR
            | OP_NOT
            | OP_NEG_I64 => {
                print_format_a(instruction);
            }
            OP_NO_OP => println!("OP_NO_OP"),
            OP_CONST_I64_IMM => {
                let imm = InstructionDecoder::decode_imm19(instruction);
                let dest = InstructionDecoder::decode_dest(instruction);

                println!("OP_CONST_I64_IMM {} | {}", dest, imm);
            }
            OP_CONST_I64 => print_format_c(instruction),
            OP_CONST_F64 => print_format_c(instruction),
            OP_CONST_TRUE => print_format_c(instruction),
            OP_CONST_FALSE => print_format_c(instruction),
            OP_CONST_STR => print_format_c(instruction),
            OP_MOVE => print_format_a(instruction),
            OP_SWAP => print_format_a(instruction),
            OP_BR_FALSE => {
                let offset = InstructionDecoder::decode_imm19(instruction);
                let cond = InstructionDecoder::decode_dest(instruction);

                println!("OP_BR_FALSE {} | {}", cond, offset);
            }
            OP_JUMP => {
                let offset = InstructionDecoder::decode_imm19(instruction);

                println!("OP_JUMP {}", offset);
            }
            OP_CALL => print_format_b(instruction),
            OP_INVOKE => print_format_b(instruction),
            OP_RET => print_format_c(instruction),
            OP_RET_VOID => println!("OP_RET_VOID"),
            OP_HALT => println!("OP_HALT"),

            _ => println!("unknown opcode {} | {:x} | {:b}", opcode, opcode, opcode)
        }
    }
}

fn print_format_a(instruction: Instruction) {
    let opcode = InstructionDecoder::decode_opcode(instruction) as u8;
    let opcode = opcode_name(opcode);

    let dest = InstructionDecoder::decode_dest(instruction);
    let src1 = InstructionDecoder::decode_src1(instruction);
    let src2 = InstructionDecoder::decode_src2(instruction);

    println!("{} | {} | {} | {}", opcode, dest, src1, src2)
}

fn print_format_b(instruction: Instruction) {
    let opcode = InstructionDecoder::decode_opcode(instruction) as u8;
    let opcode = opcode_name(opcode);

    let dest = InstructionDecoder::decode_dest(instruction);
    let src1 = InstructionDecoder::decode_src1(instruction);
    let const13 = InstructionDecoder::decode_const13(instruction);

    println!("{} | {} | {} | {}", opcode, dest, src1, const13)
}

fn print_format_c(instruction: Instruction) {
    let opcode = InstructionDecoder::decode_opcode(instruction) as u8;
    let opcode = opcode_name(opcode);

    let dest = InstructionDecoder::decode_dest(instruction);
    let const19 = InstructionDecoder::decode_const19(instruction);

    println!("{} | {} | {}", opcode, dest, const19);
}
