use super::*;

#[test]
fn rust_hook() {
    let tokens = api::get_tokens_str("\"7 3 -\n\" eval\n", "Raw string").unwrap();
    let code = api::parse_raw_tokens(tokens).unwrap();
    let mut runtime = Context::new();
    let hook = RustSttFn::new("eval".to_string(), |ctx, source| {
        let st = ctx.stack.pop_this(Value::get_str).unwrap().unwrap();
        let tokens = api::get_tokens_str(&st, format!("Eval at {source:?}")).unwrap();
        let code = api::parse_raw_tokens(tokens).unwrap();
        ctx.execute_entire_code(&code).unwrap();
    });
    runtime.add_rust_hook(hook);
    runtime.execute_entire_code(&code).unwrap();
    let stack = runtime.get_stack();
    let expected_stack = [Value::Num(4)];
    test_eq!(got: stack, expected: expected_stack);
}
