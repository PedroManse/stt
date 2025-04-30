use crate::{FnScope, Result};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Token {
    Ident(String),
    Str(String),
    Number(isize),
    Keyword(RawKeyword),
    FnArgs(Vec<String>),
    Block(Vec<Token>),
    EndOfBlock,
}

#[derive(Debug)]
pub enum RawKeyword {
    Fn(FnScope),
    Ifs,
    While,
    Include { path: PathBuf },
    Pragma { command: String },
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
    MakeFnArgs(Vec<String>, String),
}

pub struct Context {
    point: usize,
    chars: Vec<char>,
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
    // just read a '{'
    pub fn tokenize_block(&mut self) -> Result<Vec<Token>> {
        use State::*;
        use Token::*;
        let mut state = Nothing;
        let mut out = Vec::new();
        while let Some(ch) = self.next() {
            state = match (state, ch) {
                (Nothing, '}') => {
                    out.push(EndOfBlock);
                    return Ok(out);
                }
                (Nothing, '{') => {
                    let block = self.tokenize_block()?;
                    out.push(Block(block));
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
                    out.push(Ident(buf));
                    Nothing
                }
                (MakeIdent(buf), matches!(word_edge)) => {
                    out.push(Ident(buf));
                    self.unget(); // re-read char with Nothing State
                    Nothing
                }

                (MakeString(buf), '"') => {
                    out.push(Str(buf));
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
                    out.push(Number(num));
                    Nothing
                }
                (MakeNumber(buf), matches!(word_edge)) => {
                    let num = buf.parse()?;
                    out.push(Number(num));
                    self.unget(); // re-read char with Nothing State
                    Nothing
                }

                (MakeKeyword(buf), ')') => {
                    let kw = match buf.as_str().trim() {
                        "fn" => RawKeyword::Fn(FnScope::Local),
                        "fn*" => RawKeyword::Fn(FnScope::Global),
                        "fn-" => RawKeyword::Fn(FnScope::Isolated),
                        "while" => RawKeyword::While,
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
                    out.push(Keyword(kw));
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
                    out.push(FnArgs(xs));
                    Nothing
                }

                (Nothing, '#') => OnComment,
                (OnComment, '\n') => Nothing,
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
        Self { point: 0, chars }
    }
}
