use stt::*;

// TODO execution mode
// : normal
// : debug
// : syntax
// : preproc

// TODO error reporting

// TODO make (if) kw for common case of (if) {check} {if-code} Option<{else-code}>

// TODO * mode for Ifs to non-exclusive execution

// TODO (pragma <command>) // once -> only execute file once // store in execution context

pub enum SttMode {
    Normal,
    Debug,
    Syntax,
}

fn main() {
    let mode = SttMode::Normal;
    let file_path = "examples/rust.stt";
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
