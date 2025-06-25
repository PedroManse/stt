use std::path::PathBuf;

use super::*;
use crate::KeywordKind;
use crate::LineRange;

#[test]
fn parse_tokens() -> Result<(), crate::error::Error> {
    use crate::{
        Expr,
        ExprCont::{FnCall, Immediate, Keyword},
    };
    let text_name = "parse_tokens test";
    let text = "
(fn) [ typed<num> in_puts ] [ sum<num> ] fn-name {
    inputs typed 0 - -
}";
    let token_block =
        crate::api::get_tokens_str(text, text_name, &mut crate::cache::Isolated::new())?;
    let expr = crate::api::parse_raw_tokens(token_block)?;
    test_eq!(got: expr.source, expected: PathBuf::from(text_name));
    test_eq!(got: expr.expr_count(), expected: 1);
    let expr_expected: Vec<crate::Expr> = vec![Expr {
        span: LineRange::from_points(0, 4),
        cont: Keyword(KeywordKind::FnDef {
            name: "fn-name".to_string(),
            scope: crate::FnScope::Local,
            code: vec![
                Expr {
                    span: LineRange::from_points(2, 2),
                    cont: FnCall("inputs".to_string()),
                },
                Expr {
                    span: LineRange::from_points(2, 2),
                    cont: FnCall("typed".to_string()),
                },
                Expr {
                    span: LineRange::from_points(2, 2),
                    cont: Immediate(crate::Value::Num(0)),
                },
                Expr {
                    span: LineRange::from_points(2, 2),
                    cont: FnCall("-".to_string()),
                },
                Expr {
                    span: LineRange::from_points(2, 2),
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
