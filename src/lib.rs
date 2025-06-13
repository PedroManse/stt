pub mod api;
mod parse;
mod preproc;
mod runtime;
mod token;
pub use runtime::Context;

#[cfg(test)]
mod tests;

use std::cell::OnceCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use self::preproc::ProcCommand;

type OResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = std::result::Result<T, StckError>;

#[allow(private_interfaces)] // Allow private types since this should only be printed
#[derive(thiserror::Error, Debug)]
pub enum StckError {
    #[error("Can't read file {0:?}")]
    CantReadFile(PathBuf),
    #[error("No such function or function argument called `{0}`")]
    MissingIdent(String),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("No such user-defined function `{0}`")]
    MissingUserFunction(String),
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
        "Function {for_fn} accepts {args}. But [{this_arg}] must be a {expected} and got {got:?}"
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
    #[error("Switch case with no value")]
    RTSwitchCaseWithNoValue,
    #[error(
        "Closure's arguments ({:?}) are filled, but still tried to add more",
        closure_args
    )]
    DEVFillFullClosure { closure_args: ClosurePartialArgs },
    #[error(
        "Closure's arguments ({closure_args:?})'s parent function values are beeing reset with {parent_args:?}"
    )]
    DEVResettingParentValuesForClosure {
        closure_args: Box<ClosurePartialArgs>,
        parent_args: HashMap<ArgName, FnArg>,
    },
    #[error(
        "Can't make function ({fn_name}) that takes no arguments into closure, since that would never be executed"
    )]
    CantMakeFnIntoClosureZeroArgs { fn_name: String },
    #[error(
        "Can't make function ({fn_name}) that takes entire stack into closure, since it would never be executed"
    )]
    CantMakeFnIntoClosureAllStack { fn_name: String },
    #[error("Can't make closure with zero arguments, it's code spans these bytes: {span:?}")]
    CantInstanceClosureZeroArgs { span: Range<usize> },
    #[error("Unknown keyword: {0}")]
    UnknownKeyword(String),
    #[error(
        "`%%` ({0}) doesn't recognise the format directive `{1}`, only '%', 'd', 's', 'v' and 'b' are avaliable"
    )]
    RTUnknownStringFormat(String, char),
    #[error("`%%` ({0}) Can't capture any value, the stack is empty")]
    RTMissingValue(String, char),
    #[error("`%%` ({0}) The provided value, {1:?}, can't be formatted with `{2}`")]
    RTWrongValueType(String, Value, char),
    #[error("No pragma section to (end if), on span {0:?}")]
    NoSectionToClose(Range<usize>),
    #[error("Can't start pragma (else) section on {1:?} (span {0:?})")]
    CantElseCurrentSection(Range<usize>, Option<ProcCommand>),
    #[error("Invalid pragma command: {0}")]
    InvalidPragma(String),
    #[error("Expected type: {0:?} got value {1:?}")]
    RTTypeError(TypeTester, Box<Value>),
    #[error("Type {0} doesn't exist")]
    UnknownType(String),
}

#[derive(Clone, Debug)]
pub struct Code {
    source: PathBuf,
    exprs: Vec<Expr>,
}

impl Code {
    #[must_use]
    pub fn expr_count(&self) -> usize {
        self.exprs.len()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnArgDef {
    name: String,
    type_check: Option<TypeTester>,
}

impl FnArgDef {
    fn new_untyped(name: String) -> Self {
        Self {
            name,
            type_check: None,
        }
    }
    fn new_typed(name: String, type_check: TypeTester) -> Self {
        Self {
            name,
            type_check: Some(type_check),
        }
    }
    pub fn new(name: String, type_check: Option<TypeTester>) -> Self {
        Self { name, type_check }
    }
    fn get_name(&self) -> &str {
        &self.name
    }
    fn take_name(self) -> String {
        self.name
    }
    fn check(&self, v: &FnArg) -> OResult<(), TypeTester> {
        match self.type_check.as_ref() {
            Some(tt) => tt.check(&v.0),
            None => Ok(()),
        }
    }
    fn check_raw(&self, v: &Value) -> OResult<(), TypeTester> {
        match self.type_check.as_ref() {
            Some(tt) => tt.check(v),
            None => Ok(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FnArgs {
    Args(Vec<FnArgDef>),
    AllStack,
}

enum ClosureCurry {
    Full(FullClosure),
    Partial(Closure),
}

enum ClosureFillError {
    OutOfBound,
    TypeError(TypeTester, Value),
}

#[derive(Clone, Debug)]
pub struct ClosurePartialArgs {
    next_args: Vec<FnArgDef>,
    filled_args: Vec<(ArgName, Value)>,
    parent_args: OnceCell<HashMap<ArgName, FnArg>>,
}

impl ClosurePartialArgs {
    pub fn new(mut arg_list: Vec<FnArgDef>) -> Self {
        arg_list.reverse();
        ClosurePartialArgs {
            filled_args: Vec::with_capacity(arg_list.len()),
            next_args: arg_list,
            parent_args: OnceCell::new(),
        }
    }
    pub fn parse(arg_list: Vec<FnArgDef>, span: Range<usize>) -> Result<Self> {
        if arg_list.is_empty() {
            Err(StckError::CantInstanceClosureZeroArgs { span })
        } else {
            Ok(Self::new(arg_list))
        }
    }
    pub fn convert(arg_list: Vec<FnArgDef>, fn_name: &str) -> Result<Self> {
        if arg_list.is_empty() {
            Err(StckError::CantMakeFnIntoClosureZeroArgs {
                fn_name: fn_name.to_string(),
            })
        } else {
            Ok(Self::new(arg_list))
        }
    }
    fn fill(&mut self, value: Value) -> OResult<(), ClosureFillError> {
        let next = self.next_args.pop().ok_or(ClosureFillError::OutOfBound)?;
        if let Err(tt) = next.check_raw(&value) {
            return Err(ClosureFillError::TypeError(tt, value));
        }
        self.filled_args.push((next.take_name(), value));
        Ok(())
    }
    fn is_full(&self) -> bool {
        self.next_args.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct Closure {
    pub code: Vec<Expr>,
    pub request_args: ClosurePartialArgs,
    pub output_types: Option<Vec<TypeTester>>,
}

struct FullClosure {
    code: Vec<Expr>,
    request_args: HashMap<ArgName, FnArg>,
}

impl Closure {
    fn fill(mut self, value: Value) -> Result<ClosureCurry> {
        if let Err(r) = self.request_args.fill(value) {
            return Err(match r {
                ClosureFillError::OutOfBound => StckError::DEVFillFullClosure {
                    closure_args: self.request_args,
                },
                ClosureFillError::TypeError(tt, v) => StckError::RTTypeError(tt, Box::new(v)),
            });
        }
        Ok(if self.request_args.is_full() {
            let args = if let Some(parent_args) = self.request_args.parent_args.get() {
                let mut closure_args = parent_args.clone();
                for (k, v) in self.request_args.filled_args {
                    closure_args.insert(k, FnArg(v));
                }
                closure_args
            } else {
                self.request_args
                    .filled_args
                    .into_iter()
                    .map(|(k, v)| (k, FnArg(v)))
                    .collect()
            };
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
    pub fn into_vec(self) -> Vec<FnArgDef> {
        match self {
            FnArgs::AllStack => vec![],
            FnArgs::Args(xs) => xs,
        }
    }

    pub fn into_needs(self) -> Vec<String> {
        match self {
            FnArgs::AllStack => vec![],
            FnArgs::Args(xs) => xs.into_iter().map(|x| x.name).collect(),
        }
    }
}

#[derive(Clone, Debug)]
struct FnArgsIns {
    cap: FnArgsInsCap,
}

#[derive(Clone, Debug)]
enum FnArgsInsCap {
    Args(HashMap<ArgName, FnArg>),
    AllStack(Vec<Value>),
}

#[derive(Debug, Default)]
pub struct Stack(Vec<Value>);
#[derive(Debug, Clone)]
struct FnArg(Value);

impl Stack {
    fn new_with(v: Vec<Value>) -> Self {
        Self(v)
    }
    fn new() -> Self {
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
    fn into_vec(self) -> Vec<Value> {
        self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    fn take(&mut self) -> Vec<Value> {
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

type ArgName = String;
type FnName = String;

#[derive(Clone, Debug, PartialEq)]
pub enum FnScope {
    Global,   // read and writes to upper-scoped variables
    Local,    // reads upper-scoped variables
    Isolated, // fully isolated
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeTester {
    Char,
    Str,
    Num,
    Bool,
    ArrayAny,
    MapAny,
    ResultAny,
    OptionAny,
    ClosureAny,
    Array(Box<TypeTester>),
    Map(Box<TypeTester>),
    Result(Box<(TypeTester, TypeTester)>),
    Option(Box<TypeTester>),
    Closure(Vec<TypeTester>, Vec<TypeTester>),
}

impl FromStr for TypeTester {
    type Err = StckError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "char" => Self::Char,
            "string" | "str" => Self::Str,
            "num" => Self::Num,
            "bool" => Self::Bool,
            "list" | "array" => Self::ArrayAny,
            "map" => Self::MapAny,
            "result" => Self::ResultAny,
            "option" => Self::OptionAny,
            "fn" | "closure" => Self::ClosureAny,
            // TODO: parse complex type or error out
            _ => return Err(StckError::UnknownType(s.to_string())),
        })
    }
}

impl TypeTester {
    pub fn check(&self, v: &Value) -> OResult<(), TypeTester> {
        match (self, v) {
            (Self::Char, Value::Char(_)) => Ok(()),
            (Self::Str, Value::Str(_)) => Ok(()),
            (Self::Num, Value::Num(_)) => Ok(()),
            (Self::Bool, Value::Bool(_)) => Ok(()),
            (Self::ArrayAny, Value::Array(_)) => Ok(()),
            (Self::MapAny, Value::Map(_)) => Ok(()),
            (Self::ResultAny, Value::Result(_)) => Ok(()),
            (Self::OptionAny, Value::Option(_)) => Ok(()),
            (Self::ClosureAny, Value::Closure(_)) => Ok(()),
            (Self::Array(tt), Value::Array(n)) => {
                n.iter()
                    .map(|v| tt.check(v))
                    .collect::<OResult<Vec<_>, _>>()?;
                Ok(())
            }
            (Self::Map(tt_value), Value::Map(m)) => {
                for value in m.values() {
                    tt_value.check(value)?;
                }
                Ok(())
            }
            (Self::Result(tt), Value::Result(v)) => {
                let (tt_ok, tt_err) = tt.as_ref();
                match v.as_ref() {
                    Ok(v_ok) => tt_ok.check(v_ok),
                    Err(v_err) => tt_err.check(v_err),
                }
            }
            (Self::Option(_), Value::Option(None)) => Ok(()),
            (Self::Option(tt), Value::Option(Some(v))) => tt.check(v),
            (Self::Closure(ttinput, ttoutput), Value::Closure(cl)) => {
                let outs = cl
                    .request_args
                    .next_args
                    .iter()
                    .map(|arg_def| &arg_def.type_check)
                    .zip(ttinput);
                for (cl_req, tt_req) in outs {
                    let ok = cl_req.as_ref().map(|c| c == tt_req).unwrap_or(true);
                    if !ok {
                        return Err(tt_req.clone());
                    }
                }
                let Some(outputs) = cl.output_types.as_ref() else {
                    return Ok(());
                };
                for (cl_in, tt_in) in outputs.iter().zip(ttoutput) {
                    if cl_in != tt_in {
                        return Err(tt_in.clone());
                    }
                }
                Ok(())
            }
            (t, _) => Err(t.clone()),
        }
    }
}

#[derive(Clone, Debug)]
struct FnDef {
    scope: FnScope,
    code: Vec<Expr>,
    args: FnArgs,
    output_types: Option<Vec<TypeTester>>,
}

impl FnDef {
    fn new(scope: FnScope, code: Vec<Expr>, args: FnArgs) -> Self {
        FnDef {
            scope,
            code,
            args,
            output_types: None,
        }
    }
    pub fn into_closure(self, name: &str) -> Result<Closure> {
        let args = match self.args {
            FnArgs::AllStack => Err(StckError::CantMakeFnIntoClosureAllStack {
                fn_name: name.to_string(),
            }),
            FnArgs::Args(a) => Ok(a),
        }?;
        Ok(Closure {
            code: self.code,
            request_args: ClosurePartialArgs::convert(args, name)?,
            output_types: self.output_types,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Char(char),
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
    pub fn get_closure(self) -> OResult<Closure, Value> {
        match self {
            Value::Closure(x) => Ok(*x),
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

impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::Char(value)
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
    IntoClosure {
        fn_name: FnName,
    },
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
pub struct Expr {
    #[allow(dead_code)]
    span: Range<usize>,
    cont: ExprCont,
}

#[derive(Clone, Debug)]
enum ExprCont {
    Immediate(Value),
    FnCall(FnName),
    Keyword(KeywordKind),
    IncludedCode(Code),
}

pub enum ControlFlow {
    Continue,
    Break,
    Return,
}

#[derive(Debug, PartialEq)]
pub enum RawKeyword {
    FnIntoClosure { fn_name: FnName },
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
pub struct Token {
    cont: TokenCont,
    span: Range<usize>,
}

#[derive(Debug, PartialEq)]
pub enum TokenCont {
    Char(char),
    Ident(String),
    Str(String),
    Number(isize),
    Keyword(RawKeyword),
    FnArgs(Vec<FnArgDef>),
    Block(Vec<Token>),
    IncludedBlock(TokenBlock),
    EndOfBlock,
}

/// # Array of tokens and their source
///
/// Usually created by [api::get_tokens] for files or [api::get_tokens_str] for raw strings.
/// The token array ends with a [TokenCont::EndOfBlock] token, to indicate either the end of the
/// source string or a `}` that closed the code block
#[derive(Debug, PartialEq)]
pub struct TokenBlock {
    source: PathBuf,
    tokens: Vec<Token>,
}

impl TokenBlock {
    pub fn token_count(&self) -> usize {
        self.tokens.len() - (if self.last_is_eof() { 1 } else { 0 })
    }
    pub fn last_is_eof(&self) -> bool {
        self.tokens
            .last()
            .map(|e| matches!(e.cont, TokenCont::EndOfBlock))
            .unwrap_or(false)
    }
}

type RustStckFnRaw = fn(&mut runtime::Context, &Path);
#[derive(Clone)]
pub struct RustStckFn {
    name: String,
    code: RustStckFnRaw,
}

// TODO ::new test if name is valid (for tokenizer)
impl RustStckFn {
    pub fn new(name: String, code: RustStckFnRaw) -> Self {
        RustStckFn { name, code }
    }
    fn call(&self, ctx: &mut runtime::Context, source: &Path) {
        (self.code)(ctx, source)
    }
}

impl Debug for RustStckFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rust function {}", self.name)
    }
}
