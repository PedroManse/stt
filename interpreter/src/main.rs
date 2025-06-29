use clap::{Parser, ValueEnum};
use colored::Colorize;
use stck::prelude::*;
use std::path::PathBuf;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum StckMode {
    /// Normal execution
    Normal,
    /// Normal execution but dump variables and stack on error
    Debug,
    /// Check valdity of tokens
    TokenCheck,
    /// Check vailidity of expressions and pre-processor commands
    SyntaxCheck,
    /// Dump code after pre-processing
    PrintCode,
}

fn print_code(code: &stck::internals::Code, import_stack: usize) {
    for expr in code {
        if import_stack != 0 {
            println!(
                "{} {} @ {}",
                ">".repeat(import_stack).blue(),
                expr.cont,
                expr.span
            );
        } else {
            println!("{} @ {}", expr.cont, expr.span);
        }
        if let stck::internals::ExprCont::IncludedCode(code) = &expr.cont {
            print_code(code, import_stack + 1);
        }
    }
}

fn execute(
    mode: StckMode,
    file_path: PathBuf,
    file_cache: &mut impl cache::FileCacher,
) -> Result<(), Error> {
    use StckMode as M;
    match mode {
        M::Normal => {
            execute_file(file_path, file_cache)?;
        }
        M::Debug => {
            // execute and print values and stack on error
            todo!()
        }
        M::SyntaxCheck => {
            get_project_code(file_path, file_cache)?;
        }
        M::PrintCode => {
            let code = get_project_code(file_path, file_cache)?;
            print_code(&code, 0);
        }
        M::TokenCheck => {
            get_tokens(file_path, file_cache)?;
        }
    }
    Ok(())
}

#[derive(Parser, Debug)]
struct Cli {
    file: PathBuf,

    #[arg(short, long, value_name = "Mode", default_value = "normal")]
    mode: StckMode,
}

fn main() {
    let Cli { file, mode } = Cli::parse();
    let mut file_cacher = CacheHelper::new();
    if let Err(e) = execute(mode, file, &mut file_cacher) {
        eprintln!("{e}");
        if let Error::RuntimeError(error::RuntimeError::RuntimeCtx(e)) = e {
            let spans: stck::error::ErrorSpans = e.into();
            let sources = spans.try_into_sources(&mut file_cacher).unwrap();
            for source in sources {
                println!("{source}");
            }
        }
        std::process::exit(1);
    }
}
