use std::path::PathBuf;

use crate::KeywordKind;

use super::*;

#[test]
fn parse_tokens() -> Result<(), crate::StckErrorCase> {
    use crate::{
        Expr,
        ExprCont::{FnCall, Immediate, Keyword},
    };
    let text_name = "parse_tokens test";
    let text = "
(fn) [ typed<num> in_puts ] [ sum<num> ] fn-name {
    inputs typed 0 - -
}";
    let token_block = crate::api::get_tokens_str(text, text_name)?;
    let expr = crate::api::parse_raw_tokens(token_block)?;
    test_eq!(got: expr.source, expected: PathBuf::from(text_name));
    test_eq!(got: expr.expr_count(), expected: 1);
    let expr_expected: Vec<crate::Expr> = vec![Expr {
        span: 0..76,
        cont: Keyword(KeywordKind::FnDef {
            name: "fn-name".to_string(),
            scope: crate::FnScope::Local,
            code: vec![
                Expr {
                    span: 50..63,
                    cont: FnCall("inputs".to_string()),
                },
                Expr {
                    span: 63..69,
                    cont: FnCall("typed".to_string()),
                },
                Expr {
                    span: 69..71,
                    cont: Immediate(crate::Value::Num(0)),
                },
                Expr {
                    span: 71..73,
                    cont: FnCall("-".to_string()),
                },
                Expr {
                    span: 73..75,
                    cont: FnCall("-".to_string()),
                },
            ],
            args: crate::FnArgs::Args(vec![
                crate::FnArgDef {
                    name: "typed".to_string(),
                    type_check: Some(crate::TypeTester::Num),
                },
                crate::FnArgDef {
                    name: "in_puts".to_string(),
                    type_check: None,
                },
            ]),
            out_args: Some(vec![crate::FnArgDef {
                name: "sum".to_string(),
                type_check: Some(crate::TypeTester::Num),
            }]),
        }),
    }];
    test_eq!(got: expr.exprs, expected: expr_expected);

    Ok(())
}
