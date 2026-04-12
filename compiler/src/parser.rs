use std::fmt::{self};

use crate::lexer::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum Expression<'a> {
    None,
    Binary(Box<BinaryExpr<'a>>),
    Unary(Box<UnaryExpr<'a>>),
    Literal(LiteralExpr<'a>),
    Grouping(Box<GroupingExpr<'a>>),
    Call(Box<CallExpr<'a>>),
    Assign(Box<AssignExpr<'a>>),
    Variable(VariableExpr<'a>),
}

impl Expression<'_> {
    pub fn get_value_type(&self) -> ValueType {
        match self {
            Self::Assign(_) => ValueType::Void,
            Self::Binary(bin) => bin.value_type.clone(),
            Self::Unary(un) => un.value_type.clone(),
            Self::Literal(lit) => lit.value_type.clone(),
            Self::Grouping(group) => group.value_type.clone(),
            Self::Call(call) => call.value_type.clone(),
            Self::Variable(var) => var.value_type.clone(),
            Self::None => ValueType::Undefined,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryExpr<'a> {
    pub line: usize,
    pub column: usize,
    pub op: BinaryExprOp,
    pub left: Expression<'a>,
    pub right: Expression<'a>,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum BinaryExprOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Equal,
    NotEqual,

    And,
    Or,
}

impl BinaryExprOp {
    pub fn from_token(token: &Token) -> Result<Self, ParseError> {
        match token.kind {
            TokenKind::Plus => Ok(BinaryExprOp::Add),
            TokenKind::Minus => Ok(BinaryExprOp::Sub),
            TokenKind::Slash => Ok(BinaryExprOp::Div),
            TokenKind::Star => Ok(BinaryExprOp::Mul),
            TokenKind::Mod => Ok(BinaryExprOp::Mod),
            TokenKind::Caret => Ok(BinaryExprOp::Pow),

            TokenKind::Less => Ok(BinaryExprOp::Less),
            TokenKind::LessEqual => Ok(BinaryExprOp::LessEqual),
            TokenKind::Greater => Ok(BinaryExprOp::Greater),
            TokenKind::GreaterEqual => Ok(BinaryExprOp::GreaterEqual),
            TokenKind::EqualEqual => Ok(BinaryExprOp::Equal),
            TokenKind::NotEqual => Ok(BinaryExprOp::NotEqual),

            TokenKind::Or => Ok(BinaryExprOp::Or),
            TokenKind::And => Ok(BinaryExprOp::And),

            _ => Err(ParseError {
                line: token.line,
                column: token.column,
                context: format!("token {:?} is not a valid binary operator", token.kind),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnaryExpr<'a> {
    pub line: usize,
    pub column: usize,
    pub op: UnaryExprOp,
    pub value: Expression<'a>,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum UnaryExprOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub struct CallExpr<'a> {
    pub line: usize,
    pub column: usize,
    pub callee: Expression<'a>,
    pub arguments: Vec<Expression<'a>>,
    pub value_type: ValueType,
}

#[derive(Debug, Clone)]
pub struct AssignExpr<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct GroupingExpr<'a> {
    pub line: usize,
    pub column: usize,
    pub expression: Expression<'a>,
    pub value_type: ValueType,
}

#[derive(Debug, Clone)]
pub struct LiteralExpr<'a> {
    pub line: usize,
    pub column: usize,
    pub value: LiteralValue<'a>,
    pub value_type: ValueType,
}

#[derive(Debug, Clone)]
pub struct VariableExpr<'a> {
    pub line: usize,
    pub column: usize,
    pub name: &'a str,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LiteralValue<'a> {
    I64(i64),
    F64(f64),
    String(&'a str),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ValueType {
    Undefined,
    I64,
    F64,
    String,
    Bool,
    Void,
    Any,
    Fn(Box<FunctionSignature>),
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            ValueType::I64 => "i64".to_string(),
            ValueType::F64 => "f64".to_string(),
            ValueType::Bool => "bool".to_string(),
            ValueType::String => "str".to_string(),
            ValueType::Void => "void".to_string(),
            ValueType::Any => "any".to_string(),
            ValueType::Undefined => "undefined".to_string(),
            ValueType::Fn(func_sign) => {
                let mut params = Vec::new();
                for param in &func_sign.parameters {
                    params.push(param.to_string());
                }

                format!("fn ({}): {}", params.join(", "), func_sign.return_type)
            }
        };
        write!(f, "{}", val)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FunctionSignature {
    pub parameters: Vec<ValueType>,
    pub return_type: ValueType,
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    None,
    PackageDecl(PackageDeclstmt<'a>),
    FunctionDecl(Box<FunctionDeclStmt<'a>>),
    If(Box<IfStmt<'a>>),
    Block(BlockStmt<'a>),
    Return(ReturnStmt<'a>),
    VarDecl(VarDeclStmt<'a>),
    Expr(Expression<'a>),
}

#[derive(Debug, Clone)]
pub struct VarDeclStmt<'a> {
    pub name: &'a str,
    pub value: Option<Expression<'a>>,
    pub line: usize,
    pub column: usize,
    pub value_type: ValueType,
}

#[derive(Debug, Clone)]
pub enum FunctionDeclStmt<'a> {
    Native {
        name: &'a str,
        params: Vec<FuncParam<'a>>,
        line: usize,
        column: usize,
        return_type: ValueType,
    },
    Bytecode {
        name: &'a str,
        params: Vec<FuncParam<'a>>,
        body: Statement<'a>,
        line: usize,
        column: usize,
        return_type: ValueType,
    },
}

impl<'a> FunctionDeclStmt<'a> {
    pub fn get_name(&self) -> &'a str {
        match self {
            Self::Native {
                name,
                params: _,
                line: _,
                column: _,
                return_type: _,
            } => name,
            Self::Bytecode {
                name,
                params: _,
                line: _,
                column: _,
                return_type: _,
                body: _,
            } => name,
        }
    }

    pub fn get_return_type(&self) -> ValueType {
        match self {
            Self::Native {
                name: _,
                params: _,
                line: _,
                column: _,
                return_type,
            } => return_type.clone(),
            Self::Bytecode {
                name: _,
                params: _,
                line: _,
                column: _,
                return_type,
                body: _,
            } => return_type.clone(),
        }
    }

    pub fn get_signature(&self) -> FunctionSignature {
        match self {
            Self::Native {
                name: _,
                params,
                line: _,
                column: _,
                return_type,
            } => {
                let parameters = params
                    .iter()
                    .map(|param| param.value_type.clone())
                    .collect::<Vec<ValueType>>();

                FunctionSignature {
                    parameters,
                    return_type: return_type.clone(),
                }
            }
            Self::Bytecode {
                name: _,
                params,
                line: _,
                column: _,
                return_type,
                body: _,
            } => {
                let parameters = params
                    .iter()
                    .map(|param| param.value_type.clone())
                    .collect::<Vec<ValueType>>();

                FunctionSignature {
                    parameters,
                    return_type: return_type.clone(),
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct FuncParam<'a> {
    pub name: &'a str,
    pub value_type: ValueType,
}

impl fmt::Display for FuncParam<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.value_type)
    }
}

#[derive(Debug, Clone)]
pub struct IfStmt<'a> {
    pub condition: Expression<'a>,
    pub then_branch: Statement<'a>,
    pub else_branch: Option<Statement<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct PackageDeclstmt<'a> {
    pub name: &'a str,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct BlockStmt<'a> {
    pub statements: Vec<Statement<'a>>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt<'a> {
    pub value: Option<Expression<'a>>,
    pub line: usize,
    pub column: usize,
}

pub fn parse_ast(tokens: Vec<Token<'_>>) -> Result<Vec<Statement<'_>>, Vec<ParseError>> {
    let parser = Parser {
        tokens,
        current: 0,
        statements: Vec::new(),
        errors: Vec::new(),
    };

    parser.parse_statements()
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub context: String,
}

struct Parser<'a> {
    pub tokens: Vec<Token<'a>>,
    pub current: usize,
    pub statements: Vec<Statement<'a>>,
    pub errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    pub fn parse_statements(mut self) -> Result<Vec<Statement<'a>>, Vec<ParseError>> {
        while !self.is_at_end() {
            let statement = self.declaration();

            match statement {
                Ok(stmt) => self.statements.push(stmt),
                Err(err) => {
                    self.errors.push(err);
                    self.synchronize();
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(self.statements)
    }

    fn synchronize(&mut self) {
        _ = self.advance();

        while !self.is_at_end() {
            if self.previous().kind == TokenKind::NewLine {
                return;
            }

            match self.peek().kind {
                TokenKind::Fn | TokenKind::If => return,

                _ => {}
            }

            _ = self.advance();
        }
    }

    fn declaration(&mut self) -> Result<Statement<'a>, ParseError> {
        let modifiers = self.parse_modifiers();

        if self.match_tokens(&[TokenKind::Package]) {
            return self.package_decl();
        } else if self.match_tokens(&[TokenKind::Fn]) {
            return self.func_decl(modifiers);
        } else if self.check(TokenKind::Identifier) && self.check_next(TokenKind::Colon) {
            return self.var_decl();
        }

        self.statement()
    }

    fn parse_modifiers(&mut self) -> Vec<TokenKind> {
        let mut modifiers = Vec::new();

        while !self.is_at_end() {
            let current = self.peek();

            match current.kind {
                TokenKind::Pub | TokenKind::Native => {
                    modifiers.push(current.kind);
                }

                _ => break,
            }

            _ = self.advance();
        }

        modifiers
    }

    fn package_decl(&mut self) -> Result<Statement<'a>, ParseError> {
        let name = self.consume(TokenKind::Identifier, "expected package name".to_string())?;

        _ = self.consume(
            TokenKind::NewLine,
            "expected new line after package name".to_string(),
        )?;

        Ok(Statement::PackageDecl(PackageDeclstmt {
            name: name.lexeme,
            line: name.line,
            column: name.column,
        }))
    }

    fn var_decl(&mut self) -> Result<Statement<'a>, ParseError> {
        let name = self.consume(TokenKind::Identifier, "expected variable name".to_string())?;
        _ = self.consume(
            TokenKind::Colon,
            "expect ':' after variable name".to_string(),
        )?;

        let var_type = self.parse_type()?;

        _ = self.consume(
            TokenKind::Equal,
            "expected '=' after variable type".to_string(),
        )?;

        let value = self.expression()?;

        _ = self.consume(
            TokenKind::NewLine,
            "expected new line after var declaration".to_string(),
        )?;

        let var_decl = VarDeclStmt {
            name: name.lexeme,
            value: Some(value),
            line: name.line,
            column: name.line,
            value_type: var_type,
        };

        Ok(Statement::VarDecl(var_decl))
    }

    fn func_decl(&mut self, modifiers: Vec<TokenKind>) -> Result<Statement<'a>, ParseError> {
        let name = self.consume(TokenKind::Identifier, "expected function name".to_string())?;
        _ = self.consume(
            TokenKind::ParenLeft,
            "expected '(' after function name".to_string(),
        )?;

        let mut parameters = Vec::new();

        if !self.check(TokenKind::ParenRight) {
            while !self.is_at_end() {
                let param_name =
                    self.consume(TokenKind::Identifier, "expected parameter name".to_string())?;

                _ = self.consume(
                    TokenKind::Colon,
                    "expected ':' after parameter name".to_string(),
                )?;

                let param_type = self.parse_type()?;

                let param = FuncParam {
                    name: param_name.lexeme,
                    value_type: param_type,
                };

                parameters.push(param);

                if !self.match_tokens(&[TokenKind::Comma]) {
                    break;
                }
            }
        }

        _ = self.consume(
            TokenKind::ParenRight,
            "expected ')' after function parameters".to_string(),
        )?;
        _ = self.consume(TokenKind::Colon, "expected ':' after ')'".to_string())?;

        let return_type = self.parse_type()?;

        self.skip(TokenKind::NewLine);
        if modifiers.contains(&TokenKind::Native) {
            let native_func = FunctionDeclStmt::Native {
                name: name.lexeme,
                line: name.line,
                column: name.column,
                params: parameters,
                return_type,
            };

            return Ok(Statement::FunctionDecl(Box::new(native_func)));
        }

        let body = self.statement()?;

        let user_defined_func = FunctionDeclStmt::Bytecode {
            name: name.lexeme,
            line: name.line,
            column: name.column,
            params: parameters,
            body,
            return_type,
        };

        Ok(Statement::FunctionDecl(Box::new(user_defined_func)))
    }

    fn statement(&mut self) -> Result<Statement<'a>, ParseError> {
        if self.match_tokens(&[TokenKind::If]) {
            return self.if_stmt();
        }

        if self.match_tokens(&[TokenKind::BraceLeft]) {
            return self.block_stmt();
        }

        if self.match_tokens(&[TokenKind::Return]) {
            return self.return_stmt();
        }

        self.expression_stmt()
    }

    fn if_stmt(&mut self) -> Result<Statement<'a>, ParseError> {
        let prev = self.previous();

        let condition = self.expression()?;
        self.skip(TokenKind::NewLine);

        let then_branch = self.statement()?;

        let mut else_branch = None;

        if self.match_tokens(&[TokenKind::Else]) {
            else_branch = Some(self.statement()?);
        }

        let if_stmt = IfStmt {
            line: prev.line,
            column: prev.column,
            condition,
            then_branch,
            else_branch,
        };

        Ok(Statement::If(Box::new(if_stmt)))
    }

    fn block_stmt(&mut self) -> Result<Statement<'a>, ParseError> {
        let mut statements = Vec::new();
        let prev = self.previous();

        self.skip(TokenKind::NewLine);

        while !self.check(TokenKind::BraceRight) && !self.is_at_end() {
            let statement = self.declaration()?;

            statements.push(statement);
        }

        _ = self.consume(
            TokenKind::BraceRight,
            "expected '}' after end of block".to_string(),
        )?;
        self.skip(TokenKind::NewLine);

        let block_stmt = BlockStmt {
            line: prev.line,
            column: prev.column,
            statements,
        };

        Ok(Statement::Block(block_stmt))
    }

    fn return_stmt(&mut self) -> Result<Statement<'a>, ParseError> {
        let prev = self.previous();

        let mut value = None;

        if !self.check(TokenKind::NewLine) {
            value = Some(self.expression()?);
        }

        _ = self.consume(
            TokenKind::NewLine,
            "expect newline after return statement".to_string(),
        )?;

        Ok(Statement::Return(ReturnStmt {
            value,
            line: prev.line,
            column: prev.column,
        }))
    }

    fn expression_stmt(&mut self) -> Result<Statement<'a>, ParseError> {
        let expr = self.expression()?;

        _ = self.consume(
            TokenKind::NewLine,
            "expected newline after expression".to_string(),
        )?;

        Ok(Statement::Expr(expr))
    }

    fn parse_type(&mut self) -> Result<ValueType, ParseError> {
        let token = self.advance();

        match token.kind {
            TokenKind::I64 => Ok(ValueType::I64),
            TokenKind::F64 => Ok(ValueType::F64),
            TokenKind::Bool => Ok(ValueType::Bool),
            TokenKind::Void => Ok(ValueType::Void),
            TokenKind::Str => Ok(ValueType::String),
            TokenKind::Any => Ok(ValueType::Any),

            _ => Err(ParseError {
                line: token.line,
                column: token.column,
                context: format!("'{:?}' is not a valid type", token.kind),
            }),
        }
    }

    fn expression(&mut self) -> Result<Expression<'a>, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.or_expr()?;

        if self.match_tokens(&[TokenKind::Equal]) {
            let prev = self.previous();
            let value = self.assignment()?;

            match expr {
                Expression::Variable(var) => {
                    let assign_expr = AssignExpr {
                        line: prev.line,
                        column: prev.column,
                        name: var.name,
                        value,
                    };

                    return Ok(Expression::Assign(Box::new(assign_expr)));
                }

                _ => {
                    return Err(ParseError {
                        line: prev.line,
                        column: prev.column,
                        context: format!("invalid assignment target {:?}", expr),
                    });
                }
            }
        }

        Ok(expr)
    }

    fn or_expr(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.and_expr()?;

        if self.match_tokens(&[TokenKind::Or]) {
            let prev = self.previous();

            let op = BinaryExprOp::from_token(&prev)?;
            let right = self.unary()?;

            let binary_expr = BinaryExpr {
                line: prev.line,
                column: prev.column,
                op,
                left: expr,
                right,
                value_type: ValueType::Undefined,
            };

            return Ok(Expression::Binary(Box::new(binary_expr)));
        }

        Ok(expr)
    }

    fn and_expr(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.equality()?;

        if self.match_tokens(&[TokenKind::And]) {
            let prev = self.previous();

            let op = BinaryExprOp::from_token(&prev)?;
            let right = self.unary()?;

            let binary_expr = BinaryExpr {
                line: prev.line,
                column: prev.column,
                op,
                left: expr,
                right,
                value_type: ValueType::Bool,
            };

            return Ok(Expression::Binary(Box::new(binary_expr)));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.comparison()?;

        if self.match_tokens(&[TokenKind::EqualEqual, TokenKind::NotEqual]) {
            let prev = self.previous();

            let op = BinaryExprOp::from_token(&prev)?;
            let right = self.unary()?;

            let binary_expr = BinaryExpr {
                line: prev.line,
                column: prev.column,
                op,
                left: expr,
                right,
                value_type: ValueType::Bool,
            };

            return Ok(Expression::Binary(Box::new(binary_expr)));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.addition()?;

        if self.match_tokens(&[
            TokenKind::Less,
            TokenKind::LessEqual,
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::EqualEqual,
        ]) {
            let prev = self.previous();

            let op = BinaryExprOp::from_token(&prev)?;
            let right = self.unary()?;

            let binary_expr = BinaryExpr {
                line: prev.line,
                column: prev.column,
                op,
                left: expr,
                right,
                value_type: ValueType::Bool,
            };

            return Ok(Expression::Binary(Box::new(binary_expr)));
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.multiplication()?;

        if self.match_tokens(&[TokenKind::Plus, TokenKind::Minus]) {
            let prev = self.previous();

            let op = BinaryExprOp::from_token(&prev)?;
            let right = self.unary()?;

            let binary_expr = BinaryExpr {
                line: prev.line,
                column: prev.column,
                op,
                left: expr,
                right,
                value_type: ValueType::Undefined,
            };

            return Ok(Expression::Binary(Box::new(binary_expr)));
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.power()?;

        if self.match_tokens(&[TokenKind::Slash, TokenKind::Star, TokenKind::Mod]) {
            let prev = self.previous();

            let op = BinaryExprOp::from_token(&prev)?;
            let right = self.unary()?;

            let binary_expr = BinaryExpr {
                line: prev.line,
                column: prev.column,
                op,
                left: expr,
                right,
                value_type: ValueType::Undefined,
            };

            return Ok(Expression::Binary(Box::new(binary_expr)));
        }

        Ok(expr)
    }

    fn power(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.unary()?;

        if self.match_tokens(&[TokenKind::Caret]) {
            let prev = self.previous();

            let op = BinaryExprOp::from_token(&prev)?;
            let right = self.unary()?;

            let binary_expr = BinaryExpr {
                line: prev.line,
                column: prev.column,
                op,
                left: expr,
                right,
                value_type: ValueType::Undefined,
            };

            return Ok(Expression::Binary(Box::new(binary_expr)));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expression<'a>, ParseError> {
        if self.match_tokens(&[TokenKind::Minus]) {
            let prev = self.previous();

            let right = self.unary()?;
            let value_type = right.get_value_type();

            let unary_expr = UnaryExpr {
                line: prev.line,
                column: prev.column,
                op: UnaryExprOp::Neg,
                value: right,
                value_type,
            };

            return Ok(Expression::Unary(Box::new(unary_expr)));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expression<'a>, ParseError> {
        let expr = self.primary()?;

        if self.match_tokens(&[TokenKind::ParenLeft]) {
            let mut arguments = Vec::new();
            let prev = self.previous();

            if !self.check(TokenKind::ParenRight) {
                while !self.is_at_end() {
                    let arg = self.expression()?;

                    arguments.push(arg);
                    if !self.match_tokens(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            _ = self.consume(
                TokenKind::ParenRight,
                "expected ')' after call arguments".to_string(),
            )?;

            let call_expr = CallExpr {
                line: prev.line,
                column: prev.column,
                callee: expr,
                arguments,
                value_type: ValueType::Undefined,
            };

            return Ok(Expression::Call(Box::new(call_expr)));
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expression<'a>, ParseError> {
        let token = self.advance();

        match token.kind {
            TokenKind::Identifier => {
                let var_expr = VariableExpr {
                    name: token.lexeme,
                    line: token.line,
                    column: token.column,
                    value_type: ValueType::Undefined,
                };

                Ok(Expression::Variable(var_expr))
            }

            TokenKind::StringLiteral => {
                let literal_expr = LiteralExpr {
                    line: token.line,
                    column: token.column,
                    value: LiteralValue::String(token.lexeme),
                    value_type: ValueType::String,
                };

                Ok(Expression::Literal(literal_expr))
            }

            TokenKind::NumberLiteral => {
                let number_result = token.lexeme.parse::<f64>();

                match number_result {
                    Ok(value) => {
                        if value.fract() == 0.0 {
                            let literal_expr = LiteralExpr {
                                line: token.line,
                                column: token.column,
                                value: LiteralValue::I64(value as i64),
                                value_type: ValueType::I64,
                            };

                            return Ok(Expression::Literal(literal_expr));
                        }

                        let literal_expr = LiteralExpr {
                            line: token.line,
                            column: token.column,
                            value: LiteralValue::F64(value),
                            value_type: ValueType::F64,
                        };

                        Ok(Expression::Literal(literal_expr))
                    }

                    Err(err) => Err(ParseError {
                        line: token.line,
                        column: token.column,
                        context: format!(
                            "'{}' is not a valid number format: {:?}",
                            token.lexeme, err
                        ),
                    }),
                }
            }

            TokenKind::ParenLeft => {
                let expr = self.expression()?;
                let value_type = expr.get_value_type();

                _ = self.consume(
                    TokenKind::ParenRight,
                    "expected ')' after grouping".to_string(),
                )?;

                let grouping_expr = GroupingExpr {
                    line: token.line,
                    column: token.column,
                    expression: expr,
                    value_type,
                };

                Ok(Expression::Grouping(Box::new(grouping_expr)))
            }

            _ => Err(ParseError {
                line: token.line,
                column: token.column,
                context: format!("unhandled primary token {:?}", token),
            }),
        }
    }

    fn skip(&mut self, kind: TokenKind) {
        while !self.is_at_end() {
            if !self.match_tokens(&[kind]) {
                break;
            }
        }
    }

    fn consume(&mut self, kind: TokenKind, error_message: String) -> Result<Token<'a>, ParseError> {
        if self.check(kind) {
            return Ok(self.advance());
        }

        let current = self.peek();
        Err(ParseError {
            line: current.line,
            column: current.column,
            context: error_message,
        })
    }

    fn match_tokens(&mut self, kinds: &[TokenKind]) -> bool {
        for &kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn check_next(&self, kind: TokenKind) -> bool {
        self.peek_next().kind == kind
    }

    fn previous(&self) -> Token<'a> {
        if self.current == 0 {
            return Token::none();
        }

        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> Token<'a> {
        if self.is_at_end() {
            return Token::none();
        }

        self.tokens[self.current].clone()
    }

    fn peek_next(&self) -> Token<'a> {
        if self.current + 1 >= self.tokens.len() {
            return Token::none();
        }

        self.tokens[self.current + 1].clone()
    }

    fn advance(&mut self) -> Token<'a> {
        if self.is_at_end() {
            return Token::none();
        }

        self.current += 1;
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        if self.current >= self.tokens.len() {
            return true;
        }

        self.tokens[self.current].kind == TokenKind::Eof
    }
}

pub fn format_type(ty: &ValueType) -> String {
    ty.to_string()
}

pub fn format_binary_op(op: BinaryExprOp) -> &'static str {
    match op {
        BinaryExprOp::Add => "+",
        BinaryExprOp::Sub => "-",
        BinaryExprOp::Mul => "*",
        BinaryExprOp::Div => "/",
        BinaryExprOp::Mod => "%",
        BinaryExprOp::Pow => "^",
        BinaryExprOp::Greater => ">",
        BinaryExprOp::GreaterEqual => ">=",
        BinaryExprOp::Less => "<",
        BinaryExprOp::LessEqual => "<=",
        BinaryExprOp::Equal => "==",
        BinaryExprOp::NotEqual => "!=",
        BinaryExprOp::And => "and",
        BinaryExprOp::Or => "or",
    }
}
