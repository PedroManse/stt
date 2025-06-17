//! # Error handeling module

use super::*;
use std::collections::hash_map::{Entry, OccupiedEntry};

pub type Result<T> = std::result::Result<T, StckError>;
pub type ResultCtx<T> = std::result::Result<T, StckErrorCtx>;

/// # An error possibly with a context
///
/// Either an error with context [`StckErrorCtx`] or a
/// bubbled up and context-less error [`StckError`]
#[derive(thiserror::Error, Debug)]
pub enum StckErrorCase {
    #[error(transparent)]
    Context(#[from] StckErrorCtx),
    #[error(transparent)]
    Bubble(#[from] StckError),
}

/// # The context of a runtime error
///
/// Error Context, informing the source file's path, the expression
/// that caused the error and it's [span](`LineRange`)
///
/// Useful to [get the source code of the error](`ErrorSource`)
#[derive(Debug)]
pub struct ErrCtx {
    pub(crate) source: PathBuf,
    pub(crate) expr: Box<Expr>,
    pub(crate) lines: LineRange,
}

/// # A single viewable source file
///
/// made in bulk from the [stack trace](`ErrorSpans`) with [try into sources](`ErrorSpans::try_into_sources`)
pub struct ErrorSource {
    pub(crate) range: LineRange,
    pub(crate) source: PathBuf,
    pub(crate) lines: String,
}

/// # The entire call stack of an [error](`StckErrorCtx`)
///
/// Used to create viewable [sources](`ErrorSource`) of the error with [`ErrorSpans::try_into_sources`]
pub struct ErrorSpans {
    head: ErrCtx,
    stack: Vec<ErrCtx>,
}

impl ErrorSpans {
    /// # Get code from [error](`ErrCtx`)
    ///
    /// Read the source files with [`ErrorHelper`] and make [`ErrorSource`] for each [`ErrCtx`]
    /// entry
    pub fn try_into_sources(self) -> Result<Vec<ErrorSource>> {
        let mut error_helper = ErrorHelper::new();
        std::iter::once(self.head)
            .chain(self.stack)
            .map(|a| {
                Ok(ErrorSource {
                    lines: a.get_lines(&mut error_helper)?,
                    range: a.lines,
                    source: a.source,
                })
            })
            .collect()
    }
}

impl From<StckErrorCtx> for ErrorSpans {
    fn from(value: StckErrorCtx) -> Self {
        Self {
            head: value.ctx,
            stack: value.stack,
        }
    }
}

impl ErrCtx {
    #[must_use]
    pub fn new(source: &Path, expr: &Expr, lines: &LineSpan) -> Self {
        let lines = lines.line_range(expr.span.clone());
        Self {
            source: source.to_path_buf(),
            expr: Box::new(expr.clone()),
            lines,
        }
    }
    pub fn get_lines(&self, eh: &mut ErrorHelper) -> Result<String> {
        eh.get_span(&self.source, &self.lines)
            .map_err(StckError::from)
    }
}

/// # An error with context
///
/// An [error](`StckError`) with the faulty expression's [context](`ErrCtx`)
/// and the [stack trace](`StckErrorCtx::get_call_stack`)
#[derive(Debug)]
pub struct StckErrorCtx {
    pub(crate) ctx: ErrCtx,
    pub(crate) kind: Box<StckError>,
    pub(crate) stack: Vec<ErrCtx>,
}

impl StckErrorCtx {
    pub(crate) fn new(ctx: ErrCtx, kind: StckError) -> Self {
        Self {
            ctx,
            kind: Box::new(kind),
            stack: vec![],
        }
    }
    pub(crate) fn into_case(self) -> StckErrorCase {
        self.into()
    }
    #[must_use]
    pub(crate) fn append_stack(mut self, ctx: ErrCtx) -> Self {
        self.stack.push(ctx);
        self
    }
    #[must_use]
    pub fn get_call_stack(&self) -> &[ErrCtx] {
        &self.stack
    }
}

impl StckError {
    pub(crate) fn into_case(self) -> StckErrorCase {
        self.into()
    }
}

impl std::error::Error for StckErrorCtx {}

/// # An error without context
#[derive(thiserror::Error, Debug)]
pub enum StckError {
    #[error("Can't read file {0:?}")]
    CantReadFile(PathBuf),
    #[error("No such function or function argument called `{0}`")]
    MissingIdent(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
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
    CantElseCurrentSection(Range<usize>, Option<crate::preproc::ProcCommand>),
    #[error("Invalid pragma command: {0}")]
    InvalidPragma(String),
    #[error("Expected type: {0:?} got value {1:?}")]
    RTTypeError(TypeTester, Box<Value>),
    #[error("Output of function `{fn_name}` error, Expected {expected:?} got {got:?}")]
    RTOutputCountError {
        fn_name: String,
        expected: usize,
        got: usize,
    },
    #[error("Type `{0}` doesn't exist")]
    UnknownType(String),
    #[error("Unexpected end of file while building token {0:?}")]
    UnexpectedEOF(token::State),
    #[error("Tokenizer: No impl for {0:?} with {1:?}")]
    CantTokenizerChar(token::State, char),
    #[error("Parser in file {path}: Can only user param list or '*' as function arguments, not {0}", path=_1.display())]
    WrongParamList(String, PathBuf),
    #[error("Parser in file {path}: State {0:?} doesn't accept token {1:?}", path=_2.display())]
    CantParseToken(parse::State, Box<TokenCont>, PathBuf),
}

/// # Caching system for files
///
/// Used with [`LineRange`] to read specific lines from files on [get span](`ErrorHelper::get_span`)
#[derive(Default)]
pub struct ErrorHelper {
    files: HashMap<PathBuf, String>,
}

impl ErrorHelper {
    #[must_use]
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }
    fn read_file(
        &mut self,
        path: &PathBuf,
    ) -> OResult<OccupiedEntry<'_, PathBuf, String>, std::io::Error> {
        let entry = self.files.entry(path.clone());
        let entry = match entry {
            Entry::Occupied(entry) => entry,
            Entry::Vacant(entry) => {
                let cont = std::fs::read_to_string(path)?;
                entry.insert_entry(cont)
            }
        };
        Ok(entry)
    }
    pub fn get_span(
        &mut self,
        path: &PathBuf,
        lines: &LineRange,
    ) -> OResult<String, std::io::Error> {
        let entry = self.read_file(path)?;
        let lines: Vec<&str> = entry
            .get()
            .split('\n')
            .skip(lines.before - 1)
            .take(lines.during)
            .collect();
        Ok(lines.join("\n"))
    }
}

/// # The lines before and the amount of lines of a span
///
/// Made from a [line span](`LineSpan`) and the span of interest with [`LineSpan::line_range`]
///
/// Will be formated as "`before`" optionally with `:+amount` in the end if the span covers more
/// than one line. The result `before:+amount` can be used direcly with [bat](https://github.com/sharkdp/bat)
///
/// The [`LineRange`] can be used with an [`ErrorHelper`] to select specific lines to read from
/// files
#[derive(Debug, Default)]
pub struct LineRange {
    pub(crate) before: usize,
    pub(crate) during: usize,
}

impl LineRange {
    fn new() -> Self {
        Self {
            before: 1,
            during: 0,
        }
    }
    fn count(&mut self, is_before: bool) {
        if is_before {
            self.before += 1;
        } else {
            self.during += 1;
        }
    }
}

/// # The list of line breaks from a file
///
/// Used to make a [`LineRange`] with [line range](`LineSpan::line_range`)
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Default)]
pub struct LineSpan {
    feeds: BTreeSet<usize>,
}

impl LineSpan {
    #[must_use]
    pub fn new() -> Self {
        Self {
            feeds: BTreeSet::new(),
        }
    }
    pub fn add(&mut self, point: usize) {
        self.feeds.insert(point);
    }
    /// Makes the [`LineRange`] of a significant `span`
    #[must_use]
    pub fn line_range(&self, span: Range<usize>) -> LineRange {
        let mut range = LineRange::new();
        for point in self.feeds.iter().take_while(|&p| *p < span.end) {
            range.count(*point < span.start);
        }
        range
    }
}
