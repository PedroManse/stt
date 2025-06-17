use colored::Colorize;
use stck::{api::*, error::StckErrorCase};

#[derive(PartialEq, Clone, Copy)]
enum StckMode {
    Normal,
    Debug,
    SyntaxCheck,
    TokenCheck,
    PrintProccCode,
}

fn print_code(code: &stck::Code, import_stack: usize) {
    for expr in code {
        if import_stack != 0 {
            println!("{} {}", ">".repeat(import_stack).blue(), expr.cont);
        } else {
            println!("{}", expr.cont);
        }
        if let stck::ExprCont::IncludedCode(code) = &expr.cont {
            print_code(code, import_stack+1);
        }
    }
}

fn execute(mode: StckMode, file_path: String) -> Result<(), StckErrorCase> {
    use StckMode as M;
    match mode {
        M::Normal => {
            execute_file(file_path)?;
        }
        M::Debug => {
            todo!()
        }
        M::SyntaxCheck => {
            get_project_code(file_path)?;
        }
        M::PrintProccCode => {
            let code = get_project_code(file_path)?;
            print_code(&code, 0);
        }
        M::TokenCheck => {
            get_tokens(file_path)?;
        }
    }
    Ok(())
}

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    let Some(file_path) = args.next() else {
        eprintln!("Missing file to execute");
        std::process::exit(1);
    };
    let mode = if let Some(arg) = args.peek() {
        let m = match arg.as_str() {
            "--debug" => StckMode::Debug,
            "--token" => StckMode::TokenCheck,
            "--syntax" => StckMode::SyntaxCheck,
            "--proc" => StckMode::PrintProccCode,
            _ => StckMode::Normal,
        };
        if m != StckMode::Normal {
            args.next();
        }
        m
    } else {
        StckMode::Normal
    };
    if let Err(e) = execute(mode, file_path.clone()) {
        eprintln!("{e}");
        if let crate::StckErrorCase::Context(e) = e {
            let spans: stck::error::ErrorSpans = e.into();
            let sources = spans.try_into_sources().unwrap();
            for source in sources {
                println!("{source}");
            }
        }
        std::process::exit(1);
    }
}
