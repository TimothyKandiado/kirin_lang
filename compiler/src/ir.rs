use std::{collections::HashMap, fmt};

use crate::parser::{
    BinaryExprOp, Expression, FuncParam, FunctionDeclStmt, LiteralValue, Statement, UnaryExprOp,
    ValueType,
};

pub type Reg = usize;

pub type Label = usize;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Callee<'a> {
    Direct(&'a str),
    Indirect(Reg),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum IrInstruction<'a> {
    ConstI64 {
        dest: Reg,
        val: i64,
    },

    ConstF64 {
        dest: Reg,
        val: f64,
    },

    ConstBool {
        dest: Reg,
        val: bool,
    },

    ConstStr {
        dest: Reg,
        val: &'a str,
    },

    LoadGlobal {
        dest: Reg,
        name: &'a str,
    },

    Copy {
        dest: Reg,
        source: Reg,
    },

    BinOp {
        dest: Reg,
        op: BinaryExprOp,
        lhs: Reg,
        rhs: Reg,
        val_type: ValueType,
    },

    UnaryOp {
        dest: Reg,
        op: UnaryExprOp,
        rhs: Reg,
        val_type: ValueType,
    },

    Call {
        dest: Option<Reg>,
        callee: Callee<'a>,
        args: Vec<Reg>,
        val_type: ValueType,
    },

    Jump {
        label: Label,
    },

    Branch {
        cond: Reg,
        then_label: Label,
        else_label: Label,
    },

    Return {
        val: Option<Reg>,
    },
}

#[derive(Debug, Clone)]
pub struct IrBlock<'a> {
    pub label: Label,
    pub instructions: Vec<IrInstruction<'a>>,
}

impl<'a> IrBlock<'a> {
    pub fn new(label: Label) -> Self {
        Self {
            label,
            instructions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IrFunction<'a> {
    Bytecode {
        name: &'a str,
        params: Vec<FuncParam<'a>>,
        ret_type: ValueType,
        blocks: Vec<IrBlock<'a>>,
        reg_count: usize,
        reg_types: Vec<ValueType>,
    },
    Native {
        name: &'a str,
        params: Vec<FuncParam<'a>>,
        ret_type: ValueType,
    },
}

#[derive(Debug, Clone)]
pub struct IrModule<'a> {
    pub package_name: &'a str,
    pub file_name: &'a str,
    pub functions: Vec<IrFunction<'a>>,
    pub globals: HashMap<&'a str, IrGlobal<'a>>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct IrGlobal<'a> {
    pub val_type: ValueType,
    pub init: Option<IrConstant<'a>>,
}

impl fmt::Display for IrGlobal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(type = {}), (init = {:?})", self.val_type, self.init)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum IrConstant<'a> {
    String(&'a str),
    Function(&'a str),
}

struct IrBuilder<'a> {
    pub package_name: &'a str,
    pub globals: HashMap<&'a str, IrGlobal<'a>>,
    pub functions: Vec<IrFunction<'a>>,
    pub current_function: Option<usize>,
    pub scope_stack: Vec<HashMap<&'a str, Reg>>,
}

impl<'a> IrBuilder<'a> {
    pub fn lower_expression(&mut self, expr: &Expression<'a>) -> Option<Reg> {
        match expr {
            Expression::None => None,
            Expression::Binary(bin) => {
                let lhs = self
                    .lower_expression(&bin.left)
                    .expect("lhs must yield value");
                let rhs = self
                    .lower_expression(&bin.right)
                    .expect("rhs must yield value");

                let dest = self.get_register(bin.value_type.clone());

                self.push_instruction(IrInstruction::BinOp {
                    dest,
                    op: bin.op,
                    lhs,
                    rhs,
                    val_type: bin.value_type.clone(),
                });

                Some(dest)
            }
            Expression::Unary(unary) => {
                let rhs = self
                    .lower_expression(&unary.value)
                    .expect("unary expression must yield a value");
                let dest = self.get_register(unary.value_type.clone());

                self.push_instruction(IrInstruction::UnaryOp {
                    dest,
                    op: unary.op,
                    rhs,
                    val_type: unary.value_type.clone(),
                });

                Some(dest)
            }
            Expression::Grouping(grouping) => self.lower_expression(&grouping.expression),
            Expression::Assign(assign) => {
                let value = self
                    .lower_expression(&assign.value)
                    .expect("assignment expression must yield a value");

                let local = self.get_local(assign.name);
                if let Some(local) = local {
                    self.push_instruction(IrInstruction::Copy {
                        dest: local,
                        source: value,
                    });
                }

                None
            }

            Expression::Variable(variable) => {
                if let Some(local) = self.get_local(variable.name) {
                    return Some(local);
                }

                if let Some(global) = self.globals.get(variable.name) {
                    let dest = self.get_register(global.val_type.clone());

                    self.push_instruction(IrInstruction::LoadGlobal {
                        dest,
                        name: variable.name,
                    });

                    return Some(dest);
                }

                panic!("undefined variable name {}", variable.name);
            }

            Expression::Literal(literal) => {
                let dest = self.get_register(literal.value_type.clone());

                match literal.value {
                    LiteralValue::F64(val) => {
                        self.push_instruction(IrInstruction::ConstF64 { dest, val });
                    }
                    LiteralValue::I64(val) => {
                        self.push_instruction(IrInstruction::ConstI64 { dest, val });
                    }
                    LiteralValue::Bool(val) => {
                        self.push_instruction(IrInstruction::ConstBool { dest, val });
                    }
                    LiteralValue::String(val) => {
                        self.push_instruction(IrInstruction::ConstStr { dest, val });
                    }
                }

                Some(dest)
            }

            Expression::Call(call) => {
                if let Expression::Variable(var_expr) = &call.callee {
                    // optimization for direct calls
                    if self.get_local(var_expr.name).is_none()
                        && let Some(global) = self.globals.get(var_expr.name)
                        && let ValueType::Fn(_) = global.val_type
                    {
                        let mut args = Vec::new();

                        for arg in &call.arguments {
                            let reg = self
                                .lower_expression(arg)
                                .expect("argument expression should yield a value");

                            args.push(reg);
                        }

                        fn is_sequential(v: &[usize]) -> bool {
                            v.windows(2).all(|w| w[1] == w[0] + 1)
                        }

                        if !is_sequential(&args) {
                            let mut moved_args = Vec::new();

                            let function_registers = self.get_allocated_registers();

                            for arg in args {
                                let val_type = function_registers[arg].clone();
                                let reg = self.get_register(val_type);

                                self.push_instruction(IrInstruction::Copy {
                                    dest: reg,
                                    source: arg,
                                });

                                moved_args.push(reg);
                            }

                            args = moved_args;
                        }

                        let dest = self.get_register(call.value_type.clone());

                        self.push_instruction(IrInstruction::Call {
                            dest: Some(dest),
                            callee: Callee::Direct(var_expr.name),
                            args,
                            val_type: call.value_type.clone(),
                        });

                        return Some(dest);
                    }
                }

                let callee = self
                    .lower_expression(&call.callee)
                    .expect("callee expression should yield a value");

                let mut args = Vec::new();

                for arg in &call.arguments {
                    let reg = self
                        .lower_expression(arg)
                        .expect("argument expression should yield a value");
                    args.push(reg);
                }
                let dest = self.get_register(call.value_type.clone());

                self.push_instruction(IrInstruction::Call {
                    dest: Some(dest),
                    callee: Callee::Indirect(callee),
                    args,
                    val_type: call.value_type.clone(),
                });

                Some(dest)
            }
        }
    }

    pub fn lower_statement(&mut self, stmt: &Statement<'a>) {
        match stmt {
            Statement::None => {}

            Statement::FunctionDecl(func) => {
                match func.as_ref() {
                    FunctionDeclStmt::Native {
                        name,
                        params,
                        line: _,
                        column: _,
                        return_type,
                    } => {
                        self.functions.push(IrFunction::Native {
                            name,
                            params: params.clone(),
                            ret_type: return_type.clone(),
                        });
                    }

                    FunctionDeclStmt::Bytecode {
                        name,
                        params,
                        body,
                        line: _,
                        column: _,
                        return_type,
                    } => {
                        let previous_function = self.current_function.take();

                        self.current_function = Some(self.functions.len());

                        self.functions.push(IrFunction::Bytecode {
                            name,
                            params: params.clone(),
                            ret_type: return_type.clone(),
                            blocks: Vec::new(),
                            reg_count: 0,
                            reg_types: Vec::new(),
                        });

                        // push initial block
                        _ = self.push_block();
                        // add initial scope
                        self.push_scope();

                        // add parameters as locals
                        for param in params {
                            _ = self.add_local(param.name, param.value_type.clone());
                        }

                        self.lower_statement(body);

                        if let Some(last_instruction) = self.get_last_instruction() 
                        && let IrInstruction::Return {val: _} = last_instruction  {
                            
                        } else {
                            if return_type == &ValueType::Void {
                                self.push_instruction(IrInstruction::Return { val: None });
                            }
                        }

                        // remove initial scope
                        self.pop_scope();

                        self.current_function = previous_function;
                    }
                }
            }
            Statement::Block(block) => {
                self.push_scope();

                for stmt in &block.statements {
                    self.lower_statement(stmt);
                }

                self.pop_scope();
            }

            Statement::If(if_stmt) => {
                let condition_reg = self
                    .lower_expression(&if_stmt.condition)
                    .expect("condition expression must yield a value");

                let (branch_block_idx, branch_inst_idx) =
                    self.push_instruction(IrInstruction::Branch {
                        cond: condition_reg,
                        then_label: 0,
                        else_label: 0,
                    });
                let then_block_idx = self.push_block();

                // enter the then branch
                self.lower_statement(&if_stmt.then_branch);

                let (_, then_merge_jump_inst_idx) =
                    self.push_instruction(IrInstruction::Jump { label: 0 });

                let mut else_block_idx: Option<Label> = None;
                let mut else_merge_jump_inst_idx: Option<usize> = None;

                if let Some(else_branch) = &if_stmt.else_branch {
                    else_block_idx = Some(self.push_block());
                    self.lower_statement(else_branch);
                    let (_, inst_idx) = self.push_instruction(IrInstruction::Jump { label: 0 });
                    else_merge_jump_inst_idx = Some(inst_idx)
                }

                let merge_block_idx = self.push_block();

                if let (Some(else_block_idx), Some(else_merge_jump_inst_idx)) =
                    (else_block_idx, else_merge_jump_inst_idx)
                {
                    self.edit_instruction(
                        IrInstruction::Branch {
                            cond: condition_reg,
                            then_label: then_block_idx,
                            else_label: else_block_idx,
                        },
                        branch_block_idx,
                        branch_inst_idx,
                    );
                    self.edit_instruction(
                        IrInstruction::Jump {
                            label: merge_block_idx,
                        },
                        then_block_idx,
                        then_merge_jump_inst_idx,
                    );
                    self.edit_instruction(
                        IrInstruction::Jump {
                            label: merge_block_idx,
                        },
                        else_block_idx,
                        else_merge_jump_inst_idx,
                    );
                } else {
                    self.edit_instruction(
                        IrInstruction::Branch {
                            cond: condition_reg,
                            then_label: then_block_idx,
                            else_label: merge_block_idx,
                        },
                        branch_block_idx,
                        branch_inst_idx,
                    );
                    self.edit_instruction(
                        IrInstruction::Jump {
                            label: merge_block_idx,
                        },
                        then_block_idx,
                        then_merge_jump_inst_idx,
                    );
                }
            }

            Statement::For(for_stmt) => {
                self.push_scope();

                if let Some(init) = &for_stmt.initializer {
                    let init_block = self.push_block();
                    self.lower_statement(init);
                    self.push_instruction(IrInstruction::Jump { label: init_block + 1 });
                }

                let mut condition_block_label = None;
                let mut branch_block_idx = None;
                let mut branch_inst_idx = None;
                let mut cond_reg = None;

                if let Some(condition) = &for_stmt.condition {
                    let label = self.push_block();
                    condition_block_label = Some(label);

                    let condition_reg = self.lower_expression(condition).expect("condition should return value");
                    cond_reg = Some(condition_reg);

                    let (block_idx, inst_idx) = self.push_instruction(IrInstruction::Branch { cond: condition_reg, then_label: label + 1, else_label: 0 });
                    (branch_block_idx, branch_inst_idx) = (Some(block_idx), Some(inst_idx))
                }

                let loop_body = self.push_block();
                self.lower_statement(&for_stmt.body);

                let (continue_block_idx, continue_inst_idx) = self.push_instruction(IrInstruction::Jump { label: 0 });

                let footer_label = self.push_block();
                if let Some(footer) = &for_stmt.footer {
                    self.lower_statement(footer);
                }

                self.edit_instruction(IrInstruction::Jump { label: footer_label }, continue_block_idx, continue_inst_idx);
                
                if let Some(condition_label) = condition_block_label {
                    self.push_instruction(IrInstruction::Jump { label: condition_label });
                } else {
                    self.push_instruction(IrInstruction::Jump { label: loop_body });
                }

                let break_label = self.push_block();

                if condition_block_label.is_some() {
                    self.edit_instruction(
                        IrInstruction::Branch { cond: cond_reg.unwrap(), then_label: loop_body, else_label: break_label }, 
                        branch_block_idx.unwrap(), 
                        branch_inst_idx.unwrap());
                }
                

                self.pop_scope();
            }

            Statement::VarDecl(var_decl) => {
                let reg = self.add_local(var_decl.name, var_decl.value_type.clone());

                if let Some(expr) = &var_decl.value {
                    let result = self
                        .lower_expression(expr)
                        .expect("assignment expression must yield a value");

                    self.push_instruction(IrInstruction::Copy {
                        dest: reg,
                        source: result,
                    });
                }
            }

            Statement::Return(ret_stmt) => {
                let reg = ret_stmt.value.as_ref().map(|expr| {
                    self.lower_expression(expr)
                        .expect("return expression must yield a value")
                });

                self.push_instruction(IrInstruction::Return { val: reg });
            }

            Statement::Expr(expr_stmt) => {
                self.lower_expression(expr_stmt);
            }

            Statement::PackageDecl(_) => {}
        }
    }

    fn get_register(&mut self, value_type: ValueType) -> Reg {
        if self.current_function.is_none() {
            panic!("cannot request a register in global scope")
        }

        let function = self
            .functions
            .get_mut(self.current_function.unwrap())
            .unwrap();

        if let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type: _,
            blocks: _,
            reg_count,
            reg_types,
        } = function
        {
            let register = *reg_count;
            *reg_count += 1;

            reg_types.push(value_type);

            register
        } else {
            panic!("cannot allocate registers for native function")
        }
    }

    fn get_allocated_registers(&self) -> Vec<ValueType> {
        if self.current_function.is_none() {
            panic!("cannot request a register in global scope")
        }

        let function = self.functions.get(self.current_function.unwrap()).unwrap();

        let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type: _,
            blocks: _,
            reg_count: _,
            reg_types,
        } = function
        else {
            panic!("cannot get registers in a non Bytecode function")
        };

        reg_types.clone()
    }

    fn push_block(&mut self) -> Label {
        if self.current_function.is_none() {
            panic!("cannot add a block in global scope")
        }

        let function = self
            .functions
            .get_mut(self.current_function.unwrap())
            .unwrap();

        if let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type: _,
            blocks,
            reg_count: _,
            reg_types: _,
        } = function
        {
            let label = blocks.len();

            blocks.push(IrBlock {
                label,
                instructions: Vec::new(),
            });

            label
        } else {
            panic!("cannot add a block for native function")
        }
    }

    /// returns (block_idx, instruction_idx)
    fn push_instruction(&mut self, inst: IrInstruction<'a>) -> (usize, usize) {
        if self.current_function.is_none() {
            panic!("cannot add an instruction in global scope")
        }

        let function = self
            .functions
            .get_mut(self.current_function.unwrap())
            .unwrap();

        if let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type: _,
            blocks,
            reg_count: _,
            reg_types: _,
        } = function
        {
            let last_block = blocks.last_mut().unwrap();

            last_block.instructions.push(inst);

            // return idx of pushed instruction
            let instruction_idx = last_block.instructions.len() - 1;
            let block_idx = blocks.len() - 1;

            (block_idx, instruction_idx)
        } else {
            panic!("cannot add an instruction for native function")
        }
    }

    fn edit_instruction(
        &mut self,
        inst: IrInstruction<'a>,
        block_idx: usize,
        instruction_idx: usize,
    ) {
        if self.current_function.is_none() {
            panic!("cannot edit an instruction in global scope")
        }

        let function = self
            .functions
            .get_mut(self.current_function.unwrap())
            .unwrap();

        if let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type: _,
            blocks,
            reg_count: _,
            reg_types: _,
        } = function
        {
            let block = blocks.get_mut(block_idx).unwrap();
            let instruction = block.instructions.get_mut(instruction_idx).unwrap();

            *instruction = inst;
        } else {
            panic!("cannot edit an instruction for native function")
        }
    }

    fn get_last_instruction(&self) -> Option<&IrInstruction<'a>> {
        if self.current_function.is_none() {
            panic!("cannot get an instruction in global scope")
        }

        let function = self
            .functions
            .get(self.current_function.unwrap())
            .unwrap();

        if let IrFunction::Bytecode {
            name: _,
            params: _,
            ret_type: _,
            blocks,
            reg_count: _,
            reg_types: _,
        } = function
        {
            let last_block = blocks.last().unwrap();

            last_block.instructions.last()
        } else {
            panic!("cannot get an instruction for native function")
        }
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn add_local(&mut self, name: &'a str, value_type: ValueType) -> Reg {
        let reg = self.get_register(value_type);

        let top_scope = self
            .scope_stack
            .last_mut()
            .expect("expected valid scope before adding local");

        top_scope.insert(name, reg);

        reg
    }

    fn get_local(&mut self, name: &'a str) -> Option<Reg> {
        self.scope_stack
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn build_module(self) -> IrModule<'a> {
        IrModule {
            package_name: self.package_name,
            file_name: "",
            functions: self.functions,
            globals: self.globals,
        }
    }
}

pub fn lower_ast<'a>(statements: &[Statement<'a>]) -> IrModule<'a> {
    // scan globals
    let mut globals = HashMap::<&'a str, IrGlobal<'a>>::new();
    let mut package_name = "";

    // handle globals
    for stmt in statements {
        match stmt {
            Statement::FunctionDecl(func_decl) => {
                let ir_constant = IrConstant::Function(func_decl.get_name());
                let ir_global = IrGlobal {
                    val_type: ValueType::Fn(Box::new(func_decl.get_signature())),
                    init: Some(ir_constant),
                };

                globals.insert(func_decl.get_name(), ir_global);
            }
            Statement::PackageDecl(stmt) => {
                package_name = stmt.name;
            }

            _ => {}
        }
    }

    let mut ir_builder = IrBuilder {
        current_function: None,
        package_name,
        globals,
        functions: Vec::new(),
        scope_stack: Vec::new(),
    };

    // lower statements
    for stmt in statements {
        ir_builder.lower_statement(stmt);
    }

    ir_builder.build_module()
}
