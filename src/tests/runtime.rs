use super::*;
use crate::{
    api,
    error::{self, Error},
    internals::{RuntimeContext, RustStckFn, Value},
};

fn execute_string(cont: &str, test_name: &str) -> Result<RuntimeContext, Error> {
    let tokens = api::get_tokens_str(cont, test_name)?;
    let code = api::parse_raw_tokens(tokens)?;
    let mut runtime = RuntimeContext::new();
    runtime
        .execute_entire_code(&code)
        .map_err(error::RuntimeError::from)?;
    Ok(runtime)
}

#[test]
fn rust_hook() -> Result<(), Error> {
    let tokens = api::get_tokens_str("\"7 3 -\n\" eval\n", "test rust hook")?;
    let code = api::parse_raw_tokens(tokens)?;
    let mut runtime = RuntimeContext::new();
    let hook = RustStckFn::new("eval".to_string(), |ctx, source| {
        let st = ctx.stack.pop_this(Value::get_str).unwrap().unwrap();
        let tokens = api::get_tokens_str(&st, format!("Eval at {source:?}")).unwrap();
        let code = api::parse_raw_tokens(tokens).unwrap();
        ctx.execute_entire_code(&code).unwrap();
    });
    runtime.add_rust_hook(hook);
    runtime
        .execute_entire_code(&code)
        .map_err(error::RuntimeError::from)?;
    let stack = runtime.get_stack();
    let expected_stack = [Value::Num(4)];
    test_eq!(got: stack, expected: expected_stack);
    Ok(())
}

#[test]
fn closure_parent_args() -> Result<(), Error> {
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
