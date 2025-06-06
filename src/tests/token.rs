use super::*;
macro_rules! mkt {
    ($from:literal .. $to:literal $cont:expr) => {
        Token {
            span: $from..$to,
            cont: $cont,
        }
    };
}

#[test]
fn read_tokens() {
    let text = "
(fn) [ a b c ] fn-name {
    a b c + +
}
1 2 3 fn-name
";
    let mut ctx = crate::token::Context::new(text);
    let block = ctx.tokenize_block();
    assert!(block.is_ok());
    let block = block.unwrap();
    use TokenCont as C;

    let expected: Vec<Token> = vec![
        mkt!(1..5(C::Keyword(RawKeyword::Fn(FnScope::Local)))),
        mkt!(6..15(C::FnArgs(vec!["a", "b", "c"].into_strings()))),
        mkt!(16..24(C::Ident("fn-name".to_string()))),
        mkt!(
            24..41(C::Block(vec![
                mkt!(29..32(C::Ident("a".to_string()))),
                mkt!(32..34(C::Ident("b".to_string()))),
                mkt!(34..36(C::Ident("c".to_string()))),
                mkt!(36..38(C::Ident("+".to_string()))),
                mkt!(38..40(C::Ident("+".to_string()))),
                mkt!(41..41(C::EndOfBlock)),
            ]))
        ),
        mkt!(42..44(C::Number(1))),
        mkt!(44..46(C::Number(2))),
        mkt!(46..48(C::Number(3))),
        mkt!(48..56(C::Ident("fn-name".to_string()))),
        mkt!(56..56(C::EndOfBlock)),
    ];

    for index in 0..block.len() {
        let got = block.get(index);
        let wanted = expected.get(index);
        test_eq!(got: got, expected: wanted);
        if let Some(t) = got {
            eprintln!("text: {}", text[t.span.clone()].to_string());
        }
    }

    test_eq!(got: block.len(), expected: expected.len());
}
