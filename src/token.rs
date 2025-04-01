use crate::FnScope;
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
    // Pragma(PragmaCommand)
}


/*
pub enum PragmaCommand {
    Once
}
*/

/*
**** main.stt ****
(include ./)
...

**** arg.stt ****
(pragma once)
...

*/

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
        (matches!(*ident) | matches!(digit) | '.' | ':')
    };
    (*ident) => {
        'a'..='z' | 'A'..='Z' | '+' | '_' | '%' | '!' | '?' | '$' | '-' | '=' | '*'
    };
    (start_ident) => {
        (matches!(*ident))
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
    pub fn tokenize_block(&mut self) -> Result<Vec<Token>, ()> {
        use State::*;
        use Token::*;
        let mut state = Nothing;
        let mut out = Vec::new();
        loop {
            let ch = self.next().ok_or(())?;
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
                    MakeString(buf)
                }
                (MakeNumber(buf), matches!(space)) => {
                    out.push(Number(buf.parse().unwrap()));
                    Nothing
                }
                (MakeNumber(buf), matches!(word_edge)) => {
                    out.push(Number(buf.parse().unwrap()));
                    self.unget(); // re-read char with Nothing State
                    Nothing
                }

                (MakeKeyword(buf), ')') => {
                    let kw = match buf.as_str() {
                        "fn" => RawKeyword::Fn(FnScope::Local),
                        "fn*" => RawKeyword::Fn(FnScope::Global),
                        "fn-" => RawKeyword::Fn(FnScope::Isolated),
                        "while" => RawKeyword::While,
                        "ifs" => RawKeyword::Ifs,
                        _ if buf.starts_with("include ") => {
                            RawKeyword::Include { path: buf.split_once(" ").unwrap().1.into() }
                        }
                        _ => {
                            eprintln!("unknown keyword {buf}");
                            return Err(());
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
        chars.push('}');
        Self { point: 0, chars }
    }
}
