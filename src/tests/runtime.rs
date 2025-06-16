use super::*;
use crate::{Context, RustStckFn, StckErrorCase, Value, api};

fn execute_string(cont: &str, test_name: &str) -> Result<Context, StckErrorCase> {
    let tokens = api::get_tokens_str(cont, test_name)?;
    let code = api::parse_raw_tokens(tokens)?;
    let mut runtime = Context::new();
    runtime.execute_entire_code(&code)?;
    Ok(runtime)
}

#[test]
fn rust_hook() -> Result<(), StckErrorCase> {
    let tokens = api::get_tokens_str("\"7 3 -\n\" eval\n", "test rust hook")?;
    let code = api::parse_raw_tokens(tokens)?;
    let mut runtime = Context::new();
    let hook = RustStckFn::new("eval".to_string(), |ctx, source| {
        let st = ctx.stack.pop_this(Value::get_str).unwrap().unwrap();
        let tokens = api::get_tokens_str(&st, format!("Eval at {source:?}")).unwrap();
        let code = api::parse_raw_tokens(tokens).unwrap();
        ctx.execute_entire_code(&code).unwrap();
    });
    runtime.add_rust_hook(hook);
    runtime.execute_entire_code(&code)?;
    let stack = runtime.get_stack();
    let expected_stack = [Value::Num(4)];
    test_eq!(got: stack, expected: expected_stack);
    Ok(())
}

#[test]
fn closure_parent_args() -> Result<(), StckErrorCase> {
    let ctx = execute_string(
        "
(fn) [ i<num> ] [ <num> ] double { i 2 * }
(fn) [ first<fn> seccond<fn> ] [ joint<fn> ] join {
    [ v ]{ first seccond v @ @ }
}

(@double) (@double) join 2 @


[ a ] {
    [ b ] {
        [ _ ] {
            a b -
        }
    }
}

3 @ 4 @ '_' @
",
        "Test nested arguments",
    )?;
    let stack = ctx.get_stack();
    let expected_stack = [Value::Num(8), Value::Num(-1)];
    test_eq!(got: stack, expected: expected_stack);
    Ok(())
}
