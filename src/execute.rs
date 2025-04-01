use crate::*;

pub struct Context {
    pub vars: HashMap<String, Value>,
    pub fns: HashMap<FnName, FnDef>,
    pub stack: Stack,
    pub args: Option<HashMap<FnName, FnArg>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            fns: HashMap::new(),
            vars: HashMap::new(),
            stack: Stack::new(),
            args: None,
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
            FnArgsIns::AllStack(xs)=>{
                stack = Stack::new_with(xs);
                args = None;
            }
            FnArgsIns::Args(ars) =>{
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

    pub fn execute_code(&mut self, code: &Code) {
        for expr in code.as_slice() {
            self.execute(expr);
        }
    }

    pub fn execute_check(&mut self, code: &Code) -> bool {
        let stack_size = self.stack.len();
        for expr in code.as_slice() {
            self.execute(expr);
        }
        let new_stack_size = self.stack.len();
        let check = self.stack.pop();
        match check {
            Some(Value::Bool(b)) if stack_size + 1 == new_stack_size => b,
            v => {
                if stack_size + 1 != new_stack_size {
                    eprintln!(
                        "stack size was {} now it's {} when it should be {}",
                        stack_size,
                        new_stack_size,
                        stack_size + 1
                    );
                } else {
                    eprintln!("check blocks must recieve one boolean, recieved {:?}", v);
                }
                panic!("Any control flow code must push one, and only one, boolean to the stack")
            }
        }
    }

    pub fn execute(&mut self, expr: &Expr) {
        match expr {
            Expr::FnCall(name) => self.execute_fn(name),
            Expr::Keyword(kw) => self.execute_kw(kw),
            Expr::Immediate(v) => self.stack.push(v.clone()),
        };
    }

    fn execute_kw(&mut self, kw: &KeywordKind) {
        match kw {
            KeywordKind::Ifs { branches } => {
                for branch in branches {
                    if self.execute_check(&branch.check) {
                        self.execute_code(&branch.code);
                        break;
                    }
                }
            }
            KeywordKind::While { check, code } => {
                while self.execute_check(&check) {
                    self.execute_code(code);
                }
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
            }
        };
    }

    fn execute_fn(&mut self, name: &FnName) {
        if let Some(()) = self.try_execute_builtin(name.as_str()) {
            // builtin fn should handle stack pop and push
            // and are always given precedence
        } else if let Some(arg) = self.try_get_arg(&name) {
            // try_get_arg should handle stack pop
            // and have higher precedence than user-defined funcs (this was done to avoid confusion
            // if an upper-scoped function was used instead of an argument)
            self.stack.push(arg);
        } else if let Ok(rets) = self.try_execute_user_fn(name) {
            // try_execute_user_fn should handle stack pop
            // and have the lowest precedence, since the traverse the scopes
            self.stack.pushn(rets);
        } else {
            panic!(
                "No such builtin, argument or function called {}",
                name.as_str()
            );
        }
    }

    fn try_execute_user_fn(&mut self, name: &FnName) -> Result<Vec<Value>, ()> {
        let user_fn = self.fns.get(&name).ok_or(())?;

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
        fn_ctx.execute_code(&user_fn.code);

        if let FnScope::Global = user_fn.scope {
            self.vars.extend(fn_ctx.vars);
        }
        Ok(fn_ctx.stack.into_vec())
    }

    fn try_get_arg(&mut self, name: &FnName) -> Option<Value> {
        if let Some(args) = &self.args {
            args.get(&name).map(|arg| arg.0.clone())
        } else {
            None
        }
    }

    fn try_execute_builtin(&mut self, name: &str) -> Option<()> {
        match name {
            // mod system
            "print" => {
                let cont = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`print` needs [string]")
                    .expect("`print`'s [string] needs to be a string");
                print!("{}", cont);
            }
            "sys$argv" => {
                let args: Vec<_> = std::env::args().into_iter().map(Value::Str).collect();
                self.stack.push_this(args);
            }
            "sh" => {
                let shell_cmd = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`sh` needs [command]")
                    .expect("`sh` [command] must be a string");
                let out = builtin::sh(&shell_cmd).map(Value::Num).map_err(Value::Str);
                self.stack.push_this(out);
            }
            "sh!" => {
                let shell_cmd = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`sh` needs [command]")
                    .expect("`sh` [command] must be a string");
                let out = builtin::sh(&shell_cmd);
                if let Err(e) = out {
                    panic!("`sh!` {e}")
                }
            }
            "write-to" => {
                let file = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`trim` needs [content, file]")
                    .expect("`trim` [file] must be a string");
                let cont = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`trim` needs [content, file]")
                    .expect("`trim` [content] must be a string");
                let out = builtin::write_to(&cont, &file)
                    .map(Value::Num)
                    .map_err(Value::Str);
                self.stack.push_this(out);
            }

            // mod math mod logic
            "-" => {
                let rhs = self.stack.pop().expect("`-` needs [lhs, rhg]");
                let lhs = self.stack.pop().expect("`-` needs [lhs, rhg]");
                let res = match (lhs, rhs) {
                    (Value::Num(l), Value::Num(r)) => l - r,
                    _ => panic!("`-`'s [lhs] and [rhs] need to be numbers"),
                };
                self.stack.push_this(res);
            }
            "=" => {
                use Value::*;
                let rhs = self.stack.pop().expect("`=` needs [lhs, rhg]");
                let lhs = self.stack.pop().expect("`=` needs [lhs, rhg]");
                let eq = match (rhs, lhs) {
                    (Num(l), Num(r)) => l == r,
                    (Str(l), Str(r)) => l == r,
                    (Bool(l), Bool(r)) => l == r,
                    (l, r) => panic!("Can't compare {l:?} with {r:?}"),
                };
                self.stack.push_this(eq);
            }

            // mod variables
            //"stack$has" => {
            //    self.stack.push_this(self.stack.len() != 0);
            //} // can be just stack$len 0 = not
            "stack$len" => {
                self.stack.push_this(self.stack.len() as isize);
            }
            "set" => {
                let name = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`set` needs [name, value]")
                    .expect("`set`'s [name] needs to be a string");
                let value = self.stack.pop().expect("`set` needs [name, value]");
                self.vars.insert(name, value);
            }
            "get" => {
                let name = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`get` needs [name]")
                    .expect("`get`'s [name] needs to be a string");
                match self.vars.get(&name) {
                    None => panic!("`get`: variable {name} doesn't exist"),
                    Some(v) => {
                        self.stack.push(v.clone());
                    }
                }
            }
            // maybe add this to args
            "true" => {
                self.stack.push_this(true);
            }
            "false" => {
                self.stack.push_this(false);
            }
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
                    _ => panic!("Called ok$is with nothing on stack")
                };
                self.stack.push_this(is_ok);
            }
            "ok!" => {
                let v = self
                    .stack
                    .pop_this(Value::get_result)
                    .expect("`ok!` needs [result]")
                    .expect("`ok!` needs [result] to be a result");
                let v = match v {
                    Ok(v) => v,
                    Err(e) => panic!("ok! got error: {e:?}"),
                };
                self.stack.push_this(v);
            }

            // mod string
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
                let prefix = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`str-peek$has-prefix` needs [string, prefix]")
                    .expect("`str-peek$has-prefix` [prefix] must be a string");
                let s = self.stack.peek();
                let s = match s {
                    Some(Value::Str(x)) => x,
                    _ => panic!("`str-peek$has-prefix` needs [string, prefix]"),
                };
                let has = s.starts_with(&prefix);
                self.stack.push_this(has);
            }
            "str$trim" => {
                let v = self
                    .stack
                    .pop_this(Value::get_str)
                    .expect("`str$trim` needs [string]")
                    .expect("`str$trim` [string] must be a string");
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

            // mod array
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
                    .expect(&format!("arr$pack-n failed to pop {count} items"));
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
                let arr: Result<Vec<String>, Value> = self
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

            // mod debug
            "debug$stack" => println!("{:?}", self.stack),
            "debug$vars" => println!("{:?}", self.vars),
            "debug$args" => println!("{:?}", self.args),

            _ => return None,
        };
        Some(())
    }
}

mod builtin {
    use super::*;
    //use std::process::Command;
    pub fn sh(shell_cmd: &str) -> Result<isize, String> {
        println!("[CMD] {shell_cmd}");
        Ok(0)
        //Command::new("bash")
        //    .arg("-c")
        //    .arg(shell_cmd)
        //    .status()
        //    .map(|s| s.code().unwrap_or(256) as isize)
        //    .map_err(|e| e.to_string())
    }
    pub fn write_to(cont: &str, file: &str) -> Result<isize, String> {
        println!("Write {} bytes to {file}", cont.bytes().len());
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
                        .expect(&format!(
                            "`%` format string {cont:?} needs a value that's not in the stack"
                        ))
                        .expect(&format!("`%` format string {cont:?} needed a string"));
                    out.push_str(&add_str);
                    State::Nothing
                }
                (State::OnFmt, 'd') => {
                    let add_num = stack
                        .pop_this(Value::get_num)
                        .expect(&format!(
                            "`%` format string {cont:?} needs a value that's not in the stack"
                        ))
                        .expect(&format!("`%` format string {cont:?} needed a number"));
                    out.push_str(&add_num.to_string());
                    State::Nothing
                }
                (State::OnFmt, 'v') => {
                    let fmt = match stack.pop() {
                        Some(x)=>format!("{x:?}"),
                        None => format!("<Nothing in stack>")
                    };
                    out.push_str(&fmt);
                    State::Nothing
                }
                (State::OnFmt, 'b') => {
                    let add_bool = stack
                        .pop_this(Value::get_bool)
                        .expect(&format!(
                            "`%` format string {cont:?} needs a value that's not in the stack"
                        ))
                        .expect(&format!("`%` format string {cont:?} needed a bool"));
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
