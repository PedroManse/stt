pub mod runtime;
pub mod parse;
pub mod preproc;
pub mod token;

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use self::runtime::ExecMode;

pub type OResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = std::result::Result<T, SttError>;

#[derive(thiserror::Error, Debug)]
pub enum SttError {
    #[error("Can't read file {0:?}")]
    CantReadFile(PathBuf),
    #[error("No such function or function argument called `{0}`")]
    MissingIdent(String),
    #[error("WrongStackSizeDiffOnCheck {old_stack_size} -> {new_stack_size}")]
    WrongStackSizeDiffOnCheck {
        old_stack_size: usize,
        new_stack_size: usize,
        new_should_stack_size: usize,
    },
    #[error("check blocks must recieve one boolean, recieved {got:?}")]
    WrongTypeOnCheck { got: Value },
    #[error("Function {for_fn} accepts [{args}]. But {this_arg} is missing")]
    MissingValueForBuiltin {
        for_fn: String,
        args: String,
        this_arg: &'static str,
    },
    #[error("Function {for_fn} accepts {args}. But {missing} args are missing")]
    MissingValuesForBuiltin {
        for_fn: String,
        args: &'static str,
        missing: isize,
    },
    #[error(
        "Function {for_fn} accepts {args}. But [{this_arg}] must be a {expected} but got {got:?}"
    )]
    WrongTypeForBuiltin {
        for_fn: String,
        args: &'static str,
        this_arg: &'static str,
        got: Value,
        expected: &'static str,
    },
    #[error(
        "This error should never be elevated to users, if this happens to you, please report it"
    )]
    NoSuchBuiltin,
    #[error("The variable {0} is not defined")]
    NoSuchVariable(String),
    #[error("Missing char")]
    MissingChar,
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("TODO")]
    TodoErr,
    #[error("Not enough arguments to execute {name}, got {got:?} needs {needs:?}")]
    RTUserFnMissingArgs {
        name: String,
        got: Vec<Value>,
        needs: Vec<String>,
    },
    #[error("Found error while executing `!` on a Result: {error:?}")]
    RTUnwrapResultBuiltinFailed { error: Value },
    #[error("Found missing value while exeuting `!` on an Option")]
    RTUnwrapOptionBuiltinFailed,
    #[error("Can't compare {this:?} with {that:?}")]
    RTCompareError { this: Value, that: Value },
    #[error(
        "`%` doesn't recognise the format directive {0}, only '%', 'd', 's' and 'b' are avaliable "
    )]
    RTUnknownStringFormat(char),
    #[error("Switch case with no value")]
    RTSwitchCaseWithNoValue,
}

#[derive(Clone, Debug)]
pub struct Code{
    pub source: PathBuf,
    pub exprs: Vec<Expr>,
}

impl Code {
    pub fn as_slice(&self) -> &[Expr] {
        &self.exprs
    }
}

// step token.rs
pub fn get_raw_tokens(file_path: &Path) -> Result<TokenBlock> {
    let Ok(cont) = std::fs::read_to_string(file_path) else {
        return Err(SttError::CantReadFile(file_path.to_path_buf()));
    };
    let tokens = token::Context::new(&cont).tokenize_block()?;
    Ok(TokenBlock {
        tokens,
        source: file_path.into(),
    })
}

// step preproc.rs
pub fn preproc_tokens(
    TokenBlock { tokens, source }: TokenBlock,
    file_path: &Path,
) -> Result<TokenBlock> {
    let cwd = PathBuf::from(".");
    let preprocessor = preproc::Context::new(file_path.parent().unwrap_or(cwd.as_path()));
    let tokens = preprocessor.parse_clean(tokens)?;
    Ok(TokenBlock { tokens, source })
}

pub fn preproc_tokens_with_vars(
    TokenBlock { tokens, source }: TokenBlock,
    file_path: &Path,
    vars: &mut HashSet<String>,
) -> Result<TokenBlock> {
    let cwd = PathBuf::from(".");
    let preprocessor = preproc::Context::new(file_path.parent().unwrap_or(cwd.as_path()));
    let tokens = preprocessor.parse(tokens, vars)?;
    Ok(TokenBlock { tokens, source })
}

// step parse.rs
pub fn parse_tokens(TokenBlock { tokens, source }: TokenBlock) -> Result<Code> {
    let mut parser = parse::Context::new(tokens);
    let exprs = parser.parse_block()?;
    Ok(Code{
        exprs,
        source,
    })
}

pub fn execute_code(code: Code, mode: ExecMode) -> Result<()> {
    let mut executioner = runtime::Context::new(mode);
    executioner.execute_code(&code.exprs, &code.source)?;
    Ok(())
}

// abstract
pub fn get_tokens(path: impl AsRef<Path>) -> Result<TokenBlock> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens(tokens, &file_path)
}

pub fn get_tokens_with_procvars(
    path: impl AsRef<Path>,
    proc_vars: &mut HashSet<String>,
) -> Result<TokenBlock> {
    let file_path = PathBuf::from(path.as_ref());
    let tokens = get_raw_tokens(&file_path)?;
    preproc_tokens_with_vars(tokens, &file_path, proc_vars)
}

pub fn get_project_code(path: impl AsRef<Path>) -> Result<Code> {
    let TokenBlock { tokens, source } = get_tokens(path)?;
    let mut parser = parse::Context::new(tokens);
    let exprs = parser.parse_block()?;
    Ok(Code { exprs, source })
}

pub fn execute_file(path: impl AsRef<Path>, mode: ExecMode) -> Result<()> {
    let expr_block = get_project_code(path)?;
    execute_code(expr_block, mode)
}

#[derive(Clone, Debug)]
pub enum FnArgs {
    Args(Vec<String>),
    AllStack,
}

impl FnArgs {
    pub fn into_vec(self) -> Vec<String> {
        match self {
            FnArgs::AllStack => vec![],
            FnArgs::Args(xs) => xs,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FnArgsIns {
    cap: FnArgsInsCap,
    parent: Option<HashMap<FnName, FnArg>>,
}

#[derive(Clone, Debug)]
pub enum FnArgsInsCap {
    Args(HashMap<FnName, FnArg>),
    AllStack(Vec<Value>),
}
//pub struct FnArgs(pub Vec<String>);
#[derive(Debug, Default)]
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
    pub fn popn(&mut self, n: usize) -> Option<Vec<Value>> {
        if n > self.len() {
            return None;
        }
        Some(self.0.split_off(self.len() - n))
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
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn take(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.0)
    }
    pub fn pop_this<T, F>(&mut self, get_fn: F) -> Option<OResult<T, Value>>
    where
        F: Fn(Value) -> OResult<T, Value>,
    {
        self.pop().map(get_fn)
    }
    pub fn peek_this<T, F>(&mut self, get_fn: F) -> Option<OResult<&T, &Value>>
    where
        F: Fn(&Value) -> OResult<&T, &Value>,
    {
        self.peek().map(get_fn)
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
    pub code: Vec<Expr>,
    pub args: FnArgs,
}

impl FnDef {
    pub fn new(scope: FnScope, code: Vec<Expr>, args: FnArgs) -> Self {
        FnDef { scope, code, args }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Num(isize),
    Bool(bool),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Result(Box<OResult<Value, Value>>),
    Option(Option<Box<Value>>),
}

impl Value {
    pub fn get_option(self) -> OResult<Option<Box<Value>>, Value> {
        match self {
            Value::Option(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_result(self) -> OResult<OResult<Value, Value>, Value> {
        match self {
            Value::Result(x) => Ok(*x),
            o => Err(o),
        }
    }
    pub fn get_str(self) -> OResult<String, Value> {
        match self {
            Value::Str(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_num(self) -> OResult<isize, Value> {
        match self {
            Value::Num(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_bool(self) -> OResult<bool, Value> {
        match self {
            Value::Bool(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_arr(self) -> OResult<Vec<Value>, Value> {
        match self {
            Value::Array(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_map(self) -> OResult<HashMap<String, Value>, Value> {
        match self {
            Value::Map(x) => Ok(x),
            o => Err(o),
        }
    }

    pub fn get_ref_option(&self) -> OResult<&Option<Box<Value>>, &Value> {
        match self {
            Value::Option(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_result(&self) -> OResult<&OResult<Value, Value>, &Value> {
        match self {
            Value::Result(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_str(&self) -> OResult<&String, &Value> {
        match self {
            Value::Str(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_num(&self) -> OResult<&isize, &Value> {
        match self {
            Value::Num(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_bool(&self) -> OResult<&bool, &Value> {
        match self {
            Value::Bool(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_arr(&self) -> OResult<&Vec<Value>, &Value> {
        match self {
            Value::Array(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_map(&self) -> OResult<&HashMap<String, Value>, &Value> {
        match self {
            Value::Map(x) => Ok(x),
            o => Err(o),
        }
    }

    pub fn type_name(&self) -> &'static str {
        use Value::*;
        match self {
            Str(_) => "String",
            Num(_) => "Number",
            Bool(_) => "Boolean",
            Array(_) => "Array",
            Map(_) => "Map",
            Result(_) => "Result",
            Option(_) => "Option",
        }
    }
}

impl From<Option<Value>> for Value {
    fn from(value: Option<Value>) -> Self {
        Value::Option(value.map(Box::new))
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
impl From<OResult<Value, Value>> for Value {
    fn from(value: OResult<Value, Value>) -> Self {
        Value::Result(Box::new(value))
    }
}

#[derive(Clone, Debug)]
pub struct CondBranch {
    pub check: Vec<Expr>,
    pub code: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub enum KeywordKind {
    Break,
    Return,
    Ifs {
        branches: Vec<CondBranch>,
    },
    While {
        check: Vec<Expr>,
        code: Vec<Expr>,
    },
    FnDef {
        name: FnName,
        scope: FnScope,
        code: Vec<Expr>,
        args: FnArgs,
    },
    Switch {
        cases: Vec<SwitchCase>,
        default: Option<Vec<Expr>>,
    },
}

#[derive(Clone, Debug)]
pub struct SwitchCase {
    test: Value,
    code: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub struct Expr {
    span: Range<usize>,
    cont: ExprCont,
}

#[derive(Clone, Debug)]
pub enum ExprCont {
    Immediate(Value),
    FnCall(FnName),
    Keyword(KeywordKind),
    IncludedCode(Code),
}

#[derive(Debug)]
pub enum RawKeyword {
    Return,
    Fn(FnScope),
    Ifs,
    While,
    Include { path: PathBuf },
    Pragma { command: String },
    Switch,
    Break,
}

#[derive(Debug)]
pub struct Token {
    cont: TokenCont,
    span: Range<usize>,
}

#[derive(Debug)]
pub enum TokenCont {
    Ident(String),
    Str(String),
    Number(isize),
    Keyword(RawKeyword),
    FnArgs(Vec<String>),
    Block(Vec<Token>),
    IncludedBlock(TokenBlock),
    EndOfBlock,
}

#[derive(Debug)]
pub struct TokenBlock {
    source: PathBuf,
    tokens: Vec<Token>,
}
