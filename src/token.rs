use crate::{FnScope, RawKeyword, Result, Token, TokenCont};

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
    MakeFnArgs(Vec<String>, String),
}

macro_rules! matches {
    (ident) => {
        (matches!(start_ident) | matches!(digit) | '.' | '/')
    };
    (start_ident) => {
        'a'..='z' | 'A'..='Z' | '+' | '_' | '%' | '!' | '?' | '$' | '-' | '=' | '*' | '&' | '<' | '>' | '≃' | ',' | ':' | '~'
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
        //let mut push_tkn = |token, point| {
        //    let span = self.last_token_pos..point;
        //    out.push(Token { cont: token, span });
        //    self.last_token_pos = point;
        //};
        ////use $token.into
        //macro_rules! push_token {
        //    ($token:expr) => {
        //        push_tkn($token, self.point)
        //    };
        //}

        while let Some(ch) = self.next() {
            state = match (state, ch) {
                (Nothing, '}') => {
                    self.push_token(&mut out, EndOfBlock);
                    return Ok(out);
                }
                (Nothing, '{') => {
                    let block = self.tokenize_block()?;
                    self.push_token(&mut out, Block(block));
                    Nothing
                }
                (Nothing, c @ matches!(start_ident)) => MakeIdent(String::from(*c)),
                (Nothing, '"') => MakeString(String::new()),
                (Nothing, c @ matches!(digit)) => MakeNumber(String::from(*c)),
                (Nothing, '(') => MakeKeyword(String::new()),
                (Nothing, '[') => MakeFnArgs(Vec::new(), String::new()),

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
                        "fn" => RawKeyword::Fn(FnScope::Local),
                        "fn*" => RawKeyword::Fn(FnScope::Global),
                        "fn-" => RawKeyword::Fn(FnScope::Isolated),
                        "while" => RawKeyword::While,
                        "return" => RawKeyword::Return,
                        "switch" => RawKeyword::Switch,
                        "break" => RawKeyword::Break,
                        "ifs" => RawKeyword::Ifs,
                        _ if buf.starts_with("include ") => RawKeyword::Include {
                            path: buf.split_once(" ").unwrap().1.trim().into(),
                        },
                        _ if buf.starts_with("pragma ") => RawKeyword::Pragma {
                            command: buf.split_once(" ").unwrap().1.trim().into(),
                        },
                        _ => {
                            eprintln!("unknown keyword {buf}");
                            return Err(crate::SttError::TodoErr);
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
                        xs.push(buf);
                    }
                    MakeFnArgs(xs, String::new())
                }
                (MakeFnArgs(xs, mut buf), c @ matches!(ident)) => {
                    buf.push(*c);
                    MakeFnArgs(xs, buf)
                }
                (MakeFnArgs(mut xs, buf), ']') => {
                    if !buf.is_empty() {
                        xs.push(buf);
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

                (Nothing, matches!(space)) => Nothing,

                (s, c) => {
                    panic!("Tokenizer: No impl for {s:?} with {c:?}");
                }
            }
        }
        Err(crate::SttError::MissingChar)
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
        let mut chars: Vec<char> = code.chars().collect();
        chars.push('\n'); // to force close comments
        chars.push('}'); // to show EOF
        Self {
            point: 0,
            chars,
            last_token_pos: 0,
        }
    }
}
