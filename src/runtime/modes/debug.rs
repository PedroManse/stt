use super::*;
use std::io::{self, Write};
use crossterm::{
    cursor, execute, queue, style::{self, style, Stylize}, terminal, ExecutableCommand
};

pub struct DebugContext {
    pub fns: HashMap<FnName, FnDef>,
    pub exec_ctx: exec::ExecContext,
    pub stdout: io::Stdout,
}

impl DebugContext {
    pub fn new() -> Result<Self> {
        let mut stdout = io::stdout();
        stdout.execute(terminal::Clear(terminal::ClearType::Purge))?;
        stdout.execute(cursor::MoveTo(0, 0))?;
        stdout.execute(style::PrintStyledContent("Hello".magenta()))?;
        Ok(Self {
            fns: HashMap::new(),
            exec_ctx: exec::ExecContext::new(),
            stdout,
        })
    }

    pub fn frame(
        fns: HashMap<FnName, FnDef>,
        vars: HashMap<String, Value>,
        args_ins: FnArgsIns,
    ) -> Self {
        let stdout = io::stdout();
        let stack;
        let args;
        match args_ins.cap {
            FnArgsInsCap::AllStack(xs) => {
                stack = Stack::new_with(xs);
                args = args_ins.parent;
            }
            FnArgsInsCap::Args(ars) => {
                stack = Stack::new();
                let mut joint_args = HashMap::new();
                if let Some(parent_args) = args_ins.parent {
                    joint_args.extend(parent_args);
                }
                joint_args.extend(ars);
                args = Some(joint_args);
            }
        };
        Self {
            fns,
            stdout,
            exec_ctx: exec::ExecContext::frame(vars, stack, args),
        }
    }

    pub fn execute_expr(&mut self, expr: &Expr, source: &Path) -> Result<ControlFlow> {
        match &expr.cont {
            ExprCont::FnCall(name) => self.execute_fn(name, source)?,
            ExprCont::Keyword(kw) => {
                return self.execute_kw(kw, source);
            }
            ExprCont::Immediate(v) => self.exec_ctx.stack.push(v.clone()),
            ExprCont::IncludedCode(Code { source, exprs }) => {
                self.execute_code(exprs, source)?;
            }
        };
        Ok(ControlFlow::Continue)
    }


    pub fn execute_code(&mut self, code: &[Expr], source: &Path) -> Result<ControlFlow> {
        for expr in code {
            match self.execute_expr(expr, source)? {
                ControlFlow::Continue => {}
                c => return Ok(c),
            }
        }
        Ok(ControlFlow::Continue)
    }

    pub fn execute_check(&mut self, code: &[Expr], source: &Path) -> Result<bool> {
        let old_stack_size = self.exec_ctx.stack.len();
        for expr in code {
            self.execute_expr(expr, source)?;
        }
        let new_stack_size = self.exec_ctx.stack.len();
        let new_should_stack_size = old_stack_size + 1;
        let correct_size = new_should_stack_size == new_stack_size;
        let check = self.exec_ctx.stack.pop();
        let check = match (check, correct_size) {
            (Some(c), true) => Ok(c),
            _ => Err(SttError::WrongStackSizeDiffOnCheck {
                old_stack_size,
                new_stack_size,
                new_should_stack_size,
            }),
        }?;
        match check {
            Value::Bool(b) if correct_size => Ok(b),
            got => Err(SttError::WrongTypeOnCheck { got }),
        }
    }

    fn execute_kw(&mut self, kw: &KeywordKind, source: &Path) -> Result<ControlFlow> {
        Ok(match kw {
            KeywordKind::Return => ControlFlow::Return,
            KeywordKind::Break => ControlFlow::Break,
            KeywordKind::Switch { cases, default } => {
                let cmp = self
                    .exec_ctx
                    .stack
                    .pop()
                    .ok_or(SttError::RTSwitchCaseWithNoValue)?;
                for case in cases {
                    if case.test == cmp {
                        return self.execute_code(&case.code, source);
                    }
                }
                match default {
                    Some(code) => self.execute_code(code, source)?,
                    None => ControlFlow::Continue,
                }
            }
            KeywordKind::Ifs { branches } => {
                for branch in branches {
                    if self.execute_check(&branch.check, source)? {
                        return self.execute_code(&branch.code, source);
                    }
                }
                ControlFlow::Continue
            }
            KeywordKind::While { check, code } => {
                while self.execute_check(check, source)? {
                    match self.execute_code(code, source)? {
                        ControlFlow::Break => break,
                        ControlFlow::Return => return Ok(ControlFlow::Return),
                        _ => {}
                    }
                }
                ControlFlow::Continue
            }
            KeywordKind::FnDef {
                name,
                scope,
                code,
                args,
            } => {
                self.fns.insert(
                    name.clone(),
                    FnDef::new(scope.clone(), code.clone(), args.clone()),
                );
                ControlFlow::Continue
            }
        })
    }

    fn execute_fn(&mut self, name: &FnName, source: &Path) -> Result<()> {
        // builtin fn should handle stack pop and push
        // and are always given precedence
        match self.try_execute_builtin(name.as_str()) {
            Ok(()) => return Ok(()),
            Err(SttError::NoSuchBuiltin) => {}
            Err(e) => return Err(e),
        };

        if let Some(arg) = self.try_get_arg(name) {
            // try_get_arg should not pop from the stack and has higher precedence than user-defined funcs.
            // this was done to avoid confusion if an outer-scoped function was used instead of an argument
            self.exec_ctx.stack.push(arg);
        } else if let Some(rets) = self.try_execute_user_fn(name, source) {
            // try_execute_user_fn should handle stack pop
            // and have the lowest precedence, since the traverse the scopes
            self.exec_ctx.stack.pushn(rets?);
        } else {
            return Err(SttError::MissingIdent(name.0.clone()));
        }
        Ok(())
    }

    fn try_execute_user_fn(&mut self, name: &FnName, source: &Path) -> Option<Result<Vec<Value>>> {
        let user_fn = self.fns.get(name)?;

        let vars = match user_fn.scope {
            FnScope::Isolated => HashMap::new(),
            _ => self.exec_ctx.vars.clone(),
        };

        let args = match &user_fn.args {
            FnArgs::Args(args) => {
                let args_stack = match self.exec_ctx.stack.popn(args.len()) {
                    Some(xs) => xs,
                    None => {
                        return Some(Err(SttError::RTUserFnMissingArgs {
                            name: name.as_str().to_string(),
                            got: self.exec_ctx.stack.0.clone(),
                            needs: user_fn.args.clone().into_vec(),
                        }));
                    }
                };
                let arg_map = args
                    .clone()
                    .into_iter()
                    .map(FnName)
                    .zip(args_stack.into_iter().map(FnArg))
                    .collect();
                FnArgsInsCap::Args(arg_map)
            }
            FnArgs::AllStack => FnArgsInsCap::AllStack(self.exec_ctx.stack.take()),
        };
        let args = FnArgsIns {
            cap: args,
            parent: self.exec_ctx.args.clone(),
        };
        let mut fn_ctx = DebugContext::frame(self.fns.clone(), vars, args);

        // handle (return) kw and RT errors inside functions
        if let Err(e) = fn_ctx.execute_code(&user_fn.code, source) {
            return Some(Err(e));
        };

        if let FnScope::Global = user_fn.scope {
            self.exec_ctx.vars.extend(fn_ctx.exec_ctx.vars);
        }
        Some(Ok(fn_ctx.exec_ctx.stack.into_vec()))
    }

    fn try_get_arg(&mut self, name: &FnName) -> Option<Value> {
        if let Some(args) = &self.exec_ctx.args {
            args.get(name).map(|arg| arg.0.clone())
        } else {
            None
        }
    }

    fn try_execute_builtin(&mut self, fn_name: &str) -> Result<()> {
        // restore cursor position
        self.exec_ctx.try_execute_builtin(fn_name)?;
        // save cursor position
        Ok(())
    }
}


