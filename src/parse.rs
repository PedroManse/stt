use crate::*;

pub struct Context<'p> {
    code: Vec<Token>,
    source: &'p Path,
    ungotten: Option<Token>, // token to be re-parsed by different state
}

#[derive(Debug)]
pub enum State {
    Nothing,

    MakeIfs(Vec<CondBranch>),
    MakeIfsCode {
        branches: Vec<CondBranch>,
        check: Vec<Expr>,
    },

    MakeFnArgs(FnScope),
    MakeFnNameOrOutArgs(FnScope, FnArgs),
    MakeFnName(FnScope, FnArgs, Vec<FnArgDef>),
    MakeFnBlock(FnScope, FnArgs, FnName, Option<Vec<FnArgDef>>),

    MakeSwitch(Vec<SwitchCase>),
    MakeSwitchCode(Vec<SwitchCase>, Value),

    MakeWhile,
    MakeWhileCode(Vec<Expr>),

    MakeClosureBlock(Vec<FnArgDef>),
}

impl<'p> Context<'p> {
    pub fn parse_block(&mut self) -> Result<Vec<Expr>> {
        self.parse_block_start(0)
    }

    fn parse_block_start(&mut self, span_start: usize) -> Result<Vec<Expr>> {
        use ExprCont as E;
        use State::*;
        use TokenCont::*;
        let mut state = Nothing;
        let mut out = vec![];
        let mut cum_span = span_start..span_start;

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
                    push_expr!(E::FnCall(n));
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
                (Nothing, Char(c)) => {
                    push_expr!(E::Immediate(Value::Char(c)));
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
                    let mut inner_ctx = Context::new(code.tokens, self.source);
                    let parsed_code = inner_ctx.parse_block_start(cum_span.start)?;
                    push_expr!(E::IncludedCode(Code {
                        source: code.source,
                        exprs: parsed_code
                    }));
                    s
                }

                (Nothing, FnArgs(args)) => MakeClosureBlock(args),
                (MakeClosureBlock(args), Block(code)) => {
                    let mut inner_ctx = Context::new(code, self.source);
                    let code = inner_ctx.parse_block_start(cum_span.start)?;
                    let closure = Closure {
                        code,
                        request_args: ClosurePartialArgs::parse(args, span.clone())?,
                        output_types: None,
                    };
                    push_expr!(E::Immediate(Value::Closure(Box::new(closure))));
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::Switch)) => MakeSwitch(vec![]),
                (MakeSwitch(cases), Char(c)) => MakeSwitchCode(cases, Value::Char(c)),
                (MakeSwitch(cases), Str(v)) => MakeSwitchCode(cases, Value::Str(v)),
                (MakeSwitch(cases), Number(v)) => MakeSwitchCode(cases, Value::Num(v)),
                (MakeSwitchCode(mut cases, test), Block(code)) => {
                    let mut inner_ctx = Context::new(code, self.source);
                    let code = inner_ctx.parse_block_start(cum_span.start)?;
                    cases.push(SwitchCase { test, code });
                    MakeSwitch(cases)
                }
                (MakeSwitch(cases), Block(code)) => {
                    let mut inner_ctx = Context::new(code, self.source);
                    let code = inner_ctx.parse_block_start(cum_span.start)?;
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
                    }
                    push_expr!(E::Keyword(KeywordKind::Switch {
                        cases,
                        default: None,
                    }));
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::Ifs)) => MakeIfs(vec![]),
                (MakeIfs(branches), Block(code)) => {
                    let mut inner_ctx = Context::new(code, self.source);
                    let check = inner_ctx.parse_block_start(cum_span.start)?;
                    MakeIfsCode { branches, check }
                }
                (
                    MakeIfsCode {
                        mut branches,
                        check,
                    },
                    Block(code),
                ) => {
                    let mut inner_ctx = Context::new(code, self.source);
                    let code = inner_ctx.parse_block_start(cum_span.start)?;
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
                    }
                    push_expr!(E::Keyword(KeywordKind::Ifs { branches }));
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::Fn(scope))) => MakeFnArgs(scope),
                (MakeFnArgs(scope), FnArgs(args)) => {
                    MakeFnNameOrOutArgs(scope, crate::FnArgs::Args(args))
                }
                (MakeFnArgs(scope), Ident(i)) => match i.as_str() {
                    "*" => MakeFnNameOrOutArgs(scope, crate::FnArgs::AllStack),
                    _ => return Err(StckError::WrongParamList(i, self.source.to_path_buf())),
                },
                (MakeFnNameOrOutArgs(scope, args), FnArgs(out_args)) => {
                    MakeFnName(scope, args, out_args)
                }
                (MakeFnNameOrOutArgs(scope, args), Ident(name)) => {
                    MakeFnBlock(scope, args, name, None)
                }
                (MakeFnName(scope, args, out_args), Ident(name)) => {
                    MakeFnBlock(scope, args, name, Some(out_args))
                }
                (MakeFnBlock(scope, args, name, out_args), Block(code)) => {
                    let mut inner_ctx = Context::new(code, self.source);
                    let code = inner_ctx.parse_block_start(span.start)?;
                    let fndef = E::Keyword(KeywordKind::FnDef {
                        name,
                        scope,
                        code,
                        args,
                        out_args,
                    });
                    push_expr!(fndef);
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::While)) => MakeWhile,
                (MakeWhile, Block(check)) => {
                    let mut inner_ctx = Context::new(check, self.source);
                    let check = inner_ctx.parse_block_start(cum_span.start)?;
                    MakeWhileCode(check)
                }
                (MakeWhileCode(check), Block(code)) => {
                    let mut inner_ctx = Context::new(code, self.source);
                    let code = inner_ctx.parse_block_start(cum_span.start)?;
                    push_expr!(E::Keyword(KeywordKind::While { check, code }));
                    Nothing
                }

                (s, t) => {
                    return Err(StckError::CantParseToken(
                        s,
                        Box::new(t),
                        self.source.to_path_buf(),
                    ));
                }
            };
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

    pub fn new(mut tokens: Vec<Token>, source: &'p Path) -> Self {
        tokens.reverse();
        Self {
            source,
            code: tokens,
            ungotten: None,
        }
    }
}
