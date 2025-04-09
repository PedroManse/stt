use stt::*;

// TODO error reporting

// TODO execution mode
// : normal
// : debug --debug
// : syntax --raw-syncheck
// : preproc --syncheck


// TODO make (if) kw for common case of (if) {check} {if-code} {true} {else-code}

// TODO * mode for Ifs to non-exclusive execution

// TODO (pragma <command>) // once -> only execute file once // store in execution context

pub enum SttMode {
    Normal,
    Debug,
    Syntax,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let file_path = args.next().unwrap();
    let mode = SttMode::Normal;
    use SttMode as M;
    match mode {
        M::Normal => {
            execute_file(file_path).unwrap();
        }
        M::Debug => {
            todo!()
        }
        M::Syntax => {
            get_project_code(file_path).unwrap();
        }
    }
}
