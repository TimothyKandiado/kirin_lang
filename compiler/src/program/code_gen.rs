use std::collections::HashMap;

use program::{Constant, FunctionKind, FunctionMetadata, Program, TypeInfo};

use crate::{
    instruction::{Instruction, InstructionBuilder, InstructionDecoder, OpCode},
    ir::{Callee, IrBlock, IrFunction, IrInstruction, IrModule},
    parser::{BinaryExprOp, UnaryExprOp, ValueType},
    program::register_allocation::{RegisterAllocation, RegisterAllocator},
};

enum InstrJump {
    Branch {
        instr_idx: usize,
        condition_reg: usize,
        label: usize,
    },
    Jump {
        instr_idx: usize,
        label: usize,
    },
}

struct ProgramBuilder {
    instructions: Vec<Instruction>,
    constants: Vec<Constant>,
    types: Vec<TypeInfo>,
    functions: Vec<FunctionMetadata>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            types: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn build_module(mut self, ir_module: IrModule<'_>) -> Program {
        // load function symbols
        for function in ir_module.functions.iter() {
            match function {
                IrFunction::Bytecode {
                    name,
                    params,
                    ret_type,
                    blocks: _,
                    reg_count: _,
                    reg_types: _,
                } => {
                    let constant = Constant::String(name.to_string());
                    let name_idx = self.push_constant(constant) as u16;

                    let func_metadata = FunctionMetadata {
                        name_idx,
                        code_length: 0,
                        code_offset: 0,
                        function_kind: FunctionKind::Bytecode,
                        registers: 0,
                        parameters: params
                            .iter()
                            .map(|p| p.value_type.get_size())
                            .sum::<usize>() as u8,
                        return_args: ret_type.get_size() as u8,
                    };

                    self.functions.push(func_metadata);
                }
                IrFunction::Native {
                    name,
                    params,
                    ret_type,
                } => {
                    let constant = Constant::String(name.to_string());
                    let name_idx = self.push_constant(constant) as u16;

                    let func_metadata = FunctionMetadata {
                        name_idx,
                        code_length: 0,
                        code_offset: 0,
                        function_kind: FunctionKind::Native,
                        registers: 0,
                        parameters: params
                            .iter()
                            .map(|p| p.value_type.get_size())
                            .sum::<usize>() as u8,
                        return_args: ret_type.get_size() as u8,
                    };

                    self.functions.push(func_metadata);
                }
            }
        }

        for (idx, function) in ir_module.functions.iter().enumerate() {
            self.build_function(idx, function);
        }

        Program {
            instructions: self.instructions,
            constants: self.constants,
            types: self.types,
            functions: self.functions,
        }
    }

    fn build_function(&mut self, func_idx: usize, function: &IrFunction<'_>) {
        let (total_registers, reg_allocations) = RegisterAllocator::allocate_for_function(function);

        let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type,
            blocks,
            reg_count: _,
            reg_types,
        } = function
        else {
            return;
        };

        #[cfg(debug_assertions)]
        {
            println!("=== Register Allocation ===");
            println!("Total registers: {}", total_registers);
            for (idx, alloc) in reg_allocations.iter().enumerate() {
                println!("VReg: [{}] ; Allocation: ({})", idx, alloc)
            }
        }

        // (label, instruction_idx)
        let mut block_map: HashMap<usize, usize> = HashMap::new();
        let mut unresolved_jumps = Vec::new();

        let start_inst = self.instructions.len();

        // build blocks
        for block in blocks.iter() {
            block_map.insert(block.label, self.instructions.len());

            let mut jumps = self.build_block(block, &reg_allocations, reg_types);
            unresolved_jumps.append(&mut jumps);
        }

        if let Some(&last_inst) = self.instructions.last() {
            let opcode = OpCode::from_u32(InstructionDecoder::decode_opcode(last_inst));

            if ret_type == &ValueType::Void && opcode != OpCode::RetVoid {
                self.instructions.push(
                    InstructionBuilder::new()
                        .set_opcode(OpCode::RetVoid)
                        .build(),
                );
            }
        }

        let end_inst = self.instructions.len();

        let func_length = end_inst - start_inst;

        self.functions[func_idx].code_offset = start_inst as u32;
        self.functions[func_idx].code_length = func_length as u16;
        self.functions[func_idx].registers = total_registers as u8;

        // resolve_jumps
        for unresolved_jump in unresolved_jumps {
            match unresolved_jump {
                InstrJump::Branch {
                    instr_idx,
                    condition_reg,
                    label,
                } => {
                    let target_inst = *block_map
                        .get(&label)
                        .expect("undefined block while resolving jump");

                    let offset = target_inst as i64 - instr_idx as i64;

                    let instruction = InstructionBuilder::new()
                        .set_opcode(OpCode::BrFalse)
                        .set_dest(condition_reg as u32)
                        .set_imm19(offset as i32)
                        .build();

                    self.instructions[instr_idx] = instruction;
                }

                InstrJump::Jump { instr_idx, label } => {
                    let target_inst = *block_map
                        .get(&label)
                        .expect("undefined block while resolving jump");

                    let offset = target_inst as i64 - instr_idx as i64;

                    let instruction = if offset > 1 {
                        InstructionBuilder::new()
                            .set_opcode(OpCode::Jump)
                            .set_imm19(offset as i32)
                            .build()
                    } else {
                        InstructionBuilder::new().set_opcode(OpCode::NoOp).build()
                    };

                    self.instructions[instr_idx] = instruction;
                }
            }
        }
    }

    fn build_block(
        &mut self,
        block: &IrBlock<'_>,
        reg_allocations: &[RegisterAllocation],
        reg_types: &[ValueType],
    ) -> Vec<InstrJump> {
        let mut jumps: Vec<InstrJump> = Vec::new();

        for inst in block.instructions.iter() {
            match inst {
                IrInstruction::BinOp {
                    dest,
                    op,
                    lhs,
                    rhs,
                    val_type,
                } => {
                    let dest = reg_allocations[*dest].offset as u32;
                    let src1 = reg_allocations[*lhs].offset as u32;
                    let src2 = reg_allocations[*rhs].offset as u32;

                    match op {
                        BinaryExprOp::Add => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::AddI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Add not implemented for {}", val_type)
                            }
                        }
                        BinaryExprOp::Sub => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::SubI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Sub not implemented for {}", val_type)
                            }
                        }
                        BinaryExprOp::Mul => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::MulI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Mul not implemented for {}", val_type)
                            }
                        }
                        BinaryExprOp::Div => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::DivI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Div not implemented for {}", val_type)
                            }
                        }
                        BinaryExprOp::Mod => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::ModI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Mod not implemented for {}", val_type)
                            }
                        }
                        BinaryExprOp::Pow => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::PowI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Pow not implemented for {}", val_type)
                            }
                        }

                        BinaryExprOp::Less => {
                            let val_type = reg_types[*lhs].clone();
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::CmpLtI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Add not implemented for {}", val_type)
                            }
                        }

                        BinaryExprOp::LessEqual => {
                            let val_type = reg_types[*lhs].clone();
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::CmpLeI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Add not implemented for {}", val_type)
                            }
                        }

                        BinaryExprOp::Greater => {
                            let val_type = reg_types[*lhs].clone();
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::CmpLeI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::Not,
                                    dest,
                                    dest,
                                    0,
                                ));
                            } else {
                                panic!("Add not implemented for {}", val_type)
                            }
                        }

                        BinaryExprOp::GreaterEqual => {
                            let val_type = reg_types[*lhs].clone();
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::CmpLtI64,
                                    dest,
                                    src1,
                                    src2,
                                ));
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::Not,
                                    dest,
                                    dest,
                                    0,
                                ));
                            } else {
                                panic!("GreaterEqual not implemented for {}", val_type)
                            }
                        }

                        BinaryExprOp::Equal => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::CmpEq,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Equal not implemented for {}", val_type)
                            }
                        }

                        BinaryExprOp::NotEqual => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::CmpEq,
                                    dest,
                                    src1,
                                    src2,
                                ));
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::Not,
                                    dest,
                                    dest,
                                    0,
                                ));
                            } else {
                                panic!("Add not implemented for {}", val_type)
                            }
                        }

                        BinaryExprOp::And => {
                            let val_type = reg_types[*lhs].clone();
                            if let ValueType::Bool = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::And,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Add not implemented for {}", val_type)
                            }
                        }

                        BinaryExprOp::Or => {
                            let val_type = reg_types[*lhs].clone();
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::Or,
                                    dest,
                                    src1,
                                    src2,
                                ));
                            } else {
                                panic!("Add not implemented for {}", val_type)
                            }
                        }
                    }
                }

                IrInstruction::UnaryOp {
                    dest,
                    op,
                    rhs,
                    val_type,
                } => {
                    let dest = reg_allocations[*dest].offset as u32;
                    let src1 = reg_allocations[*rhs].offset as u32;

                    match op {
                        UnaryExprOp::Not => {
                            if let ValueType::Bool = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::Not,
                                    dest,
                                    src1,
                                    0,
                                ));
                            } else {
                                panic!("Not not implemented for {}", val_type)
                            }
                        }
                        UnaryExprOp::Neg => {
                            if let ValueType::I64 = val_type {
                                self.instructions.push(InstructionBuilder::new_format_a(
                                    OpCode::NegI64,
                                    dest,
                                    src1,
                                    0,
                                ));
                            } else {
                                panic!("Neg not implemented for {}", val_type)
                            }
                        }
                    }
                }

                IrInstruction::ConstBool { dest, val } => {
                    let dest = reg_allocations[*dest].offset as u32;

                    let opcode = match val {
                        true => OpCode::ConstTrue,
                        false => OpCode::ConstFalse,
                    };
                    self.instructions
                        .push(InstructionBuilder::new_format_a(opcode, dest, 0, 0));
                }

                IrInstruction::ConstI64 { dest, val } => {
                    let dest = reg_allocations[*dest].offset as u32;

                    if *val >= -262144 && *val <= 262143 {
                        self.instructions.push(
                            InstructionBuilder::new()
                                .set_opcode(OpCode::ConstI64Imm)
                                .set_dest(dest)
                                .set_imm19(*val as i32)
                                .build(),
                        );
                    } else {
                        let const_idx = self.push_constant(Constant::I64(*val));
                        self.instructions.push(InstructionBuilder::new_format_c(
                            OpCode::ConstI64,
                            dest,
                            const_idx as u32,
                        ));
                    }
                }

                IrInstruction::ConstF64 { dest, val } => {
                    let dest = reg_allocations[*dest].offset as u32;

                    let const_idx = self.push_constant(Constant::F64(*val));
                    self.instructions.push(InstructionBuilder::new_format_c(
                        OpCode::ConstF64,
                        dest,
                        const_idx as u32,
                    ));
                }

                IrInstruction::ConstStr { dest, val } => {
                    let dest = reg_allocations[*dest].offset as u32;

                    let const_idx = self.push_constant(Constant::String(val.to_string()));
                    self.instructions.push(InstructionBuilder::new_format_c(
                        OpCode::ConstStr,
                        dest,
                        const_idx as u32,
                    ));
                }

                IrInstruction::Copy { dest, source } => {
                    let dest = reg_allocations[*dest].offset as u32;
                    let src1 = reg_allocations[*source].offset as u32;

                    if dest != src1 {
                        self.instructions.push(InstructionBuilder::new_format_a(
                            OpCode::Move,
                            dest,
                            src1,
                            0,
                        ));
                    }
                }

                IrInstruction::Branch {
                    cond,
                    then_label,
                    else_label,
                } => {
                    let cond = reg_allocations[*cond].offset as u32;

                    jumps.push(InstrJump::Branch {
                        instr_idx: self.instructions.len(),
                        condition_reg: cond as usize,
                        label: *else_label,
                    });

                    self.instructions.push(InstructionBuilder::new_format_c(
                        OpCode::BrFalse,
                        cond,
                        0,
                    ));

                    if then_label - 1 != block.label {
                        // incase the then block is not placed immediately after the branch instruction
                        jumps.push(InstrJump::Jump {
                            instr_idx: self.instructions.len(),
                            label: *then_label,
                        });
                        self.instructions
                            .push(InstructionBuilder::new_format_c(OpCode::Jump, 0, 0))
                    }
                }

                IrInstruction::Jump { label } => {
                    jumps.push(InstrJump::Jump {
                        instr_idx: self.instructions.len(),
                        label: *label,
                    });

                    self.instructions
                        .push(InstructionBuilder::new_format_c(OpCode::Jump, 0, 0));
                }

                IrInstruction::Call {
                    dest,
                    callee,
                    args,
                    val_type: _,
                } => {
                    let dest = if let Some(dest) = dest { dest } else { &0 };
                    let dest = reg_allocations[*dest].offset as u32;

                    let param_start = args.first().unwrap_or(&0);
                    let param_start = reg_allocations[*param_start].offset as u32;

                    match callee {
                        Callee::Direct(name) => {
                            let name_idx = self
                                .constants
                                .iter()
                                .position(|c| c == &Constant::String(name.to_string()))
                                .expect("function name not found in constants table");

                            let function_idx = self
                                .functions
                                .iter()
                                .position(|f| f.name_idx == name_idx as u16)
                                .expect("function not found");

                            self.instructions.push(InstructionBuilder::new_format_b(
                                OpCode::Call,
                                dest,
                                param_start,
                                function_idx as u32,
                            ));
                        }
                        Callee::Indirect(reg) => {
                            let callee = reg_allocations[*reg].offset;

                            self.instructions.push(InstructionBuilder::new_format_b(
                                OpCode::Invoke,
                                dest,
                                param_start,
                                callee as u32,
                            ));
                        }
                    }
                }

                IrInstruction::LoadGlobal { dest: _, name: _ } => {
                    panic!("LoadGlobal not implemented")
                }

                IrInstruction::Return { val } => {
                    if let Some(val) = val {
                        let src1 = reg_allocations[*val].offset as u32;

                        self.instructions.push(InstructionBuilder::new_format_c(
                            OpCode::Ret,
                            0,
                            src1,
                        ));
                    }
                }
            }
        }

        jumps
    }

    fn push_constant(&mut self, new: Constant) -> usize {
        let existing = self.constants.iter().position(|constant| constant == &new);

        if let Some(index) = existing {
            index
        } else {
            self.constants.push(new);
            self.constants.len() - 1
        }
    }
}

pub fn build_program<'a>(ir_module: IrModule<'a>) -> Program {
    ProgramBuilder::new().build_module(ir_module)
}
