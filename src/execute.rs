use crate::*;

macro_rules! sget {
    (num) => {
        (Value::get_num, "Number")
    };
    (str) => {
        (Value::get_str, "String")
    };
    (bool) => {
        (Value::get_bool, "Boolean")
    };
    (arr) => {
        (Value::get_arr, "Array")
    };
    (map) => {
        (Value::get_map, "Map")
    };
    (result) => {
        (Value::get_result, "Result")
    }
}

macro_rules! stack_pop {
    (($stack:expr) -> $type:ident as $this_arg:literal for $fn_name:expr) => {
        $stack
            .pop_this(sget!($type).0)
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: stringify!( [ $this_arg: $ty ] ),
                this_arg: $this_arg,
            })
            .map(|got_v|{
                got_v.map_err(|got|{
                    SttError::WrongTypeForBuiltin {
                        for_fn: $fn_name.to_owned(),
                        args: stringify!( [ $this_arg: $ty ] ),
                        this_arg: $this_arg,
                        got,
                        expected: sget!($type).1
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
                        expected: sget!($type).1
                    }
                })
            }).transpose()
    };
    (($stack:expr) -> * as $this_arg:literal for $fn_name:expr) => {
        $stack
            .pop()
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: stringify!( [ $this_arg: $ty ] ),
                this_arg: $this_arg,
            })
    };
    (($stack:expr) -> &$type:ident as $this_arg:literal for $fn_name:expr) => {
        $stack
            .peek()
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: stringify!( [ $this_arg: $ty ] ),
                this_arg: $this_arg,
            })
            .map(|got_v|{
                got_v.map_err(|got|{
                    SttError::WrongTypeForBuiltin {
                        for_fn: $fn_name.to_owned(),
                        args: stringify!( [ $this_arg: $ty ] ),
                        this_arg: $this_arg,
                        got,
                        expected: sget!($type).1
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
        match args_ins {
            FnArgsIns::AllStack(xs) => {
                stack = Stack::new_with(xs);
                args = None;
            }
            FnArgsIns::Args(ars) => {
                args = Some(ars);
                stack = Stack::new();
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
        for _expr in code.as_slice() {
            todo!();
            //self.debug(expr);
        }
    }

    //TODO fn to change in debug mode
    pub fn execute_code(&mut self, code: &Code) -> Result<()> {
        for expr in code.as_slice() {
            self.execute(expr)?;
        }
        Ok(())
    }

    pub fn execute_check(&mut self, code: &Code) -> Result<bool> {
        let old_stack_size = self.stack.len();
        for expr in code.as_slice() {
            self.execute(expr)?;
        }
        let new_stack_size = self.stack.len();
        let new_should_stack_size = old_stack_size + 1;
        let correct_size = new_should_stack_size == new_stack_size;
        let check = self.stack.pop();
        match check {
            Some(Value::Bool(b)) if correct_size => Ok(b),
            None => Err(SttError::WrongStackSizeDiffOnCheck {
                old_stack_size,
                new_stack_size,
                new_should_stack_size,
            }),
            Some(_) if correct_size => Err(SttError::WrongStackSizeDiffOnCheck {
                old_stack_size,
                new_stack_size,
                new_should_stack_size,
            }),
            Some(got) => Err(SttError::WrongTypeOnCheck { got }),
        }
    }

    pub fn execute(&mut self, expr: &Expr) -> Result<()> {
        match expr {
            Expr::FnCall(name) => return self.execute_fn(name),
            Expr::Keyword(kw) => return self.execute_kw(kw),
            Expr::Immediate(v) => self.stack.push(v.clone()),
        };
        Ok(())
    }

    fn execute_kw(&mut self, kw: &KeywordKind) -> Result<()> {
        match kw {
            KeywordKind::Ifs { branches } => {
                for branch in branches {
                    if self.execute_check(&branch.check)? {
                        self.execute_code(&branch.code)?;
                        break;
                    }
                }
            }
            KeywordKind::While { check, code } => {
                while self.execute_check(check)? {
                    self.execute_code(code)?;
                }
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
            }
        };
        Ok(())
    }

    fn execute_fn(&mut self, name: &FnName) -> Result<()> {
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
        } else if let Some(rets) = self.try_execute_user_fn(name) {
            // try_execute_user_fn should handle stack pop
            // and have the lowest precedence, since the traverse the scopes
            self.stack.pushn(rets?);
        } else {
            return Err(SttError::MissingIdent(name.0.clone()));
        }
        Ok(())
    }

    fn try_execute_user_fn(&mut self, name: &FnName) -> Option<Result<Vec<Value>>> {
        let user_fn = self.fns.get(name)?;

        let vars = match user_fn.scope {
            FnScope::Isolated => HashMap::new(),
            _ => self.vars.clone(),
        };

        let args = match &user_fn.args {
            FnArgs::Args(args) => {
                let args_stack = match self.stack.popn(args.len()) {
                    Ok(xs) => xs,
                    Err(rest) => panic!(
                        "`Not enough arguments to execute {}, got {:?} needs {:?}`",
                        name.as_str(),
                        rest,
                        user_fn.args,
                    ),
                };
                let arg_map = args
                    .clone()
                    .into_iter()
                    .map(FnName)
                    .zip(args_stack.into_iter().map(FnArg))
                    .collect();
                FnArgsIns::Args(arg_map)
            }
            FnArgs::AllStack => FnArgsIns::AllStack(self.stack.take()),
        };
        let mut fn_ctx = Context::frame(self.fns.clone(), vars, args);
        if let Err(e) = fn_ctx.execute_code(&user_fn.code) {
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
            "sh!" => {
                let shell_cmd = stack_pop!(
                    (self.stack) -> str as "command" for fn_name
                )??;
                let out = builtin::sh(&shell_cmd);
                if let Err(e) = out {
                    panic!("`sh!` {e}")
                }
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
                    (Num(l), Num(r)) => l == r,
                    (Str(l), Str(r)) => l == r,
                    (Bool(l), Bool(r)) => l == r,
                    (Array(_), _) | (_, Array(_)) => panic!("Can't compare array"),
                    (Map(_), _) | (_, Map(_)) => panic!("Can't compare map"),
                    (_, _) => false,
                };
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
                    (l, r) => panic!("Can't compare {l:?} with {r:?}"),
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
                    (Bool(l), Bool(r)) => l > r,
                    (l, r) => panic!("Can't compare {l:?} with {r:?}"),
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
            "ok" => {
                let v = self.stack.pop().expect("`ok` needs [value]");
                self.stack.push_this(Ok(v));
            }
            "err" => {
                let v = self.stack.pop().expect("`err` needs [value]");
                self.stack.push_this(Err(v));
            }
            "ok$is" => {
                let is_ok = match self.stack.peek() {
                    Some(Value::Result(x)) => x.is_ok(),
                    Some(r) => panic!("Called ok$is on non-result value {r:?}"),
                    _ => panic!("Called ok$is with nothing on stack"),
                };
                self.stack.push_this(is_ok);
            }
            "ok!" => {
                let v = stack_pop!((self.stack) -> result as "result" for fn_name)??;
                let v = match v {
                    Ok(v) => v,
                    Err(e) => panic!("ok! got error: {e:?}"),
                };
                self.stack.push_this(v);
            }

            // seq string
            "%" => {
                let fmt = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`%` needs at least [string]")
                    .expect("`%` [string] must be a string");
                let out = builtin::fmt(&fmt, &mut self.stack);
                self.stack.push_this(out);
            }
            "str-peek$has-prefix" => {
                let prefix = stack_pop!(
                    (self.stack) -> str as "prefix" for fn_name
                )??;
                //TODO peek in stack_pop! as &<type>
                let s = self.stack.peek();
                let s = match s {
                    Some(Value::Str(x)) => x,
                    _ => panic!("`str-peek$has-prefix` needs [string, prefix]"),
                };
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

            // seq array
            "arr-peek$len" => {
                let arr = self.stack.peek().expect("`arr-peek$len` needs [arr]");
                let arr_len = match arr {
                    Value::Array(arr) => arr,
                    _ => panic!("`arr-peek$len`'s [arr] must be an array"),
                }
                .len();
                self.stack.push_this(arr_len as isize);
            }
            "arr$reverse" => {
                let arr = self.stack.pop().expect("`arr$reverse` needs [arr]");
                let mut arr = match arr {
                    Value::Array(arr) => arr,
                    _ => panic!("`arr$reverse`'s [arr] must be an array"),
                };
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
                let count = self
                    .stack
                    .pop_this(Value::get_num)
                    .expect("arr$pack-n` needs [count]")
                    .expect("arr$pack-n` [count] must be a number");
                let xs = self
                    .stack
                    .popn(count as usize)
                    .unwrap_or_else(|_| panic!("arr$pack-n failed to pop {count} items"));
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
                let joiner = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("arr$join` needs [array joiner]")
                    .expect("arr$join` [joiner] must be a string");
                let arr: OResult<Vec<String>, Value> = self
                    .stack
                    .pop_this(Value::get_arr)
                    .expect("arr$append` needs [value array]")
                    .expect("arr$append` [array] must be an array")
                    .into_iter()
                    .map(|i| i.get_str())
                    .collect();
                let joint = match arr {
                    Ok(xs) => xs.join(&joiner),
                    Err(v) => {
                        panic!("`arr$join`: join's array can only have strings, found: {v:?}")
                    }
                };
                self.stack.push_this(joint);
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
                let got = match self.stack.peek() {
                    Some(Value::Map(m))=>m.get(&key),
                    _ => panic!("")
                }.cloned();
                self.stack.push_this(got);
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
        eprintln!("Write {} bytes to {file}", cont.bytes().len());
        Ok(cont.bytes().len() as isize)
    }
    pub fn fmt(cont: &str, stack: &mut Stack) -> String {
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
                    let add_str = stack
                        .pop_this(Value::get_str)
                        .unwrap_or_else(|| {
                            panic!(
                                "`%` format string {cont:?} needs a value that'snot in the stack"
                            )
                        })
                        .unwrap_or_else(|_| panic!("`%` format string {cont:?} needed a string"));
                    out.push_str(&add_str);
                    State::Nothing
                }
                (State::OnFmt, 'd') => {
                    let add_num = stack
                        .pop_this(Value::get_num)
                        .unwrap_or_else(|| {
                            panic!(
                                "`%` format string {cont:?} needs a value that'snot in the stack"
                            )
                        })
                        .unwrap_or_else(|_| panic!("`%` format string {cont:?} needed a number"));
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
                    let add_bool = stack
                        .pop_this(Value::get_bool)
                        .unwrap_or_else(|| {
                            panic!(
                                "`%` format string {cont:?} needs a value that'snot in the stack"
                            )
                        })
                        .unwrap_or_else(|_| panic!("`%` format string {cont:?} needed a bool"));
                    out.push_str(&add_bool.to_string());
                    State::Nothing
                }
                (State::OnFmt, x) => {
                    panic!(
                        "`%` doesn't recognise the format directive {x}, only '%', 'd', 's' and 'b' are avaliable "
                    )
                }
            }
        }
        out
    }
}
