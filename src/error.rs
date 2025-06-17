use super::*;
use colored::Colorize;
use std::fmt::Display;

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
#[derive(Debug)]
pub struct ErrCtx {
    source: PathBuf,
    expr: Box<Expr>,
    lines: LineRange,
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
}

impl Display for ErrCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} in {}:{}",
            self.expr.cont,
            self.source.display().to_string().green(),
            self.lines.to_string().bright_magenta().underline(),
        )
    }
}

/// # An error with context
///
/// An [error](`StckError`) with the faulty expression's [context](`ErrCtx`)
/// and the [stack trace](`StckErrorCtx::get_stack`)
#[derive(Debug)]
pub struct StckErrorCtx {
    pub ctx: ErrCtx,
    pub kind: Box<StckError>,
    pub stack: Vec<ErrCtx>,
}

impl StckErrorCtx {
    pub(crate) fn into_case(self) -> StckErrorCase {
        self.into()
    }
    #[must_use]
    pub fn append_stack(mut self, ctx: ErrCtx) -> Self {
        self.stack.push(ctx);
        self
    }
    #[must_use]
    pub fn get_stack(&self) -> &[ErrCtx] {
        &self.stack
    }
}

impl Display for StckErrorCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} doing {}", "Error".red(), self.ctx)?;
        writeln!(f, "{}", self.kind)?;
        writeln!(f, "{} {}", "!".on_bright_red(), self.ctx)?;
        for ctx in &self.stack {
            writeln!(f, "{} {}", ">".bright_blue(), ctx)?;
        }
        Ok(())
    }
}

impl Display for LineRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.during <= 1 {
            write!(f, "{}", self.before)
        } else {
            write!(f, "{}:+{}", self.before, self.during)
        }
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
