use crate::*;

pub struct Context {
    pub vars: HashMap<String, Value>,
    pub fns: HashMap<FnName, FnDef>,
    pub stack: Stack,
    pub args: HashMap<FnName, FnArg>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            fns: HashMap::new(),
            vars: HashMap::new(),
            stack: Stack::new(),
            args: HashMap::new(),
        }
    }

    pub fn frame(
        fns: HashMap<FnName, FnDef>,
        vars: HashMap<String, Value>,
        args: HashMap<FnName, FnArg>,
    ) -> Self {
        Self {
            fns,
            vars,
            args,
            stack: Stack::new(),
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

        let args = {
            let args_stack = match self.stack.popn(user_fn.args.len()) {
                Ok(xs) => xs,
                Err(rest) => panic!(
                    "`Not enough arguments to execute {}, got {:?} needs {:?}`",
                    name.as_str(),
                    rest,
                    user_fn.args,
                ),
            };
            user_fn
                .args
                .clone()
                .into_vec()
                .into_iter()
                .map(FnName)
                .zip(args_stack.into_iter().map(FnArg))
                .collect()
        };
        let mut fn_ctx = Context::frame(self.fns.clone(), vars, args);
        fn_ctx.execute_code(&user_fn.code);

        if let FnScope::Global = user_fn.scope {
            self.vars.extend(fn_ctx.vars);
        }
        Ok(fn_ctx.stack.into_vec())
    }

    fn try_get_arg(&mut self, name: &FnName) -> Option<Value> {
        self.args.get(&name).map(|arg| arg.0.clone())
    }

    fn try_execute_builtin(&mut self, name: &str) -> Option<()> {
        match name {
            "print" => {
                let st = self.stack.pop().expect("`print` needs one argument");
                print!("{:?}", st);
            }
            "set" => {
                let name = self.stack.pop().expect("`set` needs [name, value]");
                let value = self.stack.pop().expect("`set` needs [name, value]");
                let name = match name {
                    Value::Str(name) => name,
                    _ => panic!("`set`'s [name] needs to be a string"),
                };
                self.vars.insert(name, value);
            }
            "get" => {
                let name = self.stack.pop().expect("`get` needs [name]");
                let name = match name {
                    Value::Str(name) => name,
                    _ => panic!("`get`'s [name] needs to be a string"),
                };
                match self.vars.get(&name) {
                    None => panic!("`get`: variable {name} doesn't exist"),
                    Some(v) => {
                        self.stack.push(v.clone());
                    }
                }
            }
            "true" => {
                self.stack.push(Value::Bool(true));
            }
            "false" => {
                self.stack.push(Value::Bool(false));
            }
            "-" => {
                let lhs = self.stack.pop().expect("`-` needs [lhs, rhg]");
                let rhs = self.stack.pop().expect("`-` needs [lhs, rhg]");
                let res = match (lhs, rhs) {
                    (Value::Num(l), Value::Num(r)) => l - r,
                    _ => panic!("`-`'s [lhs] and [rhs] need to be numbers"),
                };
                self.stack.push(Value::Num(res));
            }
            "arr-peek$len" => {
                //TODO self.stask.peek() to get Option<&Value>
            }
            "arr$reverse" => {
                let arr = self.stack.pop().expect("`arr$reverse` needs [arr]");
                let mut arr = match arr {
                    Value::Array(arr)=>arr,
                    _=>panic!("`arr$reverse`'s [arr] must be an array")
                };
                arr.reverse();
                self.stack.push(Value::Array(arr));
            }
            "sys-argv" => {
                self.stack.push(Value::Array(vec![
                    Value::Str("proj".to_owned()),
                    Value::Str("crateA".to_owned()),
                    Value::Str("-featA".to_owned()),
                    Value::Str("-featB".to_owned()),
                    Value::Str("crateB".to_owned()),
                    Value::Str("-featC".to_owned()),
                    Value::Str("crateC".to_owned()),
                    Value::Str("crateD".to_owned()),
                ]));
            }

            _ => return None,
        };
        Some(())
    }
}
