use self::token::{RawKeyword, Token};
use crate::*;
use std::path::Path;

pub struct Context<'p> {
    dir: &'p Path,
}

impl<'p> Context<'p> {
    pub fn new(dir: &'p Path) -> Self {
        Context { dir }
    }
}

impl<'p> Context<'p> {
    pub fn parse(&'p self, code: Vec<token::Token>) -> Result<Vec<token::Token>> {
        let mut out = Vec::with_capacity(code.len());
        for c in code {
            match c {
                Token::Keyword(RawKeyword::Include { path }) => {
                    let include_path = self.dir.join(path);
                    let metadata = include_path.metadata().ok().ok_or(SttError::CantReadFile(include_path.clone()))?;
                    let mut included_tokens = if metadata.is_dir() {
                        get_tokens(include_path.join("stt.stt"))
                    } else {
                        get_tokens(include_path)
                    }?;
                    out.append(&mut included_tokens);
                }
                x => {
                    out.push(x);
                }
            }
        }
        Ok(out)
    }
}
