use std::collections::HashMap;

use crate::parser::{BinaryExprOp, FuncParam, LiteralValue, Statement, UnaryExprOp, ValueType};

#[derive(Debug, Clone, Copy)]
pub enum MidIrValue {
    None,
    Temporary(usize, ValueType),
    Local(usize, ValueType),
    Constant(usize, ValueType),
    Global(usize, ValueType)
}

#[derive(Debug, Clone)]
pub struct MidIrBinaryInst {
    pub dest: MidIrValue,
    pub value_type: ValueType,
    pub op: BinaryExprOp,
    pub left: MidIrValue,
    pub right: MidIrValue,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct MidIrUnaryInst {
    pub dest: MidIrValue,
    pub value_type: ValueType,
    pub op: UnaryExprOp,
    pub right: MidIrValue,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct MidIrCallInst {
    pub dest: MidIrValue,
    pub value_type: ValueType,
    pub callee: MidIrValue,
    pub args: Vec<MidIrValue>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct MidIrAssignInst {
    pub dest: MidIrValue,
    pub value_type: ValueType,
    pub value: MidIrValue,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct MidIrJumpIfInst {
    pub condition: MidIrValue,
    pub label: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct MidIrJumpInst {
    pub label: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct MidIrLabelInst {
    pub label: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct MidIrReturnInst {
    pub value: Option<MidIrValue>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub enum MidIrInstruction {
    NoOp,

    Binary(MidIrBinaryInst),
    Unary(MidIrUnaryInst),

    Call(MidIrCallInst),

    Assign(MidIrAssignInst),

    JumpIf(MidIrJumpIfInst),

    Jump(MidIrJumpInst),

    Label(MidIrLabelInst),

    Return(MidIrReturnInst),
}

impl MidIrInstruction {
    pub fn get_line(&self) -> usize {
        match self {
            Self::NoOp => 0,
            Self::Binary(i) => i.line,
            Self::Unary(i) => i.line,
            Self::Call(i) => i.line,
            Self::Assign(i) => i.line,
            Self::JumpIf(i) => i.line,
            Self::Jump(i) => i.line,
            Self::Label(i) => i.line,
            Self::Return(i) => i.line,
        }
    }

    pub fn get_column(&self) -> usize {
        match self {
            Self::NoOp => 0,
            Self::Binary(i) => i.column,
            Self::Unary(i) => i.column,
            Self::Call(i) => i.column,
            Self::Assign(i) => i.column,
            Self::JumpIf(i) => i.column,
            Self::Jump(i) => i.column,
            Self::Label(i) => i.column,
            Self::Return(i) => i.column,
        }
    }
}

pub struct MidIrVariable<'a> {
    pub name: &'a str,
    pub value_type: ValueType,
    pub version: usize
}

pub enum MidIrFunction<'a> {
    Native(MidIrNativeFunc<'a>),
    UserDefined(MidIrUserFunc<'a>),
}

pub struct MidIrNativeFunc<'a> {
    pub name: &'a str,
    pub params: Vec<FuncParam<'a>>,
    pub return_type: ValueType,
}

pub struct MidIrUserFunc<'a> {
    pub name: &'a str,
    pub params: Vec<FuncParam<'a>>,
    pub return_type: ValueType,
    pub instructions: Vec<MidIrInstruction>,
    pub locals: Vec<MidIrVariable<'a>>,
}

pub struct MidIrModule<'a> {
    pub package_name: &'a str,
    pub file_name: &'a str,
    pub functions: Vec<MidIrFunction<'a>>,
    pub globals: Vec<MidIrVariable<'a>>,
    pub locals: Vec<MidIrVariable<'a>>
}

struct ScopedLocal {
    pub id: usize,
    pub value_type: ValueType,
}

pub struct MidIrError {
    pub context: String,
    pub line: usize,
    pub column: usize,
}

struct MidIrLoweringContext<'a> {
    current_function: Option<usize>,
    functions: Vec<MidIrFunction<'a>>,
    temp_counter: usize,
    label_counter: usize,
    constants: Vec<LiteralValue<'a>>,
    scope_stack: Vec<HashMap<&'a str, ScopedLocal>>,
    globals: Vec<MidIrVariable<'a>>,
}

impl<'a> MidIrLoweringContext<'a> {
    pub fn new() -> Self {
        return Self {
            current_function: None,
            functions: Vec::new(),
            temp_counter: 0,
            label_counter: 0,
            constants: Vec::new(),
            scope_stack: Vec::new(),
            globals: Vec::new()
        }
    }

    pub fn lower_ast(mut self, statements: Vec<Statement<'a>>) -> Result<MidIrModule, MidIrError> {
        todo!()
    }

    fn new_temp(&mut self) -> usize {
        self.temp_counter += 1;
        return self.temp_counter - 1;
    }

    fn new_label(&mut self) -> usize {
        self.label_counter += 1;
        return self.label_counter - 1;
    }

    fn add_instruction(&mut self, inst: MidIrInstruction) -> Result<(), MidIrError> {
        if self.current_function.is_none() {
            return Err(MidIrError { context: "cannot add an instruction in global scope".to_string(), line: inst.get_line(), column: inst.get_column() })
        }

        let function = self.functions.get_mut(self.current_function.unwrap());

        if let Some(function) = function {
            if let MidIrFunction::UserDefined(function) = function {
                function.instructions.push(inst);
            }
        }

        Ok(())
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        _ = self.scope_stack.pop();
    }

    fn add_variable(&mut self, name: &'a str, value_type: ValueType) -> MidIrValue {
        if (self.scope_stack.len() == 0) { // global scope
            let existing_global = self.globals.iter().find(|&&i| -> bool {true} );
        }
    }
}
