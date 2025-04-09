use stt::*;

// TODO error reporting

// TODO make (if) kw for common case of (if) {check} {if-code} {true} {else-code}

// TODO * mode for Ifs to non-exclusive execution

// TODO (pragma <command>) // once -> only execute file once // store in execution context

//TODO type checking (fn) [a:num b:string x:array] { ... }
// : would llow for (fn) [x:string] assert$is-string { x }
// : would allow for (fn) [arr:array] assert$arr$of-string { (while) ... { assert$is-string } }

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
    let mode = match mode.map(|s|s.as_str()) {
        None => SttMode::Normal,
        Some("--debug") => SttMode::Debug,
        Some("--token") => SttMode::TokenCheck,
        Some("--syntax") => SttMode::SyntaxCheck,
        Some(x) => panic!("No suck execution mode {x}"),
    };

    use SttMode as M;
    match mode {
        M::Normal => {
            execute_file(file_path).unwrap();
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
