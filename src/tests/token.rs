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
    let text = "
(fn) [ a b c ] fn-name {
    a b c + +
}
1 2 3 fn-name
";
    let ctx = crate::token::Context::new(text);
    let block = ctx.tokenize("read_tokens test".into())?;

    let expected = [
        mkt!(1..1(C::Keyword(RawKeyword::Fn(FnScope::Local)))),
        mkt!(
            1..1(C::FnArgs(
                ["a", "b", "c"]
                    .into_iter()
                    .map(String::from)
                    .map(|s| FnArgDef::new(s, None))
                    .collect()
            ))
        ),
        mkt!(1..1(C::Ident("fn-name".to_string()))),
        mkt!(
            2..2(C::Block(vec![
                mkt!(2..2(C::Ident("a".to_string()))),
                mkt!(2..2(C::Ident("b".to_string()))),
                mkt!(2..2(C::Ident("c".to_string()))),
                mkt!(2..2(C::Ident("+".to_string()))),
                mkt!(2..2(C::Ident("+".to_string()))),
                mkt!(2..2(C::EndOfBlock)),
            ]))
        ),
        mkt!(3..3(C::Number(1))),
        mkt!(3..3(C::Number(2))),
        mkt!(3..3(C::Number(3))),
        mkt!(3..3(C::Ident("fn-name".to_string()))),
        mkt!(3..3(C::EndOfBlock)),
    ];

    for index in 0..block.token_count() {
        let got = block.get(index);
        let wanted = expected.get(index);
        test_eq!(got: got, expected: wanted);
    }

    test_eq!(got: block.tokens.len(), expected: expected.len());
    Ok(())
}
