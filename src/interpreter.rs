use stck::{Result, api::*};

#[derive(PartialEq)]
enum StckMode {
    Normal,
    Debug,
    SyntaxCheck,
    TokenCheck,
    PrintProccCode,
}

fn execute(mode: StckMode, file_path: String) -> Result<()> {
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
            println!("{:?}", get_project_code(file_path)?);
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
        eprintln!("[ERROR] executing {file_path}:\n  {e}");
        std::process::exit(1);
    }
}
