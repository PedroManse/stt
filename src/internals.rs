//! # Internal items of the crate
//!
//! Used for more precise control or error recovery

use super::*;

pub use runtime::Context as RuntimeContext;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Debug)]
pub struct Code {
    pub(crate) line_breaks: LineSpan,
    pub(crate) source: PathBuf,
    pub(crate) exprs: Vec<Expr>,
}

impl Code {
    #[must_use]
    pub fn new(source: PathBuf, exprs: Vec<Expr>, line_breaks: LineSpan) -> Self {
        Code {
            line_breaks,
            source,
            exprs,
        }
    }
    #[must_use]
    pub fn expr_count(&self) -> usize {
        self.exprs.len()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Expr> {
        self.exprs.iter()
    }
}

impl<'p> IntoIterator for &'p Code {
    type Item = &'p Expr;
    type IntoIter = std::slice::Iter<'p, Expr>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub struct FnArgDef {
    pub(crate) name: String,
    pub(crate) type_check: Option<TypeTester>,
}

impl FnArgDef {
    pub(crate) fn new_untyped(name: String) -> Self {
        Self {
            name,
            type_check: None,
        }
    }
    pub(crate) fn new_typed(name: String, type_check: TypeTester) -> Self {
        Self {
            name,
            type_check: Some(type_check),
        }
    }
    #[must_use]
    pub fn new(name: String, type_check: Option<TypeTester>) -> Self {
        Self { name, type_check }
    }
    #[must_use]
    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }
    pub(crate) fn get_type(&self) -> Option<&TypeTester> {
        self.type_check.as_ref()
    }
    pub(crate) fn take_type(self) -> Option<TypeTester> {
        self.type_check
    }
    fn take_name(self) -> String {
        self.name
    }
    pub(crate) fn check(&self, v: &FnArg) -> Result<(), TypeTester> {
        match self.type_check.as_ref() {
            Some(tt) => tt.check(&v.0),
            None => Ok(()),
        }
    }
    fn check_raw(&self, v: &Value) -> Result<(), TypeTester> {
        match self.type_check.as_ref() {
            Some(tt) => tt.check(v),
            None => Ok(()),
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Debug)]
pub enum FnArgs {
    Args(Vec<FnArgDef>),
    AllStack,
}

pub(crate) enum ClosureCurry {
    Full(FullClosure),
    Partial(Closure),
}

#[derive(Debug)]
enum ClosureFillError {
    OutOfBound,
    TypeError(TypeTester, Value),
}

#[derive(Clone, Debug)]
pub struct ClosurePartialArgs {
    pub(crate) next: Vec<FnArgDef>,
    pub(crate) filled: Vec<(ArgName, Value)>,
    parent: OnceCell<HashMap<ArgName, FnArg>>,
}

impl ClosurePartialArgs {
    pub fn get_unfilled_args(&self) -> &[FnArgDef] {
        &self.next
    }
    pub fn take_unfilled_args(self) -> Vec<FnArgDef> {
        self.next
    }
    pub fn get_unfilled_args_count(&self) -> usize {
        self.next.len()
    }
    fn set_parent(&self, args: HashMap<String, FnArg>) -> Result<(), HashMap<ArgName, FnArg>> {
        self.parent.set(args)
    }
    #[must_use]
    fn new(mut arg_list: Vec<FnArgDef>) -> Self {
        arg_list.reverse();
        ClosurePartialArgs {
            filled: Vec::with_capacity(arg_list.len()),
            next: arg_list,
            parent: OnceCell::new(),
        }
    }
    pub fn parse(arg_list: Vec<FnArgDef>, span: Range<usize>) -> Result<Self, StckError> {
        if arg_list.is_empty() {
            Err(StckError::CantInstanceClosureZeroArgs { span })
        } else {
            Ok(Self::new(arg_list))
        }
    }
    pub fn convert(arg_list: Vec<FnArgDef>, fn_name: &str) -> Result<Self, RuntimeErrorKind> {
        if arg_list.is_empty() {
            Err(RuntimeErrorKind::CantMakeFnIntoClosureZeroArgs {
                fn_name: fn_name.to_string(),
            })
        } else {
            Ok(Self::new(arg_list))
        }
    }
    fn fill(&mut self, value: Value) -> Result<(), ClosureFillError> {
        let next = self.next.pop().ok_or(ClosureFillError::OutOfBound)?;
        if let Err(tt) = next.check_raw(&value) {
            return Err(ClosureFillError::TypeError(tt, value));
        }
        self.filled.push((next.take_name(), value));
        Ok(())
    }
    fn is_full(&self) -> bool {
        self.next.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct Closure {
    pub(crate) code: Vec<Expr>,
    pub(crate) request_args: ClosurePartialArgs,
    pub(crate) output_types: Option<TypedOutputs>,
}

pub(crate) struct FullClosure {
    pub(crate) code: Vec<Expr>,
    pub(crate) request_args: HashMap<ArgName, FnArg>,
}

impl Closure {
    pub(crate) fn get_unfilled_args_count(&self) -> usize {
        self.get_args().get_unfilled_args_count()
    }
    pub(crate) fn get_output_types(&self) -> Option<&TypedOutputs> {
        self.output_types.as_ref()
    }
    pub(crate) fn get_args(&self) -> &ClosurePartialArgs {
        &self.request_args
    }
    pub fn set_parent_args(
        &self,
        args: HashMap<String, FnArg>,
    ) -> Result<(), HashMap<String, FnArg>> {
        self.request_args.set_parent(args)
    }
    pub(crate) fn fill(mut self, value: Value) -> Result<ClosureCurry, RuntimeErrorKind> {
        if let Err(r) = self.request_args.fill(value) {
            return Err(match r {
                ClosureFillError::OutOfBound => RuntimeErrorKind::DEVFillFullClosure {
                    closure_args: self.request_args,
                },
                ClosureFillError::TypeError(tt, v) => RuntimeErrorKind::Type(tt, Box::new(v)),
            });
        }
        Ok(if self.request_args.is_full() {
            let args = if let Some(parent_args) = self.request_args.parent.get() {
                let mut closure_args = parent_args.clone();
                for (k, v) in self.request_args.filled {
                    closure_args.insert(k, FnArg(v));
                }
                closure_args
            } else {
                self.request_args
                    .filled
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
    #[must_use]
    pub fn into_vec(self) -> Vec<FnArgDef> {
        match self {
            FnArgs::AllStack => vec![],
            FnArgs::Args(xs) => xs,
        }
    }

    #[must_use]
    pub fn into_needs(self) -> Vec<String> {
        match self {
            FnArgs::AllStack => vec![],
            FnArgs::Args(xs) => xs.into_iter().map(|x| x.name).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FnArgsInsCap {
    Args(HashMap<ArgName, FnArg>),
    AllStack(Vec<Value>),
}

#[derive(Debug, Default)]
pub struct Stack(Vec<Value>);
#[derive(Debug, Clone)]
pub struct FnArg(pub Value);

impl Stack {
    pub(crate) fn new_with(v: Vec<Value>) -> Self {
        Self(v)
    }
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push(&mut self, v: Value) {
        self.0.push(v);
    }
    pub fn push_this(&mut self, v: impl Into<Value>) {
        self.0.push(v.into());
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
    #[must_use]
    pub fn as_slice(&self) -> &[Value] {
        &self.0
    }
    pub(crate) fn into_vec(self) -> Vec<Value> {
        self.0
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub(crate) fn take(&mut self) -> Vec<Value> {
        std::mem::take(&mut self.0)
    }
    pub fn pop_this<T, F>(&mut self, get_fn: F) -> Option<Result<T, Value>>
    where
        F: Fn(Value) -> Result<T, Value>,
    {
        self.pop().map(get_fn)
    }
    pub fn peek_this<T, F>(&mut self, get_fn: F) -> Option<Result<&T, &Value>>
    where
        F: Fn(&Value) -> Result<&T, &Value>,
    {
        self.peek().map(get_fn)
    }
}

pub(crate) type ArgName = String;
pub(crate) type FnName = String;

#[derive(Clone, Debug, PartialEq)]
pub enum FnScope {
    Global,   // read and writes to upper-scoped variables
    Local,    // reads upper-scoped variables
    Isolated, // fully isolated
}

#[derive(Clone, Debug)]
pub(crate) struct FnDef {
    pub(crate) scope: FnScope,
    pub(crate) code: Vec<Expr>,
    pub(crate) args: FnArgs,
    pub(crate) output_types: Option<TypedOutputs>,
}

impl FnDef {
    pub(crate) fn new(
        scope: FnScope,
        code: Vec<Expr>,
        args: FnArgs,
        output_types: Option<TypedOutputs>,
    ) -> Self {
        FnDef {
            scope,
            code,
            args,
            output_types,
        }
    }
    pub fn into_closure(self, name: &str) -> Result<Closure, RuntimeErrorKind> {
        let args = match self.args {
            FnArgs::AllStack => Err(RuntimeErrorKind::CantMakeFnIntoClosureAllStack {
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
    Result(Box<Result<Value, Value>>),
    Option(Option<Box<Value>>),
    Closure(Box<Closure>),
}

impl Value {
    pub fn get_option(self) -> Result<Option<Box<Value>>, Value> {
        match self {
            Value::Option(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_result(self) -> Result<Result<Value, Value>, Value> {
        match self {
            Value::Result(x) => Ok(*x),
            o => Err(o),
        }
    }
    pub fn get_closure(self) -> Result<Closure, Value> {
        match self {
            Value::Closure(x) => Ok(*x),
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

    pub fn get_ref_option(&self) -> Result<&Option<Box<Value>>, &Value> {
        match self {
            Value::Option(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_result(&self) -> Result<&Result<Value, Value>, &Value> {
        match self {
            Value::Result(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_closure(&self) -> Result<&Closure, &Value> {
        match self {
            Value::Closure(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_str(&self) -> Result<&String, &Value> {
        match self {
            Value::Str(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_num(&self) -> Result<&isize, &Value> {
        match self {
            Value::Num(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_bool(&self) -> Result<&bool, &Value> {
        match self {
            Value::Bool(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_arr(&self) -> Result<&Vec<Value>, &Value> {
        match self {
            Value::Array(x) => Ok(x),
            o => Err(o),
        }
    }
    pub fn get_ref_map(&self) -> Result<&HashMap<String, Value>, &Value> {
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
impl From<Result<Value, Value>> for Value {
    fn from(value: Result<Value, Value>) -> Self {
        Value::Result(Box::new(value))
    }
}
impl From<Closure> for Value {
    fn from(value: Closure) -> Self {
        Value::Closure(Box::new(value))
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Debug)]
pub struct CondBranch {
    pub(crate) check: Vec<Expr>,
    pub(crate) code: Vec<Expr>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Debug)]
pub enum KeywordKind {
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
        out_args: Option<Vec<FnArgDef>>,
    },
    Switch {
        cases: Vec<SwitchCase>,
        default: Option<Vec<Expr>>,
    },
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Debug)]
pub struct SwitchCase {
    pub(crate) test: Value,
    pub(crate) code: Vec<Expr>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Debug)]
pub struct Expr {
    pub(crate) span: Range<usize>,
    pub cont: ExprCont,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Clone, Debug)]
pub enum ExprCont {
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

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct Token {
    pub(crate) cont: TokenCont,
    pub(crate) span: Range<usize>,
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
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
/// Usually created by [`api::get_tokens`] for files or [`api::get_tokens_str`] for raw strings.
/// The token array ends with a [`TokenCont::EndOfBlock`] token, to indicate either the end of the
/// source string or a `}` that closed the code block
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug)]
pub struct TokenBlock {
    pub(crate) line_breaks: LineSpan,
    pub(crate) source: PathBuf,
    pub(crate) tokens: Vec<Token>,
}

impl<'p> IntoIterator for &'p TokenBlock {
    type Item = &'p Token;
    type IntoIter = std::slice::Iter<'p, Token>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl TokenBlock {
    #[must_use]
    pub fn token_count(&self) -> usize {
        self.tokens.len() - usize::from(self.last_is_eof())
    }
    #[must_use]
    pub fn last_is_eof(&self) -> bool {
        self.tokens
            .last()
            .is_some_and(|e| matches!(e.cont, TokenCont::EndOfBlock))
    }
    pub fn iter(&self) -> std::slice::Iter<'_, Token> {
        self.tokens.iter()
    }
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Token> {
        self.tokens.get(index)
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
    #[must_use]
    pub fn new(name: String, code: RustStckFnRaw) -> Self {
        RustStckFn { name, code }
    }
    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn call(&self, ctx: &mut runtime::Context, source: &Path) {
        (self.code)(ctx, source);
    }
}

impl std::fmt::Debug for RustStckFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rust function {}", self.name)
    }
}
