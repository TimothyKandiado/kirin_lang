pub enum Expression {
    None,
    Binary(Box<BinaryExpr>),
    Unary(Box<UnaryExpr>),
    Literal(LiteralExpr),
    Grouping(Box<GroupingExpr>),
    Call(Box<CallExpr>),
    Assign(Box<AssignExpr>)
}

pub struct BinaryExpr {
    pub line: usize,
    pub column: usize,
    pub op: BinaryExprOp,
    pub left: Expression,
    pub right: Expression,
    pub value_type: ValueType,
}

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


pub struct UnaryExpr {
    pub line: usize,
    pub column: usize,
    pub op: UnaryExprOp,
    pub value: Expression,
    pub value_type: ValueType,
}

pub enum UnaryExprOp {
    Neg,
    Not,
}

pub struct CallExpr {
    pub line: usize,
    pub column: usize,
    pub callee: Expression,
    pub arguments: Vec<Expression>,
    pub value_type: ValueType
}

pub struct AssignExpr {
    pub name: String,
    pub value: Expression,
    pub line: usize,
    pub column: usize
}

pub struct GroupingExpr {
    pub line: usize,
    pub column: usize,
    pub expression: Expression,
    pub value_type: ValueType
}

pub struct LiteralExpr {
    pub line: usize,
    pub column: usize,
    pub value: LiteralValue,
    pub value_type: ValueType
}

pub struct VariableExpr {
    pub line: usize,
    pub column: usize,
    pub name: String,
    pub value_type: ValueType
}

pub enum LiteralValue {
    I64(i64),
    F64(f64),
    String(String),
    Bool(bool),
}

pub enum ValueType {
    Undefined,
    I64,
    F64,
    String,
    Bool,
    Void,
    Any,
}

pub enum Statement {
    None,
    PackageDecl(PackageDeclstmt),
    FunctionDecl(Box<FunctionDeclStmt>),
    If(Box<IfStmt>),
    Block(BlockStmt),
    Return(ReturnStmt),
    VarDecl(VarDeclStmt),
    Expr(ExprStmt)
}

pub struct VarDeclStmt {
    pub name: String,
    pub value: Option<Expression>,
    pub line: usize,
    pub column: usize,
    pub value_type: ValueType,
}

pub enum FunctionDeclStmt {
    Native(NativeFuncDecl),
    UserFunc(UserFuncDecl),
}

pub struct NativeFuncDecl {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub line: usize,
    pub column: usize,
    pub value_type: ValueType,
}

pub struct UserFuncDecl {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub body: Statement,
    pub line: usize,
    pub column: usize,
    pub value_type: ValueType,
}

pub struct FuncParam {
    pub name: String,
    pub value_type: ValueType,
}

pub struct IfStmt {
    pub condition: Expression,
    pub then_branch: Statement,
    pub else_branch: Option<Statement>,
    pub line: usize,
    pub column: usize,
}

pub struct PackageDeclstmt {
    pub name: String,
    pub line: usize,
    pub column: usize
}

pub struct BlockStmt {
    pub statements: Vec<Statement>,
    pub line: usize,
    pub column: usize,
}

pub struct ExprStmt {
    pub expression: Expression,
    pub line: usize,
    pub column: usize,
}

pub struct ReturnStmt {
    pub value: Option<Expression>,
    pub line: usize,
    pub column: usize,
}

