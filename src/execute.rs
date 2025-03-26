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
    //pub fn proc_execute_code(mut self, code: Code, parent: &Context) {
    //    for expr in code.0 {
    //        self.proc_execute(expr);
    //    }
    //}
    pub fn execute_code(mut self, code: Code) {
        for expr in code.0 {
            self.execute(expr);
        }
    }
    //fn frame(parent: ) -> Self {
    //    Self {
    //        vars: HashMap::new(),
    //        stack: Stack::new_with(stack_args),
    //        parent: Some(Box::new(self)),
    //    }
    //}
    //fn deframe(self) -> Option<Self> {
    //    let mut this = self.parent?;
    //    this.stack.merge(self.stack);
    //    Some(*this)
    //}

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
            return;
        }
        todo!();
        //if let Some(()) = self.try_execute_user_fn(name) {
        //    return;
        //}
    }

    fn try_execute_user_fn(&mut self, name: FnName) -> Option<()> {
        let user_fn = self.fns.get(&name)?.clone();
        let new_scope = Context::new();
        new_scope.execute_code(user_fn.code);
        Some(())
    }

    fn try_execute_builtin(&mut self, name: &str) -> Option<()> {
        match name {
            "print" => {
                let st = self.stack.pop().expect("`print` needs one argument");
                print!("{:?}", st);
            }
            _ => return None,
        };
        Some(())
    }
}
