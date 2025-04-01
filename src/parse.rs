use crate::token::{RawKeyword, Token};
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
        check: Code,
    },

    MakeFnArgs(FnScope),
    MakeFnName(FnScope, FnArgs),
    MakeFnBlock(FnScope, FnArgs, FnName),

    MakeWhile,
    MakeWhileCode(Code),
}

impl Context {
    pub fn parse_block(&mut self) -> Result<Vec<Expr>, ()> {
        use Expr as E;
        use State::*;
        use Token::*;
        let mut state = Nothing;
        let mut out = vec![];
        while let Some(token) = self.next() {
            state = match (state, token) {
                (Nothing, EndOfBlock) => Nothing,
                (Nothing, Ident(n)) => {
                    out.push(E::FnCall(FnName(n)));
                    Nothing
                }
                (Nothing, Str(x)) => {
                    out.push(E::Immediate(Value::Str(x)));
                    Nothing
                }
                (Nothing, Number(x)) => {
                    out.push(E::Immediate(Value::Num(x)));
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::Ifs)) => MakeIfs(vec![]),
                (MakeIfs(branches), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let check = Code(inner_ctx.parse_block()?);
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
                    let code = Code(inner_ctx.parse_block()?);
                    branches.push(CondBranch { check, code });
                    MakeIfs(branches)
                }
                (MakeIfs(branches), t) => {
                    match t {
                        EndOfBlock=>{},
                        t => self.unget(t),
                    };
                    out.push(E::Keyword(KeywordKind::Ifs { branches }));
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::Fn(scope))) => MakeFnArgs(scope),
                (MakeFnArgs(scope), FnArgs(args)) => MakeFnName(scope, crate::FnArgs::Args(args)),
                (MakeFnArgs(scope), Ident(i)) => {
                    match i.as_str() {
                        "*" => MakeFnName(scope, crate::FnArgs::AllStack),
                        x => panic!("Can only user param list or '*' as function arguments, not {x}")
                    }
                }
                (MakeFnName(scope, args), Ident(name)) => MakeFnBlock(scope, args, FnName(name)),
                (MakeFnBlock(scope, args, name), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let code = Code(inner_ctx.parse_block()?);
                    let fndef = E::Keyword(KeywordKind::FnDef {
                        name,
                        scope,
                        code,
                        args,
                    });
                    out.push(fndef);
                    Nothing
                }

                (Nothing, Keyword(RawKeyword::While)) => {
                    MakeWhile
                }
                (MakeWhile, Block(check)) => {
                    let mut inner_ctx = Context::new(check);
                    let check = Code(inner_ctx.parse_block()?);
                    MakeWhileCode(check)
                }
                (MakeWhileCode(check), Block(code)) => {
                    let mut inner_ctx = Context::new(code);
                    let code = Code(inner_ctx.parse_block()?);
                    out.push(E::Keyword(KeywordKind::While { check, code }));
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
        self.ungotten.take().or(self.code.pop())
    }

    pub fn new(mut code: Vec<Token>) -> Self {
        code.reverse();
        Self {
            code,
            ungotten: None,
        }
    }
}
