pub mod execute;

use std::collections::HashMap;

#[derive(Clone)]
pub struct Code(pub Vec<Expr>);

impl Code {
    pub fn as_slice(&self) -> &[Expr] {
        &self.0
    }
}

#[derive(Clone)]
pub struct FnArgs(pub Vec<String>);
pub struct Stack(Vec<Value>);
pub struct FnArg(pub Value);

impl FnArgs {
    fn into_vec(self) -> Vec<String> {
        self.0
    }
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Stack {
    pub fn new_with(v: Vec<Value>) -> Self {
        Self(v)
    }
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(&mut self, v: Value) {
        self.0.push(v)
    }
    pub fn pushn(&mut self, mut vs: Vec<Value>) {
        self.0.append(&mut vs);
    }
    pub fn pop(&mut self) -> Option<Value> {
        self.0.pop()
    }
    pub fn popn(&mut self, n: usize) -> Result<Vec<Value>, Vec<Value>> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            match self.pop() {
                Some(v)=>out.push(v),
                None=>return Err(out),
            }
        }
        Ok(out)
    }
    pub fn merge(&mut self, other: Self) {
        self.pushn(other.0);
    }
    pub fn into_vec(self) -> Vec<Value> {
        self.0
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct FnName(pub String);

impl FnName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone)]
pub enum FnScope {
    Global, // read and writes to upper-scoped variables
    Local, // reads upper-scoped variables
    Isolated, // fully isolated
}

#[derive(Clone)]
pub struct FnDef {
    pub scope: FnScope,
    pub code: Code,
    pub args: FnArgs,
}

impl FnDef {
    pub fn new(scope: FnScope, code: Code, args: FnArgs) -> Self {
        FnDef { scope, code, args }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Str(String),
    Num(i64),
    Bool(bool),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
}

#[derive(Clone)]
pub struct CondBranch {
    pub check: Code,
    pub code: Code,
}

pub enum RawKeyword {
    Ifs,
    If,
    Elseif,
    Else,
    While,
}

#[derive(Clone)]
pub enum KeywordKind {
    Ifs {
        branches: Vec<CondBranch>,
    },
    If {
        if_branch: CondBranch,
        else_code: Code,
    },
    While {
        check: Code,
        code: Code,
    },
    FnDef{
        name: FnName,
        scope: FnScope,
        code: Code,
        args: FnArgs,
    },
}

#[derive(Clone)]
pub enum Expr {
    Immediate(Value),
    FnCall(FnName),
    Keyword(KeywordKind),
}

