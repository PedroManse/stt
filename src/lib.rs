mod api;
mod parse;
mod preproc;
mod runtime;
mod token;
pub use api::*;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

type OResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = std::result::Result<T, SttError>;

#[allow(private_interfaces)] // Allow private types since this should only be printed
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
        got: Box<Value>,
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
    #[error(
        "Closure's arguments ({:?}) are filled, but still tried to add more",
        closure_args
    )]
    DEVFillFullClosure { closure_args: ClosurePartialArgs },
    #[error(
        "Closure's arguments ({:?}) have been overwritten at [{}] previous value was {:?}",
        closure_args,
        index,
        removed
    )]
    DEVOverwrittenClosure {
        closure_args: ClosurePartialArgs,
        index: usize,
        removed: Value,
    },
}

#[derive(Clone, Debug)]
pub struct Code {
    source: PathBuf,
    exprs: Vec<Expr>,
}

#[derive(Clone, Debug)]
enum FnArgs {
    Args(Vec<String>),
    AllStack,
}

enum ClosureCurry {
    Full(FullClosure),
    Partial(Closure),
}

enum ClosureFillError {
    OutOfBound,
}

#[derive(Clone, Debug)]
struct ClosurePartialArgs {
    next_args: Vec<String>,
    filled_args: Vec<(String, Value)>,
}

impl ClosurePartialArgs {
    fn new(mut arg_list: Vec<String>) -> Self {
        arg_list.reverse();
        ClosurePartialArgs {
            filled_args: Vec::with_capacity(arg_list.len()),
            next_args: arg_list,
        }
    }
    fn fill(&mut self, value: Value) -> OResult<(), ClosureFillError> {
        let next = self.next_args.pop().ok_or(ClosureFillError::OutOfBound)?;
        self.filled_args.push((next, value));
        Ok(())
    }
    fn is_full(&self) -> bool {
        self.next_args.is_empty()
    }
}

#[derive(Clone, Debug)]
struct Closure {
    code: Vec<Expr>,
    request_args: ClosurePartialArgs,
}

struct FullClosure {
    code: Vec<Expr>,
    request_args: HashMap<FnName, FnArg>,
}

impl Closure {
    fn fill(mut self, value: Value) -> Result<ClosureCurry> {
        if let Err(r) = self.request_args.fill(value) {
            return Err(match r {
                ClosureFillError::OutOfBound => SttError::DEVFillFullClosure {
                    closure_args: self.request_args,
                },
            });
        }
        Ok(if self.request_args.is_full() {
            let args: HashMap<FnName, FnArg> = self
                .request_args
                .filled_args
                .into_iter()
                .map(|(k, v)| (FnName(k), FnArg(v)))
                .collect();
            ClosureCurry::Full(FullClosure {
                code: self.code,
                request_args: args,
            })
        } else {
            ClosureCurry::Partial(self)
        })
    }
}
impl PartialEq for Closure {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl FnArgs {
    fn into_vec(self) -> Vec<String> {
        match self {
            FnArgs::AllStack => vec![],
            FnArgs::Args(xs) => xs,
        }
    }
}

#[derive(Clone, Debug)]
struct FnArgsIns {
    cap: FnArgsInsCap,
    parent: Option<HashMap<FnName, FnArg>>,
}

#[derive(Clone, Debug)]
enum FnArgsInsCap {
    Args(HashMap<FnName, FnArg>),
    AllStack(Vec<Value>),
}

#[derive(Debug, Default)]
struct Stack(Vec<Value>);
#[derive(Debug, Clone)]
struct FnArg(Value);

impl Stack {
    fn new_with(v: Vec<Value>) -> Self {
        Self(v)
    }
    fn new() -> Self {
        Self(Vec::new())
    }
    fn push(&mut self, v: Value) {
        self.0.push(v)
    }
    fn push_this(&mut self, v: impl Into<Value>) {
        self.0.push(v.into())
    }
    fn pushn(&mut self, mut vs: Vec<Value>) {
        self.0.append(&mut vs);
    }
    fn pop(&mut self) -> Option<Value> {
        self.0.pop()
    }
    fn peek(&mut self) -> Option<&Value> {
        self.0.get(self.len() - 1)
    }
    fn popn(&mut self, n: usize) -> Option<Vec<Value>> {
        if n > self.len() {
            return None;
        }
        Some(self.0.split_off(self.len() - n))
    }
    fn into_vec(self) -> Vec<Value> {
        self.0
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn take(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.0)
    }
    fn pop_this<T, F>(&mut self, get_fn: F) -> Option<OResult<T, Value>>
    where
        F: Fn(Value) -> OResult<T, Value>,
    {
        self.pop().map(get_fn)
    }
    fn peek_this<T, F>(&mut self, get_fn: F) -> Option<OResult<&T, &Value>>
    where
        F: Fn(&Value) -> OResult<&T, &Value>,
    {
        self.peek().map(get_fn)
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct FnName(String);

impl FnName {
    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
enum FnScope {
    Global,   // read and writes to upper-scoped variables
    Local,    // reads upper-scoped variables
    Isolated, // fully isolated
}

#[derive(Clone, Debug)]
struct FnDef {
    scope: FnScope,
    code: Vec<Expr>,
    args: FnArgs,
}

impl FnDef {
    fn new(scope: FnScope, code: Vec<Expr>, args: FnArgs) -> Self {
        FnDef { scope, code, args }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Value {
    Str(String),
    Num(isize),
    Bool(bool),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    Result(Box<OResult<Value, Value>>),
    Option(Option<Box<Value>>),
    Closure(Box<Closure>),
}

impl Value {
    fn get_option(self) -> OResult<Option<Box<Value>>, Value> {
        match self {
            Value::Option(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_result(self) -> OResult<OResult<Value, Value>, Value> {
        match self {
            Value::Result(x) => Ok(*x),
            o => Err(o),
        }
    }
    fn get_closure(self) -> OResult<Closure, Value> {
        match self {
            Value::Closure(x) => Ok(*x),
            o => Err(o),
        }
    }
    fn get_str(self) -> OResult<String, Value> {
        match self {
            Value::Str(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_num(self) -> OResult<isize, Value> {
        match self {
            Value::Num(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_bool(self) -> OResult<bool, Value> {
        match self {
            Value::Bool(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_arr(self) -> OResult<Vec<Value>, Value> {
        match self {
            Value::Array(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_map(self) -> OResult<HashMap<String, Value>, Value> {
        match self {
            Value::Map(x) => Ok(x),
            o => Err(o),
        }
    }

    fn get_ref_option(&self) -> OResult<&Option<Box<Value>>, &Value> {
        match self {
            Value::Option(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_ref_result(&self) -> OResult<&OResult<Value, Value>, &Value> {
        match self {
            Value::Result(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_ref_closure(&self) -> OResult<&Closure, &Value> {
        match self {
            Value::Closure(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_ref_str(&self) -> OResult<&String, &Value> {
        match self {
            Value::Str(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_ref_num(&self) -> OResult<&isize, &Value> {
        match self {
            Value::Num(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_ref_bool(&self) -> OResult<&bool, &Value> {
        match self {
            Value::Bool(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_ref_arr(&self) -> OResult<&Vec<Value>, &Value> {
        match self {
            Value::Array(x) => Ok(x),
            o => Err(o),
        }
    }
    fn get_ref_map(&self) -> OResult<&HashMap<String, Value>, &Value> {
        match self {
            Value::Map(x) => Ok(x),
            o => Err(o),
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
impl From<Closure> for Value {
    fn from(value: Closure) -> Self {
        Value::Closure(Box::new(value))
    }
}

#[derive(Clone, Debug)]
struct CondBranch {
    check: Vec<Expr>,
    code: Vec<Expr>,
}

#[derive(Clone, Debug)]
enum KeywordKind {
    Break,
    Return,
    BubbleError,
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
struct SwitchCase {
    test: Value,
    code: Vec<Expr>,
}

#[derive(Clone, Debug)]
struct Expr {
    #[allow(dead_code)]
    span: Range<usize>,
    cont: ExprCont,
}

// TODO use closure as own kind of expr, to enable argument capture
#[derive(Clone, Debug)]
enum ExprCont {
    Immediate(Value),
    FnCall(FnName),
    Keyword(KeywordKind),
    IncludedCode(Code),
}

enum ControlFlow {
    Continue,
    Break,
    Return,
}

#[derive(Debug, PartialEq)]
enum RawKeyword {
    BubbleError,
    Return,
    Fn(FnScope),
    Ifs,
    While,
    Include { path: PathBuf },
    Pragma { command: String },
    Switch,
    Break,
}

#[derive(Debug, PartialEq)]
struct Token {
    cont: TokenCont,
    span: Range<usize>,
}

#[derive(Debug, PartialEq)]
enum TokenCont {
    Ident(String),
    Str(String),
    Number(isize),
    Keyword(RawKeyword),
    FnArgs(Vec<String>),
    Block(Vec<Token>),
    IncludedBlock(TokenBlock),
    EndOfBlock,
}

#[derive(Debug, PartialEq)]
pub struct TokenBlock {
    source: PathBuf,
    tokens: Vec<Token>,
}
