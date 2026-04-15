use program::Program;

use crate::{
    instruction::{Instruction, InstructionDecoder, OpCode},
};

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
        let opcode = OpCode::from_u32(InstructionDecoder::decode_opcode(instruction));

        print!("[{}] ", idx);

        match opcode {
            OpCode::AddI64
            | OpCode::SubI64
            | OpCode::MulI64
            | OpCode::DivI64
            | OpCode::ModI64
            | OpCode::PowI64
            | OpCode::CmpEq
            | OpCode::CmpLeI64
            | OpCode::CmpLtI64
            | OpCode::And
            | OpCode::Not
            | OpCode::NegI64 => {
                print_format_a(instruction);
            }
            OpCode::NoOp => println!("NoOp"),
            OpCode::ConstI64Imm => {
                let imm = InstructionDecoder::decode_imm19(instruction);
                let dest = InstructionDecoder::decode_dest(instruction);

                println!("ConstI64Imm {} | {}", dest, imm);
            }
            OpCode::ConstI64 => print_format_c(instruction),
            OpCode::ConstF64 => print_format_c(instruction),
            OpCode::ConstTrue => print_format_c(instruction),
            OpCode::ConstFalse => print_format_c(instruction),
            OpCode::ConstStr => print_format_c(instruction),
            OpCode::Move => print_format_a(instruction),
            OpCode::Swap => print_format_a(instruction),
            OpCode::Or => print_format_a(instruction),
            OpCode::BrFalse => {
                let offset = InstructionDecoder::decode_imm19(instruction);
                let cond = InstructionDecoder::decode_dest(instruction);

                println!("BrFalse {} | {}", cond, offset);
            }
            OpCode::Jump => {
                let offset = InstructionDecoder::decode_imm19(instruction);

                println!("Jump {}", offset);
            }
            OpCode::Call => print_format_b(instruction),
            OpCode::Invoke => print_format_b(instruction),
            OpCode::Ret => print_format_c(instruction),
            OpCode::RetVoid => println!("RetVoid"),
            OpCode::Halt => println!("Halt"),
        }
    }
}

fn print_format_a(instruction: Instruction) {
    let opcode = OpCode::from_u32(InstructionDecoder::decode_opcode(instruction));

    let dest = InstructionDecoder::decode_dest(instruction);
    let src1 = InstructionDecoder::decode_src1(instruction);
    let src2 = InstructionDecoder::decode_src2(instruction);

    println!("{:?} | {} | {} | {}", opcode, dest, src1, src2)
}

fn print_format_b(instruction: Instruction) {
    let opcode = OpCode::from_u32(InstructionDecoder::decode_opcode(instruction));

    let dest = InstructionDecoder::decode_dest(instruction);
    let src1 = InstructionDecoder::decode_src1(instruction);
    let const13 = InstructionDecoder::decode_const13(instruction);

    println!("{:?} | {} | {} | {}", opcode, dest, src1, const13)
}

fn print_format_c(instruction: Instruction) {
    let opcode = OpCode::from_u32(InstructionDecoder::decode_opcode(instruction));

    let dest = InstructionDecoder::decode_dest(instruction);
    let const19 = InstructionDecoder::decode_const19(instruction);

    println!("{:?} | {} | {}", opcode, dest, const19);
}
