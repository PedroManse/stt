use super::*;
use colored::Colorize;
use std::fmt::{Display, Formatter};

pub struct DisplayArgs<'a>(pub &'a [super::FnArgDef]);

impl Display for DisplayArgs<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        for arg in self.0 {
            if let Some(tt) = arg.get_type() {
                write!(f, "{arg}<{}> ", tt.to_string().underline().blue())?;
            } else {
                write!(f, "{arg} ")?;
            }
        }
        write!(f, "]")
    }
}

impl Display for FnArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllStack => write!(f, "the stack"),
            Self::Args(args) => write!(f, "{}", display::DisplayArgs(args)),
        }
    }
}

impl Display for FnScope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FnScope::Local => Ok(()),
            FnScope::Global => f.write_str("*"),
            FnScope::Isolated => f.write_str("-"),
        }
    }
}

impl Display for TypedFnPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TypedFnPart::Typed(args) => {
                write!(f, " ")?;
                for arg in args {
                    write!(f, "{} ", arg.to_string().underline().blue())?;
                }
                Ok(())
            }
            TypedFnPart::Any => write!(f, "?"),
        }
    }
}

impl Display for TypeTester {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TypeTester::*;
        match self {
            Any => write!(f, "?"),
            Char => write!(f, "char"),
            Str => write!(f, "str"),
            Num => write!(f, "num"),
            Bool => write!(f, "bool"),
            ArrayAny => write!(f, "array"),
            MapAny => write!(f, "map"),
            ResultAny => write!(f, "result"),
            OptionAny => write!(f, "option"),
            ClosureAny => write!(f, "fn"),
            Array(t) => write!(f, "array<{t}>"),
            Map(v) => write!(f, "map<{v}>"),
            Option(t) => write!(f, "option<{t}>"),
            Result(tt) => write!(f, "result<{}><{}>", tt.0, tt.1),
            Closure(tin, tout) => write!(f, "fn<{tin}><{tout}>"),
            Generic(a) => write!(f, "{a}"),
        }
    }
}

impl Display for ExprCont {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate(Value::Closure(cl)) => {
                write!(f, "instantiate Closure at {cl:p}")
            }
            Self::Immediate(v) => {
                write!(f, "Push value {v:?}")
            }
            Self::FnCall(fn_name) => {
                write!(f, "Execute `{}`", fn_name.bright_yellow())
            }
            Self::IncludedCode(code) => {
                write!(
                    f,
                    "Included file {}",
                    code.source.display().to_string().green()
                )
            }
            Self::Keyword(k) => {
                write!(f, "Keyword: ")?;
                match k {
                    KeywordKind::DefinedGeneric(g) => write!(f, "Define generic {g:?}"),
                    KeywordKind::Break => write!(f, "Break"),
                    KeywordKind::Return => write!(f, "Return"),
                    KeywordKind::IntoClosure { fn_name } => write!(f, "`{fn_name}` into Closure"),
                    KeywordKind::Ifs { .. } => write!(f, "If"),
                    KeywordKind::BubbleError => write!(f, "Bubble error"),
                    KeywordKind::While { .. } => write!(f, "While"),
                    KeywordKind::FnDef {
                        name,
                        scope,
                        args,
                        out_args: Some(out_args),
                        ..
                    } => write!(
                        f,
                        "Define fn{scope} `{}` as {args} → {}",
                        name.bright_yellow(),
                        DisplayArgs(out_args)
                    ),
                    KeywordKind::FnDef {
                        name,
                        scope,
                        args,
                        out_args: None,
                        ..
                    } => write!(
                        f,
                        "Define fn{scope} `{}` consuming {args}",
                        name.bright_yellow()
                    ),
                    KeywordKind::Switch { .. } => write!(f, "Switch"),
                }
            }
        }
    }
}

impl Display for FnArgDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for ErrCtx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} in {}:{}",
            self.expr.cont,
            self.source.display().to_string().green(),
            self.lines.to_string().bright_magenta().underline(),
        )
    }
}

impl Display for RuntimeErrorCtx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.end - self.start <= 1 {
            write!(f, "{}", self.start)
        } else {
            write!(f, "{}:+{}", self.start, self.delta())
        }
    }
}

impl Display for ErrorSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let source = self.source.display().to_string();
        let range = self.range.to_string();
        let title_len = source.len() + range.len() + 11;
        writeln!(
            f,
            "===[ {}:{} ]===",
            source.green(),
            range.bright_magenta().underline()
        )?;
        writeln!(f, "{}", self.lines)?;
        writeln!(f, "{}", "-".repeat(title_len).dimmed())?;
        Ok(())
    }
}

impl Display for parse::State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Nothing => "Making nothing",

            Self::MakeIfs(..) => "Making ifs, awaiting conditional check or end",
            Self::MakeIfsCode { .. } => "Making ifs, awaiting code block to execute",

            Self::MakeFnArgs(..) => "Making function, awaiting args",
            Self::MakeFnNameOrOutArgs(..) => "Making function, awaiting name or output args",
            Self::MakeFnName(..) => "Making function, awaiting name",
            Self::MakeFnBlock(..) => "MakeWhile function, awaiting code block to execute",

            Self::MakeSwitch(..) => "Making switch case, awaiting value to match",
            Self::MakeSwitchCode(..) => "Making switch case, awaiting code block to execute",

            Self::MakeWhile => "Making while loop, awaiting check code block",
            Self::MakeWhileCode(..) => "MakeWhile while lop, awaiting code block to execute",

            Self::MakeClosureBlockOrOutArgs(..) => {
                "Making closure, awaiting code block to execute or output args"
            }
            Self::MakeClosureBlock(..) => "Making closure, awaiting code block to execute",
        };
        write!(f, "{s}")
    }
}
