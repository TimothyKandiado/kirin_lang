use std::collections::HashMap;

use crate::parser::{
    BinaryExpr, BinaryExprOp, CallExpr, Expression, FunctionDeclStmt, FunctionSignature, LiteralValue, Statement, UnaryExpr, UnaryExprOp, ValueType, format_binary_op, format_type
};

struct SymbolTable<'a> {
    scopes: Vec<HashMap<&'a str, ValueType>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: &'a str, val_type: ValueType) {
        self.scopes.last_mut().unwrap().insert(name, val_type);
    }

    pub fn lookup(&mut self, name: &'a str) -> Option<ValueType> {
        self.scopes.iter().rev().find_map(|scope| {
            let val = scope.get(name);

            val.cloned()
        })
    }
}

#[derive(Debug, Clone)]
pub struct TypeError {
    pub line: usize,
    pub column: usize,
    pub context: String,
}

pub struct TypeChecker<'a> {
    symbols: SymbolTable<'a>,
    errors: Vec<TypeError>,
    functions: HashMap<&'a str, FunctionSignature>,
    current_return_type: Option<ValueType>,
}

impl<'a> Default for TypeChecker<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TypeChecker<'a> {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            errors: Vec::new(),
            functions: HashMap::new(),
            current_return_type: None,
        }
    }

    fn error(&mut self, line: usize, column: usize, msg: impl Into<String>) {
        self.errors.push(TypeError {
            line,
            column,
            context: msg.into(),
        });
    }

    pub fn check_module(mut self, statements: &mut [Statement<'a>]) -> Vec<TypeError> {
        // register globals
        for statement in statements.iter() {
            if let Statement::FunctionDecl(func) = statement {
                self.symbols.declare(
                    func.get_name(),
                    ValueType::Fn(Box::new(func.get_signature().clone())),
                );

                self.functions
                    .insert(func.get_name(), func.get_signature().clone());
            }
        }

        // check bodies
        for statement in statements {
            self.check_statement(statement);
        }

        self.errors
    }

    fn check_statement(&mut self, statement: &mut Statement<'a>) {
        match statement {
            Statement::None => {}

            Statement::PackageDecl(_) => {}

            Statement::Expr(expr) => {
                self.check_expression(expr);
            }

            Statement::FunctionDecl(func) => {
                if let FunctionDeclStmt::Bytecode {
                    name: _,
                    params,
                    body,
                    line,
                    column,
                    return_type,
                } = func.as_mut()
                {
                    // Validate return type.
                    if let Err(msg) = require_defined(return_type) {
                        self.error(*line, *column, msg);
                    }

                    self.symbols.push();

                    // Bring parameters into scope and validate their types.
                    for param in params {
                        if let Err(msg) = require_defined(&param.value_type) {
                            self.error(*line, *column, msg);
                        }
                        self.symbols.declare(param.name, param.value_type.clone());
                    }

                    let outer = self.current_return_type.replace(return_type.clone());
                    self.check_statement(body);
                    self.current_return_type = outer;

                    self.symbols.pop();
                };
            }
            Statement::Block(block) => {
                self.symbols.push();
                for s in &mut block.statements {
                    self.check_statement(s);
                }
                self.symbols.pop();
            }
            Statement::If(if_stmt) => {
                let cond_ty = self.check_expression(&mut if_stmt.condition);
                if cond_ty != ValueType::Bool {
                    self.error(
                        if_stmt.line,
                        if_stmt.column,
                        format!("if condition must be bool, got {}", format_type(&cond_ty)),
                    );
                }
                self.check_statement(&mut if_stmt.then_branch);
                if let Some(else_branch) = &mut if_stmt.else_branch {
                    self.check_statement(else_branch);
                }
            }

            Statement::Return(ret) => {
                let expected = self.current_return_type.clone().unwrap_or(ValueType::Void);
                match &mut ret.value {
                    None => {
                        if expected != ValueType::Void {
                            self.error(
                                ret.line,
                                ret.column,
                                format!(
                                    "empty return in function expecting {}",
                                    format_type(&expected)
                                ),
                            );
                        }
                    }
                    Some(val) => {
                        let actual = self.check_expression(val);
                        if !types_compatible(&expected, &actual) {
                            self.error(
                                ret.line,
                                ret.column,
                                format!(
                                    "return type mismatch: expected {}, got {}",
                                    format_type(&expected),
                                    format_type(&actual)
                                ),
                            );
                        }
                    }
                }
            }

            Statement::VarDecl(decl) => {
                // The declared type must be concrete.
                if let Err(msg) = require_defined(&decl.value_type) {
                    self.error(decl.line, decl.column, msg);
                }

                if let Some(init) = &mut decl.value {
                    let init_ty = self.check_expression(init);
                    if let (Ok(()), Ok(())) =
                        (require_defined(&decl.value_type), require_defined(&init_ty))
                        && !types_compatible(&decl.value_type, &init_ty) {
                            self.error(
                                decl.line,
                                decl.column,
                                format!(
                                    "variable '{}' declared as {} but initialised with {}",
                                    decl.name,
                                    format_type(&decl.value_type),
                                    format_type(&init_ty)
                                ),
                            );
                        }
                }

                self.symbols.declare(decl.name, decl.value_type.clone());
            }
        }
    }

    fn check_expression(&mut self, expression: &mut Expression<'a>) -> ValueType {
        match expression {
            Expression::None => ValueType::Undefined,

            Expression::Literal(literal) => {
                let inferred = match literal.value {
                    LiteralValue::I64(_) => ValueType::I64,
                    LiteralValue::F64(_) => ValueType::F64,
                    LiteralValue::Bool(_) => ValueType::Bool,
                    LiteralValue::String(_) => ValueType::String,
                };

                self.check_annotation(literal.line, literal.column, &literal.value_type, &inferred);
                inferred
            }

            Expression::Variable(var) => {
                match self.symbols.lookup(var.name) {
                    Some(ty) => {
                        self.check_annotation(var.line, var.column, &var.value_type, &ty);
                        ty
                    }
                    None => {
                        // Could be a function reference — check the function table.
                        if self.functions.contains_key(var.name) {
                            // Function references carry Undefined in value_type from the
                            // parser; that is acceptable here.
                            return ValueType::Undefined;
                        }
                        self.error(
                            var.line,
                            var.column,
                            format!("undefined variable '{}'", var.name),
                        );
                        ValueType::Undefined
                    }
                }
            }

            Expression::Grouping(g) => {
                let inner = self.check_expression(&mut g.expression);
                // The annotated type must agree with what we inferred.
                self.check_annotation(g.line, g.column, &g.value_type, &inner);
                inner
            }

            Expression::Assign(assign) => {
                let rhs_ty = self.check_expression(&mut assign.value);

                match self.symbols.lookup(assign.name) {
                    Some(var_ty) => {
                        if !types_compatible(&var_ty, &rhs_ty) {
                            self.error(
                                assign.line,
                                assign.column,
                                format!(
                                    "cannot assign {} to variable '{}' of type {}",
                                    format_type(&rhs_ty),
                                    assign.name,
                                    format_type(&var_ty)
                                ),
                            );
                        }
                    }
                    None => {
                        self.error(
                            assign.line,
                            assign.column,
                            format!("assignment to undefined variable '{}'", assign.name),
                        );
                    }
                }
                ValueType::Void
            }

            Expression::Binary(bin) => self.check_binary(bin.as_mut()),

            Expression::Unary(un) => self.check_unary(un.as_mut()),

            Expression::Call(call) => {
                let call = call.as_mut();
                self.check_call(call)
            }
        }
    }

    fn check_binary(&mut self, bin: &mut BinaryExpr<'a>) -> ValueType {
        let lhs_ty = self.check_expression(&mut bin.left);
        let rhs_ty = self.check_expression(&mut bin.right);

        // Skip further checks if either side failed to resolve.
        if lhs_ty == ValueType::Undefined || rhs_ty == ValueType::Undefined {
            return ValueType::Undefined;
        }

        let result_ty = match bin.op {
            // ── arithmetic ───────────────────────────────────────────────────
            BinaryExprOp::Add
            | BinaryExprOp::Sub
            | BinaryExprOp::Mul
            | BinaryExprOp::Div
            | BinaryExprOp::Mod
            | BinaryExprOp::Pow => {
                if !is_numeric(&lhs_ty) {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "left operand of '{}' must be i64 or f64, got {}",
                            format_binary_op(bin.op),
                            format_type(&lhs_ty)
                        ),
                    );
                }
                if !is_numeric(&rhs_ty) {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "right operand of '{}' must be i64 or f64, got {}",
                            format_binary_op(bin.op),
                            format_type(&rhs_ty)
                        ),
                    );
                }
                if is_numeric(&lhs_ty) && is_numeric(&rhs_ty) && lhs_ty != rhs_ty {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "arithmetic operands must have the same type, got {} and {}",
                            format_type(&lhs_ty),
                            format_type(&rhs_ty)
                        ),
                    );
                }
                // Result type mirrors the operands (or Undefined on error).
                if is_numeric(&lhs_ty) && lhs_ty == rhs_ty {
                    lhs_ty
                } else {
                    ValueType::Undefined
                }
            }

            // ── comparison ───────────────────────────────────────────────────
            BinaryExprOp::Greater
            | BinaryExprOp::GreaterEqual
            | BinaryExprOp::Less
            | BinaryExprOp::LessEqual => {
                if !is_numeric(&lhs_ty) {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "left operand of '{}' must be i64 or f64, got {}",
                            format_binary_op(bin.op),
                            format_type(&lhs_ty)
                        ),
                    );
                }
                if !is_numeric(&rhs_ty) {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "right operand of '{}' must be i64 or f64, got {}",
                            format_binary_op(bin.op),
                            format_type(&rhs_ty)
                        ),
                    );
                }
                if is_numeric(&lhs_ty) && is_numeric(&rhs_ty) && lhs_ty != rhs_ty {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "comparison operands must have the same type, got {} and {}",
                            format_type(&lhs_ty),
                            format_type(&rhs_ty)
                        ),
                    );
                }
                ValueType::Bool
            }

            // ── equality (any matching type is fine) ─────────────────────────
            BinaryExprOp::Equal | BinaryExprOp::NotEqual => {
                if !types_compatible(&lhs_ty, &rhs_ty) {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "equality operands must have the same type, got {} and {}",
                            format_type(&lhs_ty),
                            format_type(&rhs_ty)
                        ),
                    );
                }
                ValueType::Bool
            }

            // ── logical (both sides must be bool) ────────────────────────────
            BinaryExprOp::And | BinaryExprOp::Or => {
                if lhs_ty != ValueType::Bool {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "left operand of '{}' must be bool, got {}",
                            format_binary_op(bin.op),
                            format_type(&lhs_ty)
                        ),
                    );
                }
                if rhs_ty != ValueType::Bool {
                    self.error(
                        bin.line,
                        bin.column,
                        format!(
                            "right operand of '{}' must be bool, got {}",
                            format_binary_op(bin.op),
                            format_type(&rhs_ty)
                        ),
                    );
                }
                ValueType::Bool
            }
        };

        self.check_annotation(bin.line, bin.column, &bin.value_type, &result_ty);
        result_ty
    }

    fn check_unary(&mut self, un: &mut UnaryExpr<'a>) -> ValueType {
        let operand_ty = self.check_expression(&mut un.value);

        if operand_ty == ValueType::Undefined {
            return ValueType::Undefined;
        }

        let result_ty = match un.op {
            UnaryExprOp::Neg => {
                if !is_numeric(&operand_ty) {
                    self.error(
                        un.line,
                        un.column,
                        format!(
                            "unary '-' requires i64 or f64, got {}",
                            format_type(&operand_ty)
                        ),
                    );
                    ValueType::Undefined
                } else {
                    operand_ty
                }
            }
            UnaryExprOp::Not => {
                if operand_ty != ValueType::Bool {
                    self.error(
                        un.line,
                        un.column,
                        format!("unary '!' requires bool, got {}", format_type(&operand_ty)),
                    );
                    ValueType::Undefined
                } else {
                    ValueType::Bool
                }
            }
        };

        self.check_annotation(un.line, un.column, &un.value_type, &result_ty);
        result_ty
    }

    fn check_call(&mut self, call: &mut CallExpr<'a>) -> ValueType {
        // Resolve the callee name directly from a Variable node.
        let func_name = match &call.callee {
            Expression::Variable(v) => Some(v.name),
            _ => None,
        };

        let sig = func_name.and_then(|name| self.functions.get(name).cloned());

        // Check argument expressions regardless.
        let arg_types: Vec<ValueType> = call
            .arguments
            .iter_mut()
            .map(|a| self.check_expression(a))
            .collect();

        let return_ty = if let Some(sig) = sig {
            // Arity check.
            if arg_types.len() != sig.parameters.len() {
                self.error(
                    call.line,
                    call.column,
                    format!(
                        "function '{}' expects {} argument(s), got {}",
                        func_name.unwrap(),
                        sig.parameters.len(),
                        arg_types.len()
                    ),
                );
            } else {
                // Per-argument type check.
                for (i, (expected, actual)) in
                    sig.parameters.iter().zip(arg_types.iter()).enumerate()
                {
                    if !types_compatible(expected, actual) {
                        self.error(
                            call.line,
                            call.column,
                            format!(
                                "argument {} of '{}': expected {}, got {}",
                                i + 1,
                                func_name.unwrap(),
                                format_type(expected),
                                format_type(actual)
                            ),
                        );
                    }
                }
            }
            call.value_type = sig.return_type.clone();
            sig.return_type
        } else {
            if let Some(func_name) = func_name {
                self.error(
                    call.line,
                    call.column,
                    format!("call to undefined function '{}'", func_name),
                );
                
            } else {
                self.error(call.line, call.column, "callee must be a named function");
            }
            ValueType::Undefined
        };

        self.check_annotation(call.line, call.column, &call.value_type, &return_ty);
        return_ty
    }

    fn check_annotation(
        &mut self,
        line: usize,
        column: usize,
        annotated: &ValueType,
        inferred: &ValueType,
    ) {
        if matches!(annotated, ValueType::Undefined | ValueType::Any) {
            return;
        }
        if inferred == &ValueType::Undefined {
            return; // error already recorded upstream
        }
        if annotated != inferred {
            self.error(
                line,
                column,
                format!(
                    "AST annotation says {} but inferred {}",
                    format_type(annotated),
                    format_type(inferred)
                ),
            );
        }
    }
}

pub fn check_types<'a>(_statements: &mut [Statement<'a>]) {}

fn is_numeric(ty: &ValueType) -> bool {
    matches!(ty, ValueType::I64 | ValueType::F64)
}

/// `Any` is a wildcard that satisfies any type (for native function interop).
fn types_compatible(expected: &ValueType, actual: &ValueType) -> bool {
    expected == actual || expected == &ValueType::Any || actual == &ValueType::Any
}

fn require_defined(ty: &ValueType) -> Result<(), String> {
    match ty {
        ValueType::Undefined => Err("type is undefined".into()),
        _ => Ok(()),
    }
}

