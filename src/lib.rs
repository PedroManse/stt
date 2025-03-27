pub mod execute;
pub mod parse;
pub mod token;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Code(pub Vec<Expr>);

impl Code {
    pub fn as_slice(&self) -> &[Expr] {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct FnArgs(pub Vec<String>);
#[derive(Debug)]
pub struct Stack(Vec<Value>);
#[derive(Debug)]
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
    pub fn peek(&mut self) -> Option<&Value> {
        self.0.get(self.len()-1)
    }
    pub fn popn(&mut self, n: usize) -> Result<Vec<Value>, Vec<Value>> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            match self.pop() {
                Some(v) => out.push(v),
                None => return Err(out),
            }
        }
        //TODO figure out better way to make this
        out.reverse();
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
    pub fn pop_this<T, F>(&mut self, get_fn: F) -> Option<Result<T, Value>>
        where F: Fn(Value) -> Result<T, Value>
    {
        self.pop().map(get_fn)
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct FnName(pub String);

impl FnName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub enum FnScope {
    Global,   // read and writes to upper-scoped variables
    Local,    // reads upper-scoped variables
    Isolated, // fully isolated
}

#[derive(Clone, Debug)]
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
    Num(isize),
    Bool(bool),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl Value {
    pub fn get_str(self) -> Result<String, Value> {
        match self {
            Value::Str(x)=>Ok(x),
            o => Err(o),
        }
    }
    pub fn get_num(self) -> Result<isize, Value> {
        match self {
            Value::Num(x)=>Ok(x),
            o => Err(o),
        }
    }
    pub fn get_bool(self) -> Result<bool, Value> {
        match self {
            Value::Bool(x)=>Ok(x),
            o => Err(o),
        }
    }
    pub fn get_arr(self) -> Result<Vec<Value>, Value> {
        match self {
            Value::Array(x)=>Ok(x),
            o => Err(o),
        }
    }
    pub fn get_map(self) -> Result<HashMap<String, Value>, Value> {
        match self {
            Value::Map(x)=>Ok(x),
            o => Err(o),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CondBranch {
    pub check: Code,
    pub code: Code,
}

#[derive(Clone, Debug)]
pub enum KeywordKind {
    Ifs {
        branches: Vec<CondBranch>,
    },
    While {
        check: Code,
        code: Code,
    },
    FnDef {
        name: FnName,
        scope: FnScope,
        code: Code,
        args: FnArgs,
    },
}

#[derive(Clone, Debug)]
pub enum Expr {
    Immediate(Value),
    FnCall(FnName),
    Keyword(KeywordKind),
}
