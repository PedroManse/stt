use crate::*;

pub struct Context {
    pub vars: HashMap<String, Value>,
    fns: HashMap<FnName, FnDef>,
    pub stack: Stack,
    args: HashMap<FnName, FnArg>,
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

    pub fn execute_code(&mut self, code: Code) {
        for expr in code.0 {
            self.execute(expr);
        }
    }

    pub fn execute(&mut self, expr: Expr) {
        match expr {
            Expr::FnDef(global, args, name, code) => self.define_fn(global, args, name, code),
            Expr::FnCall(name) => self.execute_fn(name),
            Expr::Keyword(kw) => self.execute_kw(kw),
            Expr::Immediate(v) => self.stack.push(v),
        };
    }

    fn define_fn(&mut self, global: bool, args: FnArgs, name: FnName, code: Code) {
        self.fns.insert(name, FnDef::new(global, code, args));
    }

    fn execute_kw(&mut self, kw: KeywordKind) {
        todo!()
    }

    fn execute_fn(&mut self, name: FnName) {
        if let Some(()) = self.try_execute_builtin(name.as_str()) {
            // builtin fn should handle stack pop and push
        } else if let Some(arg) = self.try_get_arg(&name) {
            // try_get_arg should handle stack pop
            self.stack.push(arg);
        } else if let Ok(rets) = self.try_execute_user_fn(name) {
            // try_execute_user_fn should handle stack pop
            self.stack.pushn(rets);
        } else {
            panic!()
        }
    }

    fn try_execute_user_fn(&mut self, name: FnName) -> Result<Vec<Value>, ()> {
        let user_fn = self.fns.get(&name).ok_or(())?.clone();

        let vars = match user_fn.scope {
            FnScope::Local | FnScope::Global => self.vars.clone(),
            FnScope::Isolated => HashMap::new(),
        };

        let args_req = user_fn.args.into_vec();
        let args_stack = self.stack.popn(args_req.len()).ok().ok_or(())?;
        let args = args_req
            .into_iter()
            .zip(args_stack)
            .map(|(name, vl)| (FnName(name), FnArg(vl)))
            .collect();
        let mut fn_ctx = Context::frame(self.fns.clone(), vars, args);
        fn_ctx.execute_code(user_fn.code);

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
            _ => return None,
        };
        Some(())
    }
}
