use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub meta: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub uses: Vec<UseDecl>,
    #[serde(default)]
    pub imports: Vec<ImportDecl>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseDecl {
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDecl {
    pub path: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Item {
    Flow(Flow),
    GlobalState(StateBlock),
    Rule(Rule),
    Nd(NdBlock),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flow {
    pub name: String,
    pub start: Option<String>,
    pub states: Vec<StateBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBlock {
    pub name: Option<String>,
    pub statements: Vec<StateStmt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateStmt {
    On {
        event: Call,
        target: String,
    },
    Rule(Rule),
    Run {
        flow: String,
    },
    Terminate,
    Assign {
        target: String,
        op: AssignOp,
        value: Expr,
    },
    Expr(Call),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignOp {
    Set,
    Add,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub params: Vec<String>,
    #[serde(default)]
    pub scope_flow: Option<String>,
    #[serde(default)]
    pub scope_state: Option<String>,
    pub trigger: RuleTrigger,
    pub body: Vec<RuleStmt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleTrigger {
    On(Call),
    When(Expr),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleStmt {
    Assign {
        target: String,
        op: AssignOp,
        value: Expr,
    },
    Emit {
        event: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdBlock {
    pub name: String,
    pub params: Vec<String>,
    pub propose: Call,
    pub constraints: Vec<Call>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(f64),
    String(String),
    Null,
    Ident(String),
    Call(Call),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
    Gte,
    Gt,
    Lt,
    Eq,
    Neq,
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub name: String,
    pub args: Vec<CallArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallArg {
    pub name: Option<String>,
    pub value: Expr,
}
