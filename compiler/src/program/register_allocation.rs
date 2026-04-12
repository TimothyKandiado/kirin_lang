use crate::ir::{Callee, IrFunction, IrInstruction};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Default)]
struct IrInstCoord {
    pub block_id: usize,
    pub inst_id: usize,
}

#[derive(Debug, Clone, Copy)]
struct VirtualRegister {
    id: usize,
    size: usize,
    start_use: IrInstCoord,
    end_use: IrInstCoord,
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterAllocation {
    pub offset: usize,
    pub size: usize,
}

pub struct RegisterAllocator {
    /// (vreg, alloc, end)
    active: Vec<(usize, RegisterAllocation, IrInstCoord)>,

    /// (offset, size)
    free_list: Vec<(usize, usize)>,
    next_offset: usize,
}

impl RegisterAllocator {
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
            free_list: Vec::new(),
            next_offset: 0,
        }
    }

    fn expire_old_intervals(&mut self, current_start: IrInstCoord) {
        self.active.retain(|(_vreg, alloc, end)| {
            if *end < current_start {
                self.free_list.push((alloc.offset, alloc.size));
                false
            } else {
                true
            }
        });
    }

    fn get_total_registers(&self) -> usize {
        let mut total = 0usize;

        for (_, alloc, _) in self.active.iter() {
            total += alloc.size;
        }

        for (_, size) in self.free_list.iter() {
            total += *size;
        }

        total
    }

    fn allocate(&mut self, size: usize) -> RegisterAllocation {
        // try to reuse a free block
        if let Some(index) = self.free_list.iter().position(|&(_, s)| s >= size) {
            let (offset, block_size) = self.free_list.remove(index);

            if block_size > size {
                self.free_list.push((offset + size, block_size - size));
            }

            return RegisterAllocation { offset, size };
        }

        // otherwise grow
        let alloc = RegisterAllocation {
            offset: self.next_offset,
            size,
        };

        self.next_offset += size;
        alloc
    }

    /// (total, allocations)
    fn run_allocation(values: &mut [VirtualRegister]) -> (usize, Vec<RegisterAllocation>) {
        let mut alloc = RegisterAllocator::new();

        let mut result = vec![RegisterAllocation { offset: 0, size: 0 }; values.len()];

        values.sort_by_key(|v| v.start_use);

        for v in values.iter() {
            alloc.expire_old_intervals(v.start_use);

            let a = alloc.allocate(v.size);

            alloc.active.push((v.id, a, v.end_use));
            result[v.id] = a;
        }

        (alloc.get_total_registers(), result)
    }

    /// (total, allocations)
    pub fn allocate_for_function(function: &IrFunction<'_>) -> (usize, Vec<RegisterAllocation>) {
        let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type: _,
            blocks,
            reg_count: _,
            reg_types,
        } = function
        else {
            return (0, Vec::new());
        };

        let mut virtual_registers: Vec<VirtualRegister> = reg_types
            .iter()
            .enumerate()
            .map(|(idx, reg_type)| VirtualRegister {
                id: idx,
                size: reg_type.get_size(),
                start_use: IrInstCoord::default(),
                end_use: IrInstCoord::default(),
            })
            .collect();

        let mut started_virtual_registers: Vec<usize> = Vec::new();

        for (block_idx, block) in blocks.iter().enumerate() {
            for (inst_idx, inst) in block.instructions.iter().enumerate() {
                let ir_coord = IrInstCoord {
                    block_id: block_idx,
                    inst_id: inst_idx,
                };

                match inst {
                    IrInstruction::BinOp {
                        dest,
                        op: _,
                        lhs,
                        rhs,
                        val_type: _,
                    } => {
                        if started_virtual_registers.contains(dest) {
                            virtual_registers[*dest].end_use = ir_coord;
                        } else {
                            virtual_registers[*dest].start_use = ir_coord;
                            started_virtual_registers.push(*dest);
                        }

                        if started_virtual_registers.contains(lhs) {
                            virtual_registers[*lhs].end_use = ir_coord;
                        } else {
                            virtual_registers[*lhs].start_use = ir_coord;
                            started_virtual_registers.push(*lhs);
                        }

                        if started_virtual_registers.contains(rhs) {
                            virtual_registers[*rhs].end_use = ir_coord;
                        } else {
                            virtual_registers[*rhs].start_use = ir_coord;
                            started_virtual_registers.push(*lhs);
                        }
                    }
                    IrInstruction::UnaryOp {
                        dest,
                        op: _,
                        rhs,
                        val_type: _,
                    } => {
                        if started_virtual_registers.contains(dest) {
                            virtual_registers[*dest].end_use = ir_coord;
                        } else {
                            virtual_registers[*dest].start_use = ir_coord;
                            started_virtual_registers.push(*dest);
                        }

                        if started_virtual_registers.contains(rhs) {
                            virtual_registers[*rhs].end_use = ir_coord;
                        } else {
                            virtual_registers[*rhs].start_use = ir_coord;
                            started_virtual_registers.push(*rhs);
                        }
                    }
                    IrInstruction::Branch {
                        cond,
                        then_label: _,
                        else_label: _,
                    } => {
                        if started_virtual_registers.contains(cond) {
                            virtual_registers[*cond].end_use = ir_coord;
                        } else {
                            virtual_registers[*cond].start_use = ir_coord;
                            started_virtual_registers.push(*cond);
                        }
                    }
                    IrInstruction::Call {
                        dest,
                        callee,
                        args,
                        val_type: _,
                    } => {
                        if let Some(dest) = dest {
                            if started_virtual_registers.contains(dest) {
                                virtual_registers[*dest].end_use = ir_coord;
                            } else {
                                virtual_registers[*dest].start_use = ir_coord;
                                started_virtual_registers.push(*dest);
                            }
                        }

                        if let Callee::Indirect(callee) = callee {
                            if started_virtual_registers.contains(callee) {
                                virtual_registers[*callee].end_use = ir_coord;
                            } else {
                                virtual_registers[*callee].start_use = ir_coord;
                                started_virtual_registers.push(*callee);
                            }
                        }

                        for arg in args {
                            if started_virtual_registers.contains(arg) {
                                virtual_registers[*arg].end_use = ir_coord;
                            } else {
                                virtual_registers[*arg].start_use = ir_coord;
                                started_virtual_registers.push(*arg);
                            }
                        }
                    }
                    IrInstruction::ConstBool { dest, val: _ } => {
                        if started_virtual_registers.contains(dest) {
                            virtual_registers[*dest].end_use = ir_coord;
                        } else {
                            virtual_registers[*dest].start_use = ir_coord;
                            started_virtual_registers.push(*dest);
                        }
                    }
                    IrInstruction::ConstI64 { dest, val: _ } => {
                        if started_virtual_registers.contains(dest) {
                            virtual_registers[*dest].end_use = ir_coord;
                        } else {
                            virtual_registers[*dest].start_use = ir_coord;
                            started_virtual_registers.push(*dest);
                        }
                    }
                    IrInstruction::ConstF64 { dest, val: _ } => {
                        if started_virtual_registers.contains(dest) {
                            virtual_registers[*dest].end_use = ir_coord;
                        } else {
                            virtual_registers[*dest].start_use = ir_coord;
                            started_virtual_registers.push(*dest);
                        }
                    }
                    IrInstruction::ConstStr { dest, val: _ } => {
                        if started_virtual_registers.contains(dest) {
                            virtual_registers[*dest].end_use = ir_coord;
                        } else {
                            virtual_registers[*dest].start_use = ir_coord;
                        }
                    }
                    IrInstruction::Copy { dest, source } => {
                        if started_virtual_registers.contains(dest) {
                            virtual_registers[*dest].end_use = ir_coord;
                        } else {
                            virtual_registers[*dest].start_use = ir_coord;
                            started_virtual_registers.push(*dest);
                        }

                        if started_virtual_registers.contains(source) {
                            virtual_registers[*source].end_use = ir_coord;
                        } else {
                            virtual_registers[*source].start_use = ir_coord;
                            started_virtual_registers.push(*source);
                        }
                    }
                    IrInstruction::LoadGlobal { dest, name: _ } => {
                        if started_virtual_registers.contains(dest) {
                            virtual_registers[*dest].end_use = ir_coord;
                        } else {
                            virtual_registers[*dest].start_use = ir_coord;
                            started_virtual_registers.push(*dest);
                        }
                    }
                    IrInstruction::Return { val } => {
                        if let Some(val) = val {
                            if started_virtual_registers.contains(val) {
                                virtual_registers[*val].end_use = ir_coord;
                            } else {
                                virtual_registers[*val].start_use = ir_coord;
                                started_virtual_registers.push(*val);
                            }
                        }
                    }
                    IrInstruction::Jump { label: _ } => {}
                }
            }
        }

        println!("=== Virtual Register Lifetimes ===");
        for v_reg in virtual_registers.iter() {
            println!("{:?}", v_reg)
        }

        RegisterAllocator::run_allocation(&mut virtual_registers)
    }
}
