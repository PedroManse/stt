use super::*;
use crate::*;
use TypeTester as TT;
use std::collections::HashMap;
type TR = std::result::Result<(), TypeTester>;
const T_OK: TR = TR::Ok(());
const T_ERR: fn(&TT) -> TR = |d| TR::Err(d.clone());

#[test]
fn test_simple_types() -> Result<(), crate::error::RuntimeErrorKind> {
    let closure_sum = Value::Closure(Box::new(crate::Closure {
        code: vec![],
        request_args: ClosurePartialArgs::convert(
            vec![
                FnArgDef::new("a".to_string(), Some(TT::Num)),
                FnArgDef::new("b".to_string(), Some(TT::Num)),
            ],
            "test closure",
        )?,
        output_types: Some(TypedOutputs::new(vec![FnArgDef::new(
            String::new(),
            Some(TT::Num),
        )])),
    }));

    let values = [
        Value::Num(0),
        Value::Str(String::new()),
        Value::Array(vec![Value::Num(0), Value::Str(String::new())]),
        closure_sum,
        Value::Option(Some(Box::new(Value::Num(0)))),
        Value::Result(Box::new(Ok(Value::Num(0)))),
        Value::Map(HashMap::new()),
        Value::Char('a'),
        Value::Bool(false),
    ];
    let types = [
        TT::Num,
        TT::Str,
        TT::ArrayAny,
        TT::ClosureAny,
        TT::OptionAny,
        TT::ResultAny,
        TT::MapAny,
        TT::Char,
        TT::Bool,
    ];

    for (tt, vl) in types.iter().zip(values.iter()) {
        test_eq!(got: tt.check(vl), expected: T_OK);
    }

    for (tt_index, tt) in types.iter().enumerate() {
        for (v_index, v) in values.iter().enumerate() {
            if v_index != tt_index {
                test_eq!(got: tt.check(v), expected: T_ERR(tt));
            }
        }
    }
    Ok(())
}

#[test]
fn test_array_type() {
    let arr_of_num_type = TT::Array(Box::new(TT::Num));
    let arr_or_num = Value::Array(vec![Value::Num(3), Value::Num(0)]);
    let type_test = arr_of_num_type.check(&arr_or_num);

    test_eq!(got: type_test, expected: T_OK);
}

#[test]
fn test_closure_type() -> Result<(), crate::error::RuntimeErrorKind> {
    let closure_sum_type = TT::Closure(
        TypedFnPart::Typed(vec![TT::Num, TT::Num]),
        TypedFnPart::Typed(vec![TT::Num]),
    );
    let closure_sum = Value::Closure(Box::new(Closure {
        code: vec![],
        request_args: ClosurePartialArgs::convert(
            vec![
                FnArgDef::new("a".to_string(), Some(TT::Num)),
                FnArgDef::new("b".to_string(), Some(TT::Num)),
            ],
            "test closure",
        )?,
        output_types: Some(TypedOutputs::new(vec![FnArgDef::new(
            String::new(),
            Some(TT::Num),
        )])),
    }));
    let type_test = closure_sum_type.check(&closure_sum);
    test_eq!(got: type_test, expected: T_OK);
    Ok(())
}
