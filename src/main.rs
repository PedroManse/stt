use stt::*;

#[derive(PartialEq)]
enum SttMode {
    Normal,
    Debug,
    SyntaxCheck,
    TokenCheck,
    PrintProccCode,
}

fn execute(mode: SttMode, file_path: String) -> Result<()> {
    use SttMode as M;
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
            "--debug" => SttMode::Debug,
            "--token" => SttMode::TokenCheck,
            "--syntax" => SttMode::SyntaxCheck,
            "--proc" => SttMode::PrintProccCode,
            _ => SttMode::Normal,
        };
        if m != SttMode::Normal {
            args.next();
        }
        m
    } else {
        SttMode::Normal
    };
    if let Err(e) = execute(mode, file_path.clone()) {
        eprintln!("[ERROR] executing {file_path}:\n  {e}");
        std::process::exit(1);
    }
}
