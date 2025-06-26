use std::path::PathBuf;
use std::str::FromStr;

use crate::{
    DefinedGenericBuilder, FnArgDef, FnScope, LineRange, RawKeyword, StckError, Token, TokenBlock,
    TokenCont,
};

type Result<T> = std::result::Result<T, StckError>;

pub struct Context {
    current_line: usize,
    last_line: usize,
    point: usize,
    chars: Vec<char>,
}

#[derive(Debug)]
pub enum State {
    Nothing,
    OnComment,
    MakeIdent(String),
    MakeString(String),
    MakeStringEsc(String), // found \ on string
    MakeNumber(String),
    MakeKeyword(String),
    MakeFnArgs(Vec<FnArgDef>, String),
    MakeFnArgType {
        args: Vec<FnArgDef>,
        arg_name: String,
        type_buf: String,
        tag_count: usize,
    },
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
    (arg_type) => {
        (matches!(letter) | matches!(space) | '?' | '*')
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
        self.last_line = self.current_line;
        let span = LineRange::from_points(self.last_line, self.current_line+1);
        out.push(Token { cont: token, span });
    }
    pub fn tokenize(mut self, source: PathBuf) -> Result<TokenBlock> {
        let tokens = self.tokenize_block()?;
        Ok(TokenBlock { source, tokens })
    }

    // just read a '{'
    fn tokenize_block(&mut self) -> Result<Vec<Token>> {
        use State::*;
        use TokenCont::*;
        let mut state = Nothing;
        let mut out = Vec::new();

        while let Some(ch) = self.next() {
            state = match (state, ch) {
                (Nothing, '}') => {
                    self.last_line = self.current_line;
                    self.push_token(&mut out, EndOfBlock);
                    return Ok(out);
                }
                (Nothing, '{') => {
                    // keep start of block's span = to where { is
                    // but use end of block span = to where } is
                    let last_token_line = self.last_line;
                    let block = self.tokenize_block()?;
                    // whyyyyy??
                    self.last_line = last_token_line;
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
                            let trc = otherwise
                                .strip_prefix("TRC")
                                .map(str::trim)
                                .map(DefinedGenericBuilder::from_str)
                                .and_then(Result::ok)
                                .map(RawKeyword::from);
                            include
                                .or(pragma)
                                .or(fn_into_closure)
                                .or(trc)
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
                (MakeFnArgs(args, arg_name), '<') => MakeFnArgType {
                    args,
                    arg_name,
                    type_buf: String::new(),
                    tag_count: 0,
                },
                (
                    MakeFnArgType {
                        args,
                        arg_name,
                        mut type_buf,
                        tag_count,
                    },
                    c @ matches!(arg_type),
                ) => {
                    type_buf.push(*c);
                    MakeFnArgType {
                        args,
                        arg_name,
                        type_buf,
                        tag_count,
                    }
                }
                (
                    MakeFnArgType {
                        args,
                        arg_name,
                        mut type_buf,
                        tag_count,
                    },
                    c @ '<',
                ) => {
                    type_buf.push(*c);
                    MakeFnArgType {
                        args,
                        arg_name,
                        type_buf,
                        tag_count: tag_count + 1,
                    }
                }
                (
                    MakeFnArgType {
                        mut args,
                        arg_name,
                        type_buf,
                        tag_count: 0,
                    },
                    '>',
                ) => {
                    let x = FnArgDef::new_typed(arg_name, type_buf.trim().parse()?);
                    args.push(x);
                    MakeFnArgs(args, String::new())
                }
                (
                    MakeFnArgType {
                        args,
                        arg_name,
                        mut type_buf,
                        tag_count,
                    },
                    c @ '>',
                ) => {
                    type_buf.push(*c);
                    MakeFnArgType {
                        args,
                        arg_name,
                        type_buf,
                        tag_count: tag_count - 1,
                    }
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
                (OnComment, '\n') => Nothing,
                (OnComment, _) => OnComment,

                (Nothing, matches!(space)) => Nothing,

                (s, c) => {
                    return Err(StckError::CantTokenizerChar(s, *c));
                }
            }
        }
        if self.at_eof() {
            match state {
                Nothing | OnComment => {}
                MakeIdent(s) => {
                    self.push_token(&mut out, Ident(s));
                }
                MakeNumber(buf) => {
                    let num = buf.parse()?;
                    self.push_token(&mut out, Number(num));
                }
                s => return Err(StckError::UnexpectedEOF(s)),
            }
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
        if ch == &'\n' {
            self.current_line += 1;
        }
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
            last_line: 1,
            current_line: 1,
        }
    }
}
