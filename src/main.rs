use stt::*;

// TODO error reporting

// TODO make (if) kw for common case of (if) {check} {if-code} {true} {else-code}

// TODO * mode for Ifs to non-exclusive execution

// TODO type checking (fn) [a:num b:string x:array] { ... }
// : would llow for (fn) [x:string] assert$is-string { x }
// : would allow for (fn) [arr:array] assert$arr$of-string { (while) ... { assert$is-string } }

// TODO (pragma)
// : (pragma once) // execute file only once
// : (pramga set <VAR>)
// : (pramga unset <VAR>)
// : (pramga if <VAR> [<IFID>])
// : (pramga endif <VAR> [<IFID>])
//// IFID: string, VAR: string

pub enum SttMode {
    Normal,
    Debug,
    SyntaxCheck,
    TokenCheck,
}

fn main() {
    let mut args = std::env::args().skip(1).peekable();
    let file_path = args.next().unwrap();
    let mode = args.peek();
    let mode = match mode.map(|s| s.as_str()) {
        None => SttMode::Normal,
        Some("--debug") => SttMode::Debug,
        Some("--token") => SttMode::TokenCheck,
        Some("--syntax") => SttMode::SyntaxCheck,
        Some(_) => SttMode::Normal,
    };

    use SttMode as M;
    match mode {
        M::Normal => {
            if let Err(x) = execute_file(file_path) {
                eprintln!("{x}");
            };
        }
        M::Debug => {
            todo!()
        }
        M::SyntaxCheck => {
            get_project_code(file_path).unwrap();
        }
        M::TokenCheck => {
            get_tokens(file_path).unwrap();
        }
    }
}
