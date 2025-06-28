use stck::prelude::*;
use std::collections::HashSet;
use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::str::FromStr;

struct Event {
    name: String,
    test: Code,
}

struct Date {
    day: isize,
    month: isize,
    year: isize,
}

fn parse_date(cont: String) -> Result<Date, SError> {
    let rg = regex::Regex::from_str(r#"(?<day>\d{1,2})-(?<month>\d{1,2})-(?<year>\d{1,4})"#)?;
    let caps = rg
        .captures(&cont)
        .ok_or(SError::WrongFormat(cont.to_string()))?;
    let day = caps["day"].parse()?;
    let month = caps["month"].parse()?;
    let year = caps["year"].parse()?;
    Ok(Date { day, month, year })
}

fn parse_file(dir_path: &Path, f: DirEntry, cache: &mut CacheHelper) -> Result<Event, SError> {
    let file_name = f.file_name();
    let name = file_name.to_string_lossy();
    let name = match name.strip_suffix(".stck") {
        Some(a) => a.to_string(),
        None => name.to_string(),
    };
    let tokens = get_tokens(dir_path.join(file_name), cache)?;
    let test = api::parse_raw_tokens(tokens)?;
    Ok(Event {
        name: name.to_string(),
        test,
    })
}

#[derive(Debug, thiserror::Error)]
enum SError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    StckRuntime(#[from] error::RuntimeErrorCtx),
    #[error(transparent)]
    StckError(#[from] error::Error),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("Wrongly formatted date: `{0}`, should be dd-mm-yyyy")]
    WrongFormat(String),
    #[error("Program {0} didn't return anything")]
    DidntReturn(String),
    #[error("Program {0} returned: {1:?} instead of a boolean")]
    WrongReturn(String, stck::internals::Value),
}

fn execute() -> Result<(), SError> {
    let events_dir_name: PathBuf = std::env::var("EVENTS_DIR")
        .unwrap_or("events".to_string())
        .into();
    let events_dir = std::fs::read_dir(&events_dir_name)?;
    let args: Vec<_> = std::env::args()
        .skip(1)
        .map(parse_date)
        .collect::<Result<_, _>>()?;
    let mut cacher = CacheHelper::new();
    let events: Vec<_> = events_dir
        .map(|e| parse_file(&events_dir_name, e?, &mut cacher))
        .collect::<Result<_, _>>()?;

    let mut events_to_show: HashSet<String> = HashSet::new();

    let mut ctx = RuntimeContext::new();
    for arg in args {
        for e in &events {
            ctx.stack.push_this(arg.year);
            ctx.stack.push_this(arg.month);
            ctx.stack.push_this(arg.day);
            ctx.execute_entire_code(&e.test)?;
            let show_event = ctx
                .stack
                .pop_this(stck::internals::Value::get_bool)
                .ok_or(SError::DidntReturn(e.name.clone()))?
                .map_err(|v| SError::WrongReturn(e.name.clone(), v))?;
            if show_event {
                events_to_show.insert(e.name.clone());
            }
        }
    }

    for event in events_to_show {
        println!("{event}");
    }

    Ok(())
}

fn main() {
    if let Err(e) = execute() {
        eprintln!("{e}");
    }
}
