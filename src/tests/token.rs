use super::*;
use crate::*;
macro_rules! mkt {
    ($from:literal .. $to:literal $cont:expr) => {
        Token {
            span: LineRange::from_points($from, $to),
            cont: $cont,
        }
    };
}

#[test]
fn read_tokens() -> Result<(), error::Error> {
    use TokenCont as C;
    let text = "(fn) [ typed<num> in_puts a ] [ sum<num> ] fn-name {
    inputs typed a + +
}
1 2 3 fn-name
";
    let ctx = crate::token::Context::new(text);
    let block = ctx.tokenize("read_tokens test".into())?;

    let expected = [
        mkt!(1..1(C::Keyword(RawKeyword::Fn(FnScope::Local)))),
        mkt!(
            1..1(C::FnArgs(vec![
                FnArgDef {
                    name: "typed".to_string(),
                    type_check: Some(TypeTester::Num)
                },
                FnArgDef {
                    name: "in_puts".to_string(),
                    type_check: None
                },
                FnArgDef {
                    name: "a".to_string(),
                    type_check: None
                }
            ]))
        ),
        mkt!(
            1..1(C::FnArgs(vec![FnArgDef {
                name: "sum".to_string(),
                type_check: Some(TypeTester::Num)
            },]))
        ),
        mkt!(1..1(C::Ident("fn-name".to_string()))),
        mkt!(
            1..3(C::Block(vec![
                mkt!(2..2(C::Ident("inputs".to_string()))),
                mkt!(2..2(C::Ident("typed".to_string()))),
                mkt!(2..2(C::Ident("a".to_string()))),
                mkt!(2..2(C::Ident("+".to_string()))),
                mkt!(2..2(C::Ident("+".to_string()))),
                mkt!(3..3(C::EndOfBlock)),
            ]))
        ),
        mkt!(4..4(C::Number(1))),
        mkt!(4..4(C::Number(2))),
        mkt!(4..4(C::Number(3))),
        mkt!(4..4(C::Ident("fn-name".to_string()))),
        mkt!(4..4(C::EndOfBlock)),
    ];

    for index in 0..block.token_count() {
        let got = block.get(index);
        let wanted = expected.get(index);
        test_eq!(got: got, expected: wanted);
    }

    test_eq!(got: block.tokens.len(), expected: expected.len());
    Ok(())
}
