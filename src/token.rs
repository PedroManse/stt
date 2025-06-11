use crate::{FnArgDef, FnScope, RawKeyword, Result, StckError, Token, TokenCont};

pub struct Context {
    point: usize,
    last_token_pos: usize,
    chars: Vec<char>,
}

#[derive(Debug)]
enum State {
    Nothing,
    OnComment,
    MakeIdent(String),
    MakeString(String),
    MakeStringEsc(String), // found \ on string
    MakeNumber(String),
    MakeKeyword(String),
    MakeFnArgs(Vec<FnArgDef>, String),
    MakeFnArgType(Vec<FnArgDef>, String, String),
    MakeChar,
    MakeCharEnd(char),
    MakeCharEndEsc(char),
}

macro_rules! matches {
    (arg_ident) => {
        'a'..='z' | 'A'..='Z' | '_' | '-' | '&'
    };
    (ident) => {
        (matches!(start_ident) | matches!(digit) | '.' | '/' | '\'')
    };
    (letter) => {
        'a'..='z' | 'A'..='Z'
    };
    (start_ident) => {
        'a'..='z' | 'A'..='Z' | '+' | '_' | '%' | '!' | '?' | '$' | '-' | '=' | '*' | '&' | '<' | '>' | '≃' | ',' | ':' | '~' | '@'
    };
    (word_edge) => {
        '(' | ')' | '{' | '}' | '[' | ']'
    };
    (space) => {
        ' ' | '\n' | '\t'
    };
    (digit) => {
        ('0'..='9')
    }
}

impl Context {
    fn push_token(&mut self, out: &mut Vec<Token>, token: TokenCont) {
        let span = self.last_token_pos..self.point;
        out.push(Token { cont: token, span });
        self.last_token_pos = self.point;
    }

    // just read a '{'
    pub fn tokenize_block(&mut self) -> Result<Vec<Token>> {
        use State::*;
        use TokenCont::*;
        let mut state = Nothing;
        let mut out = Vec::new();

        while let Some(ch) = self.next() {
            state = match (state, ch) {
                (Nothing, '}') => {
                    self.last_token_pos = self.point;
                    self.push_token(&mut out, EndOfBlock);
                    return Ok(out);
                }
                (Nothing, '{') => {
                    // keep start of block's span = to where { is
                    // but use end of block span = to where } is
                    let last_token_pos = self.last_token_pos;
                    let block = self.tokenize_block()?;
                    self.last_token_pos = last_token_pos;
                    self.push_token(&mut out, Block(block));
                    Nothing
                }
                (Nothing, c @ matches!(start_ident)) => MakeIdent(String::from(*c)),
                (Nothing, '"') => MakeString(String::new()),
                (Nothing, c @ matches!(digit)) => MakeNumber(String::from(*c)),
                (Nothing, '(') => MakeKeyword(String::new()),
                (Nothing, '[') => MakeFnArgs(Vec::new(), String::new()),
                (Nothing, '\'') => MakeChar,
                (MakeChar, c) => MakeCharEnd(*c),
                (MakeCharEnd('\\'), c @ ('\\' | '\'')) => MakeCharEndEsc(*c),
                (MakeCharEnd(c), '\'') => {
                    self.push_token(&mut out, Char(c));
                    Nothing
                }
                (MakeCharEnd('\\'), 'n') => MakeCharEndEsc('\n'),
                (MakeCharEndEsc(c), '\'') => {
                    self.push_token(&mut out, Char(c));
                    Nothing
                }

                (MakeIdent(mut buf), c @ matches!(ident)) => {
                    buf.push(*c);
                    MakeIdent(buf)
                }
                (MakeIdent(buf), matches!(space)) => {
                    self.push_token(&mut out, Ident(buf));
                    Nothing
                }
                (MakeIdent(buf), matches!(word_edge)) => {
                    self.push_token(&mut out, Ident(buf));
                    self.unget(); // re-read char with Nothing State
                    Nothing
                }

                (MakeString(buf), '"') => {
                    self.push_token(&mut out, Str(buf));
                    Nothing
                }
                (MakeString(buf), '\\') => MakeStringEsc(buf),
                (MakeString(mut buf), c) => {
                    buf.push(*c);
                    MakeString(buf)
                }
                (MakeStringEsc(mut buf), '\\') => {
                    buf.push('\\');
                    MakeString(buf)
                }
                (MakeStringEsc(mut buf), 'n') => {
                    buf.push('\n');
                    MakeString(buf)
                }

                (MakeNumber(mut buf), c @ matches!(digit)) => {
                    buf.push(*c);
                    MakeNumber(buf)
                }
                (MakeNumber(buf), matches!(space)) => {
                    let num = buf.parse()?;
                    self.push_token(&mut out, Number(num));
                    Nothing
                }
                (MakeNumber(buf), matches!(word_edge)) => {
                    let num = buf.parse()?;
                    self.push_token(&mut out, Number(num));
                    self.unget(); // re-read char with Nothing State
                    Nothing
                }

                (MakeKeyword(buf), ')') => {
                    let kw = match buf.as_str().trim() {
                        "!" => RawKeyword::BubbleError,
                        "fn" => RawKeyword::Fn(FnScope::Local),
                        "fn*" => RawKeyword::Fn(FnScope::Global),
                        "fn-" => RawKeyword::Fn(FnScope::Isolated),
                        "while" => RawKeyword::While,
                        "return" => RawKeyword::Return,
                        "switch" => RawKeyword::Switch,
                        "break" => RawKeyword::Break,
                        "ifs" => RawKeyword::Ifs,
                        otherwise => {
                            let include =
                                otherwise
                                    .strip_prefix("include ")
                                    .map(|p| RawKeyword::Include {
                                        path: p.trim().into(),
                                    });
                            let pragma = otherwise
                                .strip_prefix("pragma ")
                                .map(|c| RawKeyword::Pragma { command: c.into() });
                            let fn_into_closure = otherwise
                                .strip_prefix("@")
                                .map(|f| RawKeyword::FnIntoClosure { fn_name: f.into() });
                            include
                                .or(pragma)
                                .or(fn_into_closure)
                                .ok_or(StckError::UnknownKeyword(otherwise.to_string()))?
                        }
                    };
                    self.push_token(&mut out, Keyword(kw));
                    Nothing
                }
                (MakeKeyword(mut buf), c) => {
                    buf.push(*c);
                    MakeKeyword(buf)
                }

                (MakeFnArgs(mut xs, buf), matches!(space)) => {
                    if !buf.is_empty() {
                        xs.push(FnArgDef::new_untyped(buf));
                    }
                    MakeFnArgs(xs, String::new())
                }
                (MakeFnArgs(xs, buf), '<') => {
                    MakeFnArgType(xs, buf, String::new())
                }
                (MakeFnArgType(xs, buf, mut type_buf), c @ matches!(letter)) => {
                    type_buf.push(*c);
                    MakeFnArgType(xs, buf, type_buf)
                }
                (MakeFnArgType(mut xs, buf, type_buf), '>') => {
                    let x = FnArgDef::new_typed(buf, type_buf.parse()?);
                    xs.push(x);
                    MakeFnArgs(xs, String::new())
                }
                (MakeFnArgs(xs, mut buf), c @ matches!(arg_ident)) => {
                    buf.push(*c);
                    MakeFnArgs(xs, buf)
                }
                (MakeFnArgs(mut xs, buf), ']') => {
                    if !buf.is_empty() {
                        xs.push(FnArgDef::new_untyped(buf));
                    }
                    self.push_token(&mut out, FnArgs(xs));
                    Nothing
                }

                (Nothing, '#') => OnComment,
                (OnComment, '\n') => {
                    self.last_token_pos = self.point;
                    Nothing
                }
                (OnComment, _) => OnComment,

                (Nothing, matches!(space)) => {
                    self.last_token_pos += 1;
                    Nothing
                }

                (s, c) => {
                    panic!("Tokenizer: No impl for {s:?} with {c:?}");
                }
            }
        }
        if self.at_eof() {
            self.last_token_pos = self.point;
            self.push_token(&mut out, EndOfBlock);
            Ok(out)
        } else {
            Err(crate::StckError::MissingChar)
        }
    }

    fn at_eof(&self) -> bool {
        self.point == self.chars.len()
    }

    fn next(&mut self) -> Option<&char> {
        let ch = self.chars.get(self.point)?;
        self.point += 1;
        Some(ch)
    }

    // to re-read char with differnt State
    fn unget(&mut self) {
        self.point -= 1;
    }

    pub fn new(code: &str) -> Self {
        let chars: Vec<char> = code.chars().collect();
        Self {
            point: 0,
            chars,
            last_token_pos: 0,
        }
    }
}
