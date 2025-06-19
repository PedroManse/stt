use crate::error::Error;
use crate::*;
use std::collections::HashSet;
use std::path::Path;

enum ProcChange {
    Keep,
    Pop,
    PushIf { reading: bool },
    PushElse,
}

struct ProcStatus {
    status: ProcCommand,
    reading: bool,
}

#[derive(Debug, PartialEq)]
pub enum ProcCommand {
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
    pub fn parse_clean(&'p self, code: Vec<Token>) -> Result<Vec<Token>, Error> {
        let mut proc_vars: HashSet<String> = HashSet::new();
        self.parse(code, &mut proc_vars)
    }

    pub fn parse(
        &'p self,
        code: Vec<Token>,
        proc_vars: &mut HashSet<String>,
    ) -> Result<Vec<Token>, Error> {
        // TODO would have to keep track of removed span from pragma lines
        let mut if_stack: Vec<ProcStatus> = vec![];
        let mut out = Vec::with_capacity(code.len());
        for Token { cont, span } in code {
            match cont {
                TokenCont::Keyword(RawKeyword::Include { path }) => {
                    let include_path = self.dir.join(path);
                    let metadata = include_path
                        .metadata()
                        .ok()
                        .ok_or(StckError::CantReadFile(include_path.clone()))?;
                    let included_tokens = if metadata.is_dir() {
                        api::get_tokens_with_procvars(include_path.join("stck.stck"), proc_vars)
                    } else {
                        api::get_tokens_with_procvars(include_path, proc_vars)
                    }?;
                    let included_tokens = TokenCont::IncludedBlock(included_tokens);
                    let included_tokens = Token {
                        cont: included_tokens,
                        span,
                    };
                    out.push(included_tokens);
                }
                TokenCont::Keyword(RawKeyword::Pragma { command }) => {
                    manage_pragma(&mut if_stack, &command, proc_vars, span)?;
                }
                x if if_stack.last().is_none_or(|s| s.reading) => {
                    out.push(Token { cont: x, span });
                }
                _ => {} // ignore code in IgnoringIf or IgnoringElse status
            }
        }
        Ok(out)
    }
}

fn manage_pragma(
    if_stack: &mut Vec<ProcStatus>,
    command: &str,
    proc_vars: &mut HashSet<String>,
    span: Range<usize>,
) -> Result<(), Error> {
    let is_reading = if_stack.last().is_none_or(|s| s.reading);
    let proc_cmd = execute_command(command, proc_vars)?;
    match proc_cmd {
        ProcChange::Keep => {}
        ProcChange::Pop => {
            if_stack.pop().ok_or(StckError::NoSectionToClose(span))?;
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
            s => return Err(StckError::CantElseCurrentSection(span, s).into()),
        },
    }
    Ok(())
}

fn execute_command(command: &str, proc_vars: &mut HashSet<String>) -> Result<ProcChange, Error> {
    let cmd_parts: Vec<&str> = command.split(' ').collect();
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
            proc_vars.insert((*v).to_string());
            ProcChange::Keep
        }
        ["unset", v] => {
            proc_vars.remove(*v);
            ProcChange::Keep
        }
        e => {
            return Err(StckError::InvalidPragma(e.join(" ")).into());
        }
    })
}
