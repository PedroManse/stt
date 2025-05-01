use self::token::Token;
use crate::*;
use std::collections::HashSet;
use std::path::Path;

enum ProcChange {
    Keep,
    Pop,
    PushIf { reading: bool },
    PushElse,
}

#[derive(Debug)]
struct ProcStatus {
    status: ProcCommand,
    reading: bool,
}

#[derive(Debug, PartialEq)]
enum ProcCommand {
    If,
    IfElse,
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
        let mut if_stack: Vec<ProcStatus> = vec![];
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
                    manage_pragma(&mut if_stack, command, proc_vars)?;
                }
                x if if_stack.last().map(|s| s.reading).unwrap_or(true) => {
                    out.push(x);
                }
                _ => {} // ignore code in IgnoringIf or IgnoringElse status
            }
        }
        Ok(out)
    }
}

fn manage_pragma(
    if_stack: &mut Vec<ProcStatus>,
    command: String,
    proc_vars: &mut HashSet<String>,
) -> Result<()> {
    let is_reading = if_stack.last().map(|s| s.reading).unwrap_or(true);
    let proc_cmd = execute_command(command, proc_vars)?;
    match proc_cmd {
        ProcChange::Keep => {}
        ProcChange::Pop => {
            if_stack.pop().ok_or(SttError::TodoErr)?;
        }
        ProcChange::PushIf { reading } => {
            if_stack.push(ProcStatus {
                status: ProcCommand::If,
                reading: reading && is_reading,
            });
        }
        ProcChange::PushElse => match if_stack.pop().map(|x| x.status) {
            Some(ProcCommand::If) => {
                if_stack.push(ProcStatus {
                    status: ProcCommand::IfElse,
                    reading: !is_reading,
                });
            }
            _ => return Err(SttError::TodoErr),
        },
    };
    Ok(())
}

fn execute_command(command: String, proc_vars: &mut HashSet<String>) -> Result<ProcChange> {
    let cmd_parts: Vec<&str> = command.split(" ").collect();
    Ok(match cmd_parts.as_slice() {
        ["if", v] => ProcChange::PushIf {
            reading: proc_vars.contains(*v),
        },
        ["if", "not", v] => ProcChange::PushIf {
            reading: !proc_vars.contains(*v),
        },
        ["else"] => ProcChange::PushElse,
        ["end", "if"] => ProcChange::Pop,
        ["set", v] => {
            proc_vars.insert(v.to_string());
            ProcChange::Keep
        }
        ["unset", v] => {
            proc_vars.remove(*v);
            ProcChange::Keep
        }
        e => {
            println!("{e:?}");
            return Err(SttError::TodoErr);
        }
    })
}
