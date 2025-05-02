use crate::*;

macro_rules! sget {
    (num) => {
        (Value::get_num, Value::get_ref_num, "Number")
    };
    (str) => {
        (Value::get_str, Value::get_ref_str, "String")
    };
    (bool) => {
        (Value::get_bool, Value::get_ref_bool, "Boolean")
    };
    (arr) => {
        (Value::get_arr, Value::get_ref_arr, "Array")
    };
    (map) => {
        (Value::get_map, Value::get_ref_map, "Map")
    };
    (result) => {
        (Value::get_result, Value::get_ref_result, "Result")
    };
    (option) => {
        (Value::get_option, Value::get_ref_option, "Option")
    };
}

macro_rules! stack_pop {
    (($stack:expr) -> $type:ident as $this_arg:literal for $fn_name:expr) => {
        $stack
            .pop_this(sget!($type).0)
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: format!( "[{}: {}]", $this_arg, sget!($type).2 ),
                this_arg: $this_arg,
            })
            .map(|got_v|{
                got_v.map_err(|got|{
                    SttError::WrongTypeForBuiltin {
                        for_fn: $fn_name.to_owned(),
                        args: stringify!( [ $this_arg: $type ] ),
                        this_arg: $this_arg,
                        got,
                        expected: sget!($type).2
                    }
                })
            })
    };
    (($stack:expr) -> $type:ident? as $this_arg:literal for $fn_name:expr) => {
        $stack
            .pop_this(sget!($type).0)
            .map(|got_v|{
                got_v.map_err(|got|{
                    SttError::WrongTypeForBuiltin {
                        for_fn: $fn_name.to_owned(),
                        args: stringify!( [ $this_arg: $ty ] ),
                        this_arg: $this_arg,
                        got,
                        expected: sget!($type).2
                    }
                })
            }).transpose()
    };
    (($stack:expr) -> * as $this_arg:literal for $fn_name:expr) => {
        $stack
            .pop()
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: format!( "[{}]", $this_arg ),
                this_arg: $this_arg,
            })
    };
    (($stack:expr) -> &$type:ident as $this_arg:literal for $fn_name:expr) => {
        $stack
            .peek_this(sget!($type).1)
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: format!( "[{}: {}]", $this_arg, sget!($type).2 ),
                this_arg: $this_arg,
            })
            .map(|got_v|{
                got_v.map_err(|got|{
                    SttError::WrongTypeForBuiltin {
                        for_fn: $fn_name.to_owned(),
                        args: stringify!( [ $this_arg: $ty ] ),
                        this_arg: $this_arg,
                        got: got.clone(),
                        expected: sget!($type).2
                    }
                })
            })
    };
}

//use std::marker::PhantomData;
//pub struct DebugMode;
//pub struct NormalMode;
//trait ExecMode {}
//impl ExecMode for DebugMode {}
//impl ExecMode for NormalMode {}

//pub struct Context<Mode: ExecMode> {
#[derive(Default)]
pub struct Context {
    pub vars: HashMap<String, Value>,
    pub fns: HashMap<FnName, FnDef>,
    pub stack: Stack,
    pub args: Option<HashMap<FnName, FnArg>>,
    //pub _mode: PhantomData<Mode>,
}

//impl<M: ExecMode> Context<M> {
impl Context {
    pub fn new() -> Self {
        Self {
            fns: HashMap::new(),
            vars: HashMap::new(),
            stack: Stack::new(),
            args: None,
            //_mode: PhantomData,
        }
    }

    pub fn frame(
        fns: HashMap<FnName, FnDef>,
        vars: HashMap<String, Value>,
        args_ins: FnArgsIns,
    ) -> Self {
        let stack;
        let args;
        match args_ins.cap {
            FnArgsInsCap::AllStack(xs) => {
                stack = Stack::new_with(xs);
                args = args_ins.parent;
            }
            FnArgsInsCap::Args(mut ars) => {
                stack = Stack::new();
                if let Some(parent_args) = args_ins.parent {
                    ars.extend(parent_args);
                }
                args = Some(ars);
            }
        };
        Self {
            fns,
            vars,
            args,
            stack,
        }
    }

    pub fn debug_code(&mut self, code: &Code) {
        for _expr in &code.exprs {
            todo!();
            //self.debug(expr);
        }
    }

    //TODO fn to change in debug mode
    pub fn execute_code(&mut self, code: &[Expr], source: &Path) -> Result<ControlFlow> {
        println!("[{source:?}] -> {code:?}");
        for expr in code {
            match self.execute_expr(expr, source)? {
                ControlFlow::Continue => {}
                c => return Ok(c),
            }
        }
        Ok(ControlFlow::Continue)
    }

    pub fn execute_check(&mut self, code: &[Expr], source: &Path) -> Result<bool> {
        let old_stack_size = self.stack.len();
        for expr in code {
            self.execute_expr(expr, source)?;
        }
        let new_stack_size = self.stack.len();
        let new_should_stack_size = old_stack_size + 1;
        let correct_size = new_should_stack_size == new_stack_size;
        let check = self.stack.pop();
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

    pub fn execute_expr(&mut self, expr: &Expr, source: &Path) -> Result<ControlFlow> {
        match &expr.cont {
            ExprCont::FnCall(name) => self.execute_fn(name, source)?,
            ExprCont::Keyword(kw) => {
                return self.execute_kw(kw, source);
            }
            ExprCont::Immediate(v) => self.stack.push(v.clone()),
            ExprCont::IncludedCode(Code { source, exprs }) => {
                self.execute_code(exprs, source)?;
            }
        };
        Ok(ControlFlow::Continue)
    }

    fn execute_kw(&mut self, kw: &KeywordKind, source: &Path) -> Result<ControlFlow> {
        Ok(match kw {
            KeywordKind::Return => ControlFlow::Return,
            KeywordKind::Break => ControlFlow::Break,
            KeywordKind::Switch { cases, default } => {
                let cmp = self.stack.pop().ok_or(SttError::RTSwitchCaseWithNoValue)?;
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
                // TODO pass on args to make closures
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
            self.stack.push(arg);
        } else if let Some(rets) = self.try_execute_user_fn(name, source) {
            // try_execute_user_fn should handle stack pop
            // and have the lowest precedence, since the traverse the scopes
            self.stack.pushn(rets?);
        } else {
            return Err(SttError::MissingIdent(name.0.clone()));
        }
        Ok(())
    }

    fn try_execute_user_fn(&mut self, name: &FnName, source: &Path) -> Option<Result<Vec<Value>>> {
        let user_fn = self.fns.get(name)?;

        let vars = match user_fn.scope {
            FnScope::Isolated => HashMap::new(),
            _ => self.vars.clone(),
        };

        let args = match &user_fn.args {
            FnArgs::Args(args) => {
                let args_stack = match self.stack.popn(args.len()) {
                    Some(xs) => xs,
                    None => {
                        return Some(Err(SttError::RTUserFnMissingArgs {
                            name: name.as_str().to_string(),
                            got: self.stack.0.clone(),
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
            FnArgs::AllStack => FnArgsInsCap::AllStack(self.stack.take()),
        };
        let args = FnArgsIns {
            cap: args,
            parent: self.args.clone(),
        };
        let mut fn_ctx = Context::frame(self.fns.clone(), vars, args);

        // handle (return) kw and RT errors inside functions
        if let Err(e) = fn_ctx.execute_code(&user_fn.code, source) {
            return Some(Err(e));
        };

        if let FnScope::Global = user_fn.scope {
            self.vars.extend(fn_ctx.vars);
        }
        Some(Ok(fn_ctx.stack.into_vec()))
    }

    fn try_get_arg(&mut self, name: &FnName) -> Option<Value> {
        if let Some(args) = &self.args {
            args.get(name).map(|arg| arg.0.clone())
        } else {
            None
        }
    }

    fn try_execute_builtin(&mut self, fn_name: &str) -> Result<()> {
        match fn_name {
            // seq system
            "print" => {
                let cont = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`print` needs [string]")
                    .expect("`print`'s [string] needs to be a string");
                print!("{}", cont);
            }
            "sys$exit" => {
                let code = stack_pop!(
                    (self.stack) -> num as "exit_code" for fn_name
                )??;
                std::process::exit(code as i32);
            }
            "sys$argv" => {
                let args: Vec<_> = std::env::args().map(Value::Str).collect();
                self.stack.push_this(args);
            }
            "sh" => {
                let shell_cmd = stack_pop!(
                    (self.stack) -> str as "command" for fn_name
                )??;
                let out = builtin::sh(&shell_cmd).map(Value::Num).map_err(Value::Str);
                self.stack.push_this(out);
            }
            "write-to" => {
                let file = stack_pop!(
                    (self.stack) -> str as "file" for fn_name
                )??;
                let cont = stack_pop!(
                    (self.stack) -> str as "content" for fn_name
                )??;
                let out = builtin::write_to(&cont, &file)
                    .map(Value::Num)
                    .map_err(Value::Str);
                self.stack.push_this(out);
            }

            // seq math seq logic
            "-" => {
                let rhs = stack_pop!(
                    (self.stack) -> num as "rhs" for fn_name
                )??;
                let lhs = stack_pop!(
                    (self.stack) -> num as "lhs" for fn_name
                )??;
                self.stack.push_this(lhs - rhs);
            }
            "*" => {
                let rhs = stack_pop!(
                    (self.stack) -> num as "rhs" for fn_name
                )??;
                let lhs = stack_pop!(
                    (self.stack) -> num as "lhs" for fn_name
                )??;
                self.stack.push_this(lhs * rhs);
            }
            "≃" => {
                use Value::*;
                let rhs = stack_pop!((self.stack) -> * as "rhs" for fn_name)?;
                let lhs = stack_pop!((self.stack) -> * as "lhs" for fn_name)?;
                let eq = match (lhs, rhs) {
                    (Num(l), Num(r)) => Ok(l == r),
                    (Str(l), Str(r)) => Ok(l == r),
                    (Bool(l), Bool(r)) => Ok(l == r),
                    (r @ Array(_), l) | (l, r @ Array(_)) => {
                        Err(SttError::RTCompareError { this: l, that: r })
                    }
                    (m @ Map(_), l) | (l, m @ Map(_)) => {
                        Err(SttError::RTCompareError { this: l, that: m })
                    }
                    (_, _) => Ok(false),
                }?;
                self.stack.push_this(eq);
            }
            "=" => {
                use Value::*;
                let rhs = stack_pop!((self.stack) -> * as "rhs" for fn_name)?;
                let lhs = stack_pop!((self.stack) -> * as "lhs" for fn_name)?;
                let eq = match (lhs, rhs) {
                    (Num(l), Num(r)) => l == r,
                    (Str(l), Str(r)) => l == r,
                    (Bool(l), Bool(r)) => l == r,
                    (l, r) => {
                        return Err(SttError::RTCompareError { this: l, that: r });
                    }
                };
                self.stack.push_this(eq);
            }
            ">" => {
                use Value::*;
                let rhs = stack_pop!((self.stack) -> * as "rhs" for fn_name)?;
                let lhs = stack_pop!((self.stack) -> * as "lhs" for fn_name)?;
                let eq = match (lhs, rhs) {
                    (Num(l), Num(r)) => l > r,
                    (Str(l), Str(r)) => l > r,
                    (Bool(l), Bool(r)) => l & !r,
                    (l, r) => {
                        println!("->> {r:?} {l:?}");
                        return Err(SttError::RTCompareError { this: l, that: r });
                    }
                };
                self.stack.push_this(eq);
            }

            // seq variables
            "stack$len" => {
                self.stack.push_this(self.stack.len() as isize);
            }
            "set" => {
                let name = stack_pop!(
                    (self.stack) -> str as "name" for fn_name
                )??;
                let value = stack_pop!(
                    (self.stack) -> * as "value" for fn_name
                )?;
                self.vars.insert(name, value);
            }
            "get" => {
                let name = stack_pop!(
                    (self.stack) -> str as "name" for fn_name
                )??;
                match self.vars.get(&name) {
                    None => {
                        return Err(SttError::NoSuchVariable(name));
                    }
                    Some(v) => {
                        self.stack.push(v.clone());
                    }
                };
            }

            // seq error handeling
            "!" => {
                let may = stack_pop!((self.stack) -> * as "Monad" for fn_name)?;
                match may {
                    Value::Result(r) => {
                        match *r {
                            Err(error) => {
                                return Err(SttError::RTUnwrapResultBuiltinFailed { error });
                            }
                            Ok(o) => self.stack.push_this(o),
                        };
                    }
                    Value::Option(o) => match o {
                        None => return Err(SttError::RTUnwrapOptionBuiltinFailed),
                        Some(s) => self.stack.push_this(*s),
                    },
                    e => {
                        return Err(SttError::WrongTypeForBuiltin {
                            for_fn: fn_name.to_string(),
                            args: "[Monad]",
                            this_arg: "Monad",
                            got: e,
                            expected: "Result or Option",
                        });
                    }
                }
            }
            "ok" => {
                let v = self.stack.pop().expect("`ok` needs [value]");
                self.stack.push_this(Ok(v));
            }
            "err" => {
                let v = self.stack.pop().expect("`err` needs [value]");
                self.stack.push_this(Err(v));
            }
            "none" => {
                self.stack.push_this(None);
            }
            "some" => {
                let v = stack_pop!((self.stack) -> * as "v" for fn_name)?;
                self.stack.push_this(Some(v));
            }
            "&result$is-ok" => {
                let is_ok = stack_pop!((self.stack) -> &result as "result" for fn_name)??.is_ok();
                self.stack.push_this(is_ok);
            }
            "&option$is-some" => {
                let is_some =
                    stack_pop!((self.stack) -> &option as "option" for fn_name)??.is_some();
                self.stack.push_this(is_some);
            }

            // seq string
            "%" => {
                let fmt = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`%` needs at least [string]")
                    .expect("`%` [string] must be a string");
                let out = builtin::fmt(&fmt, &mut self.stack);
                self.stack.push_this(out?);
            }
            "&str$has-prefix" => {
                let prefix = stack_pop!(
                    (self.stack) -> str as "prefix" for fn_name
                )??;
                let s = stack_pop!((self.stack) -> &str as "string" for fn_name)??;
                let has = s.starts_with(&prefix);
                self.stack.push_this(has);
            }
            "str$trim" => {
                let v = stack_pop!(
                    (self.stack) -> str as "string" for fn_name
                )??;
                self.stack.push_this(v.trim().to_owned());
            }
            "str$remove-prefix" => {
                let prefix = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`str$remove-prefix` needs [string prefix]")
                    .expect("`str$remove-prefix` needs [prefix] to be a string");
                let st = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`str$remove-prefix` needs [string prefix]")
                    .expect("`str$remove-prefix` needs [string] to be a string");
                let out = st.strip_prefix(&prefix).map(String::from).unwrap_or(st);
                self.stack.push_this(out);
            }
            "str$into-arr" => {
                let string = stack_pop!((self.stack) -> str as "string" for fn_name)??;
                let chars: Vec<_> = string.chars().map(String::from).map(Value::from).collect();
                self.stack.push_this(chars);
            }

            // seq array
            "&arr$len" => {
                let arr_len = stack_pop!((self.stack) -> &arr as "array" for fn_name)??.len();
                self.stack.push_this(arr_len as isize);
            }
            "arr$reverse" => {
                let mut arr = stack_pop!((self.stack) -> arr as "arr" for fn_name)??;
                arr.reverse();
                self.stack.push_this(arr);
            }
            "arr$unpack" => {
                let arr = self
                    .stack
                    .pop_this(Value::get_arr)
                    .expect("arr$unpack` needs [arr]")
                    .expect("arr$unpack` [arr] must be an array");
                let len = arr.len();
                self.stack.pushn(arr);
                self.stack.push_this(len as isize);
            }
            "arr$pack-n" => {
                let count = stack_pop!((self.stack) -> num as "count" for fn_name)??;
                let xs = self.stack.popn(count as usize).ok_or_else(|| {
                    let got = self.stack.len() as isize;
                    let missing = count - got;
                    SttError::MissingValuesForBuiltin {
                        for_fn: fn_name.to_string(),
                        args: "[n, [n]]",
                        missing,
                    }
                })?;
                self.stack.push_this(xs);
            }
            "arr$new" => {
                self.stack.push_this(Vec::new());
            }
            "arr$append" => {
                let mut arr = self
                    .stack
                    .pop_this(Value::get_arr)
                    .expect("arr$append` needs [value array]")
                    .expect("arr$append` [array] must be an array");
                let any = self.stack.pop().expect("`arr$append` needs [value array]");
                arr.push(any);
                self.stack.push_this(arr);
            }
            "arr$join" => {
                let joiner = stack_pop!((self.stack) -> str as "joiner" for fn_name)??;
                let arr = stack_pop!((self.stack) -> arr as "array" for fn_name)??;
                let arr = arr
                    .into_iter()
                    .map(|i| i.get_str())
                    .collect::<OResult<Vec<_>, _>>()
                    .map_err(|got| SttError::WrongTypeForBuiltin {
                        for_fn: fn_name.to_string(),
                        args: "[array joiner]",
                        this_arg: "array",
                        expected: "String array",
                        got,
                    })?;
                self.stack.push_this(arr.join(&joiner));
            }

            // seq map
            "map$new" => {
                self.stack.push_this(HashMap::new());
            }
            "map$insert-kv" => {
                let value = stack_pop!(
                    (self.stack) -> * as "value" for fn_name
                )?;
                let key = stack_pop!(
                    (self.stack) -> str as "key" for fn_name
                )??;
                let mut map = stack_pop!(
                    (self.stack) -> map as "map" for fn_name
                )??;
                map.insert(key, value);
                self.stack.push_this(map);
            }
            "map$get" => {
                let key = stack_pop!(
                    (self.stack) -> str as "key" for fn_name
                )??;
                let got = stack_pop!((self.stack) -> &map as "map" for fn_name)??
                    .get(&key)
                    .cloned();
                self.stack.push_this(got);
            }

            // seq type
            "type$is-str" => {
                let is_type = stack_pop!((self.stack) -> str as "value" for fn_name)?.is_ok();
                self.stack.push_this(is_type);
            }
            "type$is-num" => {
                let is_type = stack_pop!((self.stack) -> num as "value" for fn_name)?.is_ok();
                self.stack.push_this(is_type);
            }
            "type$is-bool" => {
                let is_type = stack_pop!((self.stack) -> bool as "value" for fn_name)?.is_ok();
                self.stack.push_this(is_type);
            }
            "type$is-array" => {
                let is_type = stack_pop!((self.stack) -> arr as "value" for fn_name)?.is_ok();
                self.stack.push_this(is_type);
            }
            "type$is-map" => {
                let is_type = stack_pop!((self.stack) -> map as "value" for fn_name)?.is_ok();
                self.stack.push_this(is_type);
            }
            "type$is-result" => {
                let is_type = stack_pop!((self.stack) -> result as "value" for fn_name)?.is_ok();
                self.stack.push_this(is_type);
            }
            "type$is-option" => {
                let is_type = stack_pop!((self.stack) -> option as "value" for fn_name)?.is_ok();
                self.stack.push_this(is_type);
            }

            // seq debug
            "debug$stack" => eprintln!("{:?}", self.stack),
            "debug$vars" => eprintln!("{:?}", self.vars),
            "debug$args" => eprintln!("{:?}", self.args),

            _ => {
                return Err(SttError::NoSuchBuiltin);
            }
        };
        Ok(())
    }
}

mod builtin {
    use super::*;
    pub fn sh(shell_cmd: &str) -> OResult<isize, String> {
        eprintln!("[CMD] {shell_cmd}");
        Ok(0)
        //std::proces::Command::new("bash")
        //    .arg("-c")
        //    .arg(shell_cmd)
        //    .status()
        //    .map(|s| s.code().unwrap_or(256) as isize)
        //    .map_err(|e| e.to_string())
    }
    pub fn write_to(cont: &str, file: &str) -> OResult<isize, String> {
        eprintln!("Write {} bytes to {file}", cont.len());
        Ok(cont.len() as isize)
    }
    pub fn fmt(cont: &str, stack: &mut Stack) -> Result<String> {
        let mut out = String::with_capacity(cont.len());
        enum State {
            Nothing,
            OnFmt,
        }
        let mut state = State::Nothing;
        for ch in cont.chars() {
            state = match (state, ch) {
                (State::Nothing, '%') => State::OnFmt,
                (State::Nothing, ch) => {
                    out.push(ch);
                    State::Nothing
                }
                (State::OnFmt, '%') => {
                    out.push('%');
                    State::Nothing
                }
                (State::OnFmt, 's') => {
                    let add_str = stack_pop!((stack) -> str as "%s" for "%")??;
                    out.push_str(&add_str);
                    State::Nothing
                }
                (State::OnFmt, 'd') => {
                    let add_num = stack_pop!((stack) -> num as "%d" for "%")??;
                    out.push_str(&add_num.to_string());
                    State::Nothing
                }
                (State::OnFmt, 'v') => {
                    let fmt = match stack.pop() {
                        Some(x) => format!("{x:?}"),
                        None => "<Nothing in stack>".to_string(),
                    };
                    out.push_str(&fmt);
                    State::Nothing
                }
                (State::OnFmt, 'b') => {
                    let add_bool = stack_pop!((stack) -> bool as "%b" for "%")??;
                    out.push_str(&add_bool.to_string());
                    State::Nothing
                }
                (State::OnFmt, x) => {
                    return Err(SttError::RTUnknownStringFormat(x));
                }
            }
        }
        Ok(out)
    }
}
