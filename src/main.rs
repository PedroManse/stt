use stt::*;

fn main() {
    use Expr::*;

    let print_twice_code = vec![
        FnCall(FnName("to-print".to_string())),
        FnCall(FnName("print".to_string())),
        FnCall(FnName("to-print".to_string())),
        FnCall(FnName("print".to_string())),
        Immediate(Value::Str("var-name".to_string())),
        FnCall(FnName("get".to_string())),
        FnCall(FnName("print".to_string())),
    ];

    let code = vec![
        Immediate(Value::Str("Hello".to_string())),
        FnCall(FnName("print".to_string())),


        FnDef(
            true,
            FnArgs(vec!["to-print".to_string()]),
            FnName("print-twice".to_string()),
            Code(print_twice_code),
        ),

        Immediate(Value::Str("uwu".to_string())),
        Immediate(Value::Str("var-name".to_string())),
        FnCall(FnName("set".to_string())),

        Immediate(Value::Str("var-name".to_string())),
        FnCall(FnName("get".to_string())),
        FnCall(FnName("print-twice".to_string())),
    ];
    let mut ctx = execute::Context::new();
    for c in code {
        ctx.execute(c);
    }
    println!("{:?}", ctx.vars);
    println!("{:?}", ctx.stack.into_vec());
}
