use core::fmt;

use crate::ir::{Callee, IrFunction, IrInstruction};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Default)]
struct IrInstCoord {
    pub block_id: usize,
    pub inst_id: usize,
}

impl fmt::Display for IrInstCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B{}:I{}", self.block_id, self.inst_id)
    }
}

#[derive(Debug, Clone, Copy)]
struct VirtualRegister {
    id: usize,
    size: usize,
    start_use: IrInstCoord,
    end_use: IrInstCoord,
    group_id: Option<usize>,
    start_set: bool,
}

impl VirtualRegister {
    pub fn new(id: usize, size: usize) -> Self {
        Self {
            id,
            size,
            start_use: IrInstCoord::default(),
            end_use: IrInstCoord::default(),
            group_id: None,
            start_set: false,
        }
    }

    pub fn set_coord(&mut self, coord: IrInstCoord) {
        if !self.start_set {
            self.start_use = coord;
            self.end_use = coord;
            self.start_set = true;
        } else {
            self.end_use = coord;
        }
    }
}

impl fmt::Display for VirtualRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let group_id = match self.group_id {
            Some(id) => id.to_string(),
            None => "_".to_string(),
        };

        write!(
            f,
            "v{} (size: {}, group: {}) lifetime [{} - {}]",
            self.id, self.size, group_id, self.start_use, self.end_use
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RegisterAllocation {
    pub offset: usize,
    pub size: usize,
}

impl fmt::Display for RegisterAllocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "offset: {}, size: {}", self.offset, self.size)
    }
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

    fn merge_adjacent_free_blocks(&mut self) {
        if self.free_list.len() <= 1 {
            return;
        }

        // Sort by offset
        self.free_list.sort_unstable_by_key(|&(offset, _)| offset);

        let mut merged: Vec<(usize, usize)> = Vec::with_capacity(self.free_list.len());

        let mut current = self.free_list[0];

        for &(offset, size) in &self.free_list[1..] {
            let current_end = current.0 + current.1;

            if current_end == offset {
                // Adjacent → extend current block
                current.1 += size;
            } else {
                // Not adjacent → push current and start new
                merged.push(current);
                current = (offset, size);
            }
        }

        // Push the last block
        merged.push(current);

        self.free_list = merged;
    }

    fn get_total_registers(&self) -> usize {
        self.next_offset
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

    fn allocate_group(&mut self, sizes: &[usize]) -> Vec<RegisterAllocation> {
        let total_size = sizes.iter().sum();

        let total_allocation = self.allocate(total_size);

        let mut current_offset = total_allocation.offset;

        let mut allocations = Vec::new();

        for &size in sizes {
            allocations.push(RegisterAllocation {
                offset: current_offset,
                size,
            });

            current_offset += size
        }

        allocations
    }

    /// (total, allocations)
    fn run_allocation(values: &mut [VirtualRegister]) -> (usize, Vec<RegisterAllocation>) {
        let mut alloc = RegisterAllocator::new();

        let mut result = vec![RegisterAllocation { offset: 0, size: 0 }; values.len()];

        values.sort_by_key(|v| v.start_use);

        let mut grouped_values: Vec<Vec<&VirtualRegister>> = Vec::new();

        for v in values.iter() {
            if let Some(last) = grouped_values.last_mut()
                && let (Some(last_group), Some(current_group)) = (last[0].group_id, v.group_id)
                    && last_group == current_group
                {
                    last.push(v);
                    continue;
                }

            grouped_values.push(vec![v]);
        }

        for group in grouped_values.iter() {
            for v in group.iter() {
                alloc.expire_old_intervals(v.start_use);
            }

            alloc.merge_adjacent_free_blocks();

            let sizes: Vec<usize> = group.iter().map(|i| i.size).collect();

            let allocations = alloc.allocate_group(&sizes);

            for (a, v) in allocations.iter().zip(group) {
                alloc.active.push((v.id, *a, v.end_use));
                result[v.id] = *a;
            }
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
            .map(|(idx, reg_type)| VirtualRegister::new(idx, reg_type.get_size()))
            .collect();

        let mut current_group_id: usize = 0;

        let mut get_group_id = move || {
            current_group_id += 1;

            current_group_id - 1
        };

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
                        virtual_registers[*dest].set_coord(ir_coord);

                        virtual_registers[*lhs].set_coord(ir_coord);

                        virtual_registers[*rhs].set_coord(ir_coord);
                    }
                    IrInstruction::UnaryOp {
                        dest,
                        op: _,
                        rhs,
                        val_type: _,
                    } => {
                        virtual_registers[*dest].set_coord(ir_coord);

                        virtual_registers[*rhs].set_coord(ir_coord);
                    }
                    IrInstruction::Branch {
                        cond,
                        then_label: _,
                        else_label: _,
                    } => {
                        virtual_registers[*cond].set_coord(ir_coord);
                    }
                    IrInstruction::Call {
                        dest,
                        callee,
                        args,
                        val_type: _,
                    } => {
                        if let Some(dest) = dest {
                            virtual_registers[*dest].set_coord(ir_coord);
                        }

                        if let Callee::Indirect(callee) = callee {
                            virtual_registers[*callee].set_coord(ir_coord);
                        }

                        let group_id = get_group_id();

                        for arg in args {
                            virtual_registers[*arg].set_coord(ir_coord);
                            virtual_registers[*arg].group_id = Some(group_id);
                        }
                    }
                    IrInstruction::ConstBool { dest, val: _ } => {
                        virtual_registers[*dest].set_coord(ir_coord);
                    }
                    IrInstruction::ConstI64 { dest, val: _ } => {
                        virtual_registers[*dest].set_coord(ir_coord);
                    }
                    IrInstruction::ConstF64 { dest, val: _ } => {
                        virtual_registers[*dest].set_coord(ir_coord);
                    }
                    IrInstruction::ConstStr { dest, val: _ } => {
                        virtual_registers[*dest].set_coord(ir_coord);
                    }
                    IrInstruction::Copy { dest, source } => {
                        virtual_registers[*dest].set_coord(ir_coord);

                        virtual_registers[*source].set_coord(ir_coord);
                    }
                    IrInstruction::LoadGlobal { dest, name: _ } => {
                        virtual_registers[*dest].set_coord(ir_coord);
                    }
                    IrInstruction::Return { val } => {
                        if let Some(val) = val {
                            virtual_registers[*val].set_coord(ir_coord);
                        }
                    }
                    IrInstruction::Jump { label: _ } => {}
                }
            }
        }

        #[cfg(debug_assertions)]
        {
            println!("=== Virtual Register Lifetimes ===");
            for v_reg in virtual_registers.iter() {
                println!("{}", v_reg)
            }
        }

        RegisterAllocator::run_allocation(&mut virtual_registers)
    }
}
