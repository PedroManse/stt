use crate::*;

pub struct Context {
    code: Vec<Token>,
    ungotten: Option<Token>, // token to be re-parsed by different state
}

#[derive(Debug)]
enum State {
    Nothing,

    MakeIfs(Vec<CondBranch>),
    MakeIfsCode {
        branches: Vec<CondBranch>,
        check: Vec<Expr>,
    },

    MakeFnArgs(FnScope),
    MakeFnName(FnScope, FnArgs),
    MakeFnBlock(FnScope, FnArgs, FnName),

    MakeSwitch(Vec<SwitchCase>),
    MakeSwitchCode(Vec<SwitchCase>, Value),

    MakeWhile,
    MakeWhileCode(Vec<Expr>),

    MakeClosureBlock(Vec<String>),
}

impl Context {
    pub fn parse_block(&mut self) -> Result<Vec<Expr>> {
        use ExprCont as E;
        use State::*;
        use TokenCont::*;
        let mut state = Nothing;
        let mut out = vec![];
        let mut cum_span = 0..0;

        while let Some(token) = self.next() {
            let Token { cont, span } = token;
            cum_span.end = span.end;
            macro_rules! push_expr {
                ($expr:expr) => {
                    out.push(Expr {
                        cont: $expr,
                        span: cum_span,
                    });
                    cum_span = span.end..span.end;
                };
            }
            state = match (state, cont) {
                (Nothing, EndOfBlock) => Nothing,
                (Nothing, Ident(n)) => {
                    push_expr!(E::FnCall(FnName(n)));
                    Nothing
                }
                (Nothing, Str(x)) => {
                    push_expr!(E::Immediate(Value::Str(x)));
                    Nothing
                }
                (Nothing, Number(x)) => {
                    push_expr!(E::Immediate(Value::Num(x)));
                    Nothing
                }
                (Nothing, Keyword(RawKeyword::Break)) => {
                    push_expr!(E::Keyword(KeywordKind::Break));
                    Nothing
                }
                (Nothing, Keyword(RawKeyword::Return)) => {
                    push_expr!(E::Keyword(KeywordKind::Return));
                    Nothing
                }
                (Nothing, Keyword(RawKeyword::BubbleError)) => {
                    push_expr!(E::Keyword(KeywordKind::BubbleError));
                    Nothing
                }
                (Nothing, Keyword(RawKeyword::FnIntoClosure { fn_name })) => {
                    push_expr!(E::Keyword(KeywordKind::IntoClosure { fn_name }));
                    Nothing
                }

                (s, IncludedBlock(code)) => {
                    let mut inner_ctx = Context::new(code.tokens);
                    let parsed_code = inner_ctx.parse_block()?;
                    push_expr!(E::IncludedCode(Code {
                        source: code.source,
                        exprs: parsed_code
                    }));
                    s
                }

                (Nothing, FnArgs(args)) => MakeClosureBlock(args),
                (MakeClosureBlock(args), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let code = inner_ctx.parse_block()?;
                    let closure = Closure {
                        code,
                        request_args: ClosurePartialArgs::parse(args, span.clone())?,
                    };
                    push_expr!(E::Immediate(Value::Closure(Box::new(closure))));
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::Switch)) => MakeSwitch(vec![]),
                (MakeSwitch(cases), Str(v)) => MakeSwitchCode(cases, Value::Str(v)),
                (MakeSwitch(cases), Number(v)) => MakeSwitchCode(cases, Value::Num(v)),
                (MakeSwitchCode(mut cases, test), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let code = inner_ctx.parse_block()?;
                    cases.push(SwitchCase { test, code });
                    MakeSwitch(cases)
                }
                (MakeSwitch(cases), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let code = inner_ctx.parse_block()?;
                    push_expr!(E::Keyword(KeywordKind::Switch {
                        cases,
                        default: Some(code),
                    }));
                    Nothing
                }
                (MakeSwitch(cases), cont) => {
                    match cont {
                        EndOfBlock => {}
                        cont => self.unget(Token {
                            cont,
                            span: span.clone(),
                        }),
                    };
                    push_expr!(E::Keyword(KeywordKind::Switch {
                        cases,
                        default: None,
                    }));
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::Ifs)) => MakeIfs(vec![]),
                (MakeIfs(branches), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let check = inner_ctx.parse_block()?;
                    MakeIfsCode { branches, check }
                }
                (
                    MakeIfsCode {
                        mut branches,
                        check,
                    },
                    Block(code),
                ) => {
                    let mut inner_ctx = Context::new(code);
                    let code = inner_ctx.parse_block()?;
                    branches.push(CondBranch { check, code });
                    MakeIfs(branches)
                }
                (MakeIfs(branches), cont) => {
                    match cont {
                        EndOfBlock => {}
                        cont => self.unget(Token {
                            cont,
                            span: span.clone(),
                        }),
                    };
                    push_expr!(E::Keyword(KeywordKind::Ifs { branches }));
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::Fn(scope))) => MakeFnArgs(scope),
                (MakeFnArgs(scope), FnArgs(args)) => MakeFnName(scope, crate::FnArgs::Args(args)),
                (MakeFnArgs(scope), Ident(i)) => match i.as_str() {
                    "*" => MakeFnName(scope, crate::FnArgs::AllStack),
                    x => panic!("Can only user param list or '*' as function arguments, not {x}"),
                },
                (MakeFnName(scope, args), Ident(name)) => MakeFnBlock(scope, args, FnName(name)),
                (MakeFnBlock(scope, args, name), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let code = inner_ctx.parse_block()?;
                    let fndef = E::Keyword(KeywordKind::FnDef {
                        name,
                        scope,
                        code,
                        args,
                    });
                    push_expr!(fndef);
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::While)) => MakeWhile,
                (MakeWhile, Block(check)) => {
                    let mut inner_ctx = Context::new(check);
                    let check = inner_ctx.parse_block()?;
                    MakeWhileCode(check)
                }
                (MakeWhileCode(check), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let code = inner_ctx.parse_block()?;
                    push_expr!(E::Keyword(KeywordKind::While { check, code }));
                    Nothing
                }

                (s, t) => {
                    panic!("Parser: State {s:?} doesn't accept token {t:?}")
                }
            }
        }
        Ok(out)
    }

    fn unget(&mut self, token: Token) {
        assert!(self.ungotten.is_none());
        self.ungotten = Some(token);
    }

    fn next(&mut self) -> Option<Token> {
        match self.ungotten.take() {
            None => self.code.pop(),
            x => x,
        }
    }

    pub fn new(mut tokens: Vec<Token>) -> Self {
        tokens.reverse();
        Self {
            code: tokens,
            ungotten: None,
        }
    }
}
