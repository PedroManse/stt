pub mod execute;
pub mod parse;
pub mod preproc;
pub mod token;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use self::token::Token;

#[derive(Clone, Debug)]
pub struct Code(pub Vec<Expr>);

impl Code {
    pub fn as_slice(&self) -> &[Expr] {
        &self.0
    }
}

// step token.rs
pub fn get_raw_tokens(file_path: &PathBuf) -> Result<Vec<Token>, ()> {
    let cont = std::fs::read_to_string(file_path).unwrap();
    token::Context::new(&cont).tokenize_block()
}

// step preproc.rs
pub fn preproc_tokens(tokens: Vec<Token>, file_path: &PathBuf) -> Result<Vec<Token>, ()> {
    let preprocessor = preproc::Context::new(file_path.parent().unwrap());
    preprocessor.parse(tokens)
}

// step parse.rs
pub fn parse_tokens(tokens: Vec<Token>) -> Result<Vec<Expr>, ()> {
    let mut parser = parse::Context::new(tokens);
    parser.parse_block()
}

pub fn execute_code(code: Code) -> Result<(), ()> {
    let mut executioner = execute::Context::new();
    executioner.execute_code(&code);
    Ok(())
}

// abstract
pub fn get_tokens(path: impl AsRef<Path>) -> Result<Vec<Token>, ()> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens(tokens, &file_path)
}

pub fn get_project_code(path: impl AsRef<Path>) -> Result<Code, ()> {
    let procced_block = get_tokens(path)?;
    let mut parser = parse::Context::new(procced_block);
    parser.parse_block().map(Code)
}

pub fn execute_file(path: impl AsRef<Path>) -> Result<(), ()> {
    let code = get_project_code(path)?;
    execute_code(code)
}

#[derive(Clone, Debug)]
pub enum FnArgs {
    Args(Vec<String>),
    AllStack,
}
#[derive(Clone, Debug)]
pub enum FnArgsIns {
    Args(HashMap<FnName, FnArg>),
    AllStack(Vec<Value>),
}
//pub struct FnArgs(pub Vec<String>);
#[derive(Debug)]
pub struct Stack(Vec<Value>);
#[derive(Debug, Clone)]
pub struct FnArg(pub Value);

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
    pub fn push_this(&mut self, v: impl Into<Value>) {
        self.0.push(v.into())
    }
    pub fn pushn(&mut self, mut vs: Vec<Value>) {
        self.0.append(&mut vs);
    }
    pub fn pop(&mut self) -> Option<Value> {
        self.0.pop()
    }
    pub fn peek(&mut self) -> Option<&Value> {
        self.0.get(self.len() - 1)
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
    pub fn take(&mut self) -> Vec<Value> {
        std::mem::replace(&mut self.0, Vec::new())
    }
    //TODO Stack.pop_mut
    //pub fn pop_mut<T, F, 'a>(&'a mut self, get_fn: F) -> Option<Result<&mut T, &mut Value>>
    //where
    //    F: Fn(&'a mut Value) -> Result<&'a mut T, &'a mut Value>,
    //{
    //    self.0.get_mut(self.len() - 1).map(get_fn)
    //}
    pub fn pop_this<T, F>(&mut self, get_fn: F) -> Option<Result<T, Value>>
    where
        F: Fn(Value) -> Result<T, Value>,
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
    Result(Box<Result<Value, Value>>),
    Null,
}

impl Value {
    pub fn get_result(self) -> Result<Result<Value, Value>, Value> {
        match self {
            Value::Result(x) => Ok(*x),
            o => Err(o),
        }
    }
    pub fn get_null(self) -> Result<(), Value> {
        match self {
            Value::Null => Ok(()),
            o => Err(o),
        }
    }
    pub fn get_str(self) -> Result<String, Value> {
        match self {
            Value::Str(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_num(self) -> Result<isize, Value> {
        match self {
            Value::Num(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_bool(self) -> Result<bool, Value> {
        match self {
            Value::Bool(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_arr(self) -> Result<Vec<Value>, Value> {
        match self {
            Value::Array(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_map(self) -> Result<HashMap<String, Value>, Value> {
        match self {
            Value::Map(x) => Ok(x),
            o => Err(o),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Str(value)
    }
}
impl From<isize> for Value {
    fn from(value: isize) -> Self {
        Value::Num(value)
    }
}
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}
impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::Array(value)
    }
}
impl From<HashMap<String, Value>> for Value {
    fn from(value: HashMap<String, Value>) -> Self {
        Value::Map(value)
    }
}
impl From<Result<Value, Value>> for Value {
    fn from(value: Result<Value, Value>) -> Self {
        Value::Result(Box::new(value))
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
    //Include {
    //    path: PathBuf,
    //},
}

#[derive(Clone, Debug)]
pub enum Expr {
    Immediate(Value),
    FnCall(FnName),
    Keyword(KeywordKind),
}
