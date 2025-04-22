use self::token::{RawKeyword, Token};
use crate::*;
use std::collections::HashSet;
use std::path::Path;

#[derive(PartialEq)]
enum ProcStatus {
    Reading,
    IgnoringIf,   // ignoring code because of mistaken if
    IgnoringElse, // ignoring code because of else on correct if
}

pub struct Context<'p> {
    dir: &'p Path,
}

impl<'p> Context<'p> {
    pub fn new(dir: &'p Path) -> Self {
        Context { dir }
    }
}

impl<'p> Context<'p> {
    pub fn parse_clean(&'p self, code: Vec<token::Token>) -> Result<Vec<token::Token>> {
        let mut proc_vars: HashSet<String> = HashSet::new();
        self.parse(code, &mut proc_vars)
    }

    pub fn parse(
        &'p self,
        code: Vec<token::Token>,
        proc_vars: &mut HashSet<String>,
    ) -> Result<Vec<token::Token>> {
        let mut status = ProcStatus::Reading;
        let mut out = Vec::with_capacity(code.len());
        for c in code {
            match c {
                Token::Keyword(RawKeyword::Include { path }) => {
                    let include_path = self.dir.join(path);
                    let metadata = include_path
                        .metadata()
                        .ok()
                        .ok_or(SttError::CantReadFile(include_path.clone()))?;
                    let mut included_tokens = if metadata.is_dir() {
                        get_tokens_with_procvars(include_path.join("stt.stt"), proc_vars)
                    } else {
                        get_tokens_with_procvars(include_path, proc_vars)
                    }?;
                    out.append(&mut included_tokens);
                }
                Token::Keyword(RawKeyword::Pragma { command }) => {
                    status = execute_command(command, proc_vars, status)?;
                }
                x if status == ProcStatus::Reading => {
                    out.push(x);
                }
                _ => {} // ignore code in IgnoringIf or IgnoringElse status
            }
        }
        Ok(out)
    }
}

fn execute_command(
    command: String,
    proc_vars: &mut HashSet<String>,
    status: ProcStatus,
) -> Result<ProcStatus> {
    let cmd_parts: Vec<&str> = command.split(" ").collect();
    Ok(match cmd_parts.as_slice() {
        ["if", v] => {
            if !proc_vars.contains(*v) {
                ProcStatus::IgnoringIf
            } else {
                ProcStatus::Reading
            }
        }
        ["if", "not", v] => {
            if proc_vars.contains(*v) {
                ProcStatus::IgnoringIf
            } else {
                ProcStatus::Reading
            }
        }
        ["else"] => {
            if status == ProcStatus::IgnoringIf {
                ProcStatus::Reading
            } else {
                ProcStatus::IgnoringElse
            }
        }
        ["end", "if"] => ProcStatus::Reading,
        ["set", v] => {
            proc_vars.insert(v.to_string());
            status
        }
        ["unset", v] => {
            proc_vars.remove(*v);
            status
        }
        e => {
            println!("{e:?}");
            return Err(SttError::TodoErr)
        },
    })
}
