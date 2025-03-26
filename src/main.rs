use stt::*;

fn main() {
    use Expr::*;
    let code = vec![
        Immediate(Value::Str("Hello".to_string())),
        FnCall(FnName("print".to_string())),
    ];
    let mut ctx = execute::Context::new();
    for c in code {
        ctx.execute(c);
    }
}
