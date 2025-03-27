use stt::*;

//TODO execution mode
// : normal
// : debug
// : syntax

// TODO make (if) kw for common case of (if) {check} {if-code} Option<{else-code}>

// TODO * mode for FnArgs, allow access to entire stack

// TODO * mode for Ifs to non-exclusive execution

fn main() {
    let cont = include_str!("../examples/rust.stt");

    let mut tokenizer = token::Context::new(cont);
    let root_block = tokenizer.tokenize_block().unwrap();

    let mut parser = parse::Context::new(root_block);
    let code = parser.parse_block().unwrap();

    let mut executioner = execute::Context::new();
    for c in &code {
        executioner.execute(c);
    }
}
