use stt::*;

use self::token::Context;

macro_rules! e {
    (call $fn_name:expr) => {
        FnCall(FnName($fn_name.to_owned()))
    };
    (imm str $v:expr) => {
        Immediate(Value::Str($v.to_owned()))
    };
    (imm bool $v:expr) => {
        Immediate(Value::Bool($v))
    };
    (imm num $v:expr) => {
        Immediate(Value::Num($v))
    };
}

fn _main() {
    use Expr::*;

    let print_twice_code = vec![
        e!(call "to-print"),
        e!(call "print"),
        e!(call "to-print"),
        e!(call "print"),
        e!(imm str "var-name"),
        e!(call "get"),
        e!(call "print"),
    ];

    let code = vec![
        e!(imm str "Hello"),
        e!(call "print"),
    
        Keyword(KeywordKind::FnDef{
            scope: FnScope::Global,
            args: FnArgs(vec!["to-print".to_string()]),
            name: FnName("print-twice".to_string()),
            code: Code(print_twice_code),
        }),
    
        e!(imm str "uwu"),
        e!(imm str "var-name"),
        e!(call "set"),
    
        e!(imm str "var-name"),
        e!(call "get"),
        e!(call "print-twice"),
    ];

    //let code_true = vec![e!(imm str "false path"), e!(call "print")];

    //let code_false = vec![e!(imm str "false path"), e!(call "print")];

    //let code_check = vec![ e!(imm bool true) ];

    //let code = vec![Keyword(KeywordKind::If {
    //    if_branch: CondBranch {
    //        check: Code(code_check),
    //        code: Code(code_true),
    //    },
    //    else_code: Code(code_false),
    //})];

    let mut ctx = execute::Context::new();
    for c in &code {
        ctx.execute(c);
    }
    println!("{:?}", ctx.vars);
    println!("{:?}", ctx.stack.into_vec());
}

fn main() {
    let cont = include_str!("../examples/rust.stt");
    let mut tokenizer = Context::new(cont);
    let root_block = tokenizer.tokenize_block();
    println!("{root_block:?}");
}

