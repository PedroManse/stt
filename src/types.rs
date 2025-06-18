use super::*;
use std::str::FromStr;

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum TypedFnPart {
    Typed(Vec<TypeTester>),
    Any,
}

#[derive(PartialEq)]
pub enum TypedFnPartEq {
    Typed(Vec<TypeTesterEq>),
    Any,
}

impl TypedFnPart {
    #[must_use]
    fn as_eq(&self) -> TypedFnPartEq {
        match self {
            Self::Any => TypedFnPartEq::Any,
            Self::Typed(ts) => TypedFnPartEq::Typed(ts.iter().map(TypeTester::as_eq).collect()),
        }
    }
}

pub enum TypeTesterEq {
    Any,
    Char,
    Str,
    Num,
    Bool,
    ArrayAny,
    MapAny,
    ResultAny,
    OptionAny,
    ClosureAny,
    Array(Box<Self>),
    Map(Box<Self>),
    Result(Box<(Self, Self)>),
    Option(Box<Self>),
    Closure(Box<(TypedFnPartEq, TypedFnPartEq)>),
}

impl PartialEq for TypeTesterEq {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, _) => true,
            (Self::Char, Self::Char) => true,
            (Self::Str, Self::Str) => true,
            (Self::Num, Self::Num) => true,
            (Self::Bool, Self::Bool) => true,
            (Self::ArrayAny, Self::ArrayAny) => true,
            (Self::MapAny, Self::MapAny) => true,
            (Self::ResultAny, Self::ResultAny) => true,
            (Self::OptionAny, Self::OptionAny) => true,
            (Self::ClosureAny, Self::ClosureAny) => true,
            (Self::Array(t), Self::Array(to)) => t == to,
            (Self::Map(t), Self::Map(to)) => t == to,
            (Self::Result(tt), Self::Result(tto)) => tt == tto,
            (Self::Option(t), Self::Option(to)) => t == to,
            (Self::Closure(tt), Self::Closure(tto)) => tt == tto,
            (Self::ArrayAny, Self::Array(_)) => true,
            (Self::MapAny, Self::Map(_)) => true,
            (Self::ResultAny, Self::Result(_)) => true,
            (Self::OptionAny, Self::Option(_)) => true,
            (Self::ClosureAny, Self::Closure(_)) => true,
            (_, _) => false,
        }
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone)]
pub enum TypeTester {
    Any,
    Char,
    Str,
    Num,
    Bool,
    ArrayAny,
    MapAny,
    ResultAny,
    OptionAny,
    ClosureAny,
    Array(Box<TypeTester>),
    Map(Box<TypeTester>),
    Result(Box<(TypeTester, TypeTester)>),
    Option(Box<TypeTester>),
    Closure(TypedFnPart, TypedFnPart),
}

impl FromStr for TypeTester {
    type Err = StckError;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "?" => Self::Any,
            "char" => Self::Char,
            "string" | "str" => Self::Str,
            "num" => Self::Num,
            "bool" => Self::Bool,
            "list" | "array" => Self::ArrayAny,
            "map" => Self::MapAny,
            "result" => Self::ResultAny,
            "option" => Self::OptionAny,
            "fn" | "closure" => Self::ClosureAny,
            otherwise => {
                return None
                    .or(try_parse_fn(otherwise))
                    .or(try_parse_result(otherwise))
                    .or(try_parse_simple(otherwise))
                    .ok_or(StckError::UnknownType(s.to_string()));
            }
        })
    }
}

fn try_parse_simple(cont: &str) -> Option<TypeTester> {
    let cont = cont.strip_suffix('>')?;
    let (t, cont) = cont.split_once('<')?;
    let t_internal: TypeTester = cont.parse().ok()?;
    let make_t = match t {
        "option" => TypeTester::Option,
        "map" => TypeTester::Map,
        "array" => TypeTester::Array,
        _ => return None,
    };
    Some(make_t(Box::new(t_internal)))
}

// result<ok><err>
fn try_parse_result(cont: &str) -> Option<TypeTester> {
    let cont = cont.strip_prefix("result<")?;
    let cont = cont.strip_suffix('>')?;
    let (l, r) = cont.split_once("><")?;
    Some(TypeTester::Result(Box::new((
        l.parse().ok()?,
        r.parse().ok()?,
    ))))
}

fn parse_type_list(cont: &str) -> Result<TypedFnPart> {
    Ok(match cont {
        "*" => TypedFnPart::Any,
        cont => {
            let types = cont
                .split_whitespace()
                .map(TypeTester::from_str)
                .collect::<Result<Vec<_>>>()?;
            TypedFnPart::Typed(types)
        }
    })
}

fn try_parse_fn(txt: &str) -> Option<TypeTester> {
    txt.strip_prefix("fn<")
        .and_then(|tx| tx.strip_suffix('>'))
        .filter(|tx| tx.contains('>'))
        .and_then(|tx| {
            let (ins, outs) = tx.split_once('>')?;
            let outs = outs.strip_prefix('<')?;
            let Ok(ins) = parse_type_list(ins) else {
                return None;
            };
            let Ok(outs) = parse_type_list(outs) else {
                return None;
            };
            Some(TypeTester::Closure(ins, outs))
        })
}

impl TypeTester {
    #[must_use]
    pub fn as_eq(&self) -> TypeTesterEq {
        match self {
            Self::Any => TypeTesterEq::Any,
            Self::Char => TypeTesterEq::Char,
            Self::Str => TypeTesterEq::Str,
            Self::Num => TypeTesterEq::Num,
            Self::Bool => TypeTesterEq::Bool,
            Self::ArrayAny => TypeTesterEq::ArrayAny,
            Self::MapAny => TypeTesterEq::MapAny,
            Self::ResultAny => TypeTesterEq::ResultAny,
            Self::OptionAny => TypeTesterEq::OptionAny,
            Self::ClosureAny => TypeTesterEq::ClosureAny,
            Self::Array(a) => TypeTesterEq::Array(Box::new(a.as_eq())),
            Self::Map(a) => TypeTesterEq::Map(Box::new(a.as_eq())),
            Self::Result(a) => TypeTesterEq::Result(Box::new((a.0.as_eq(), a.1.as_eq()))),
            Self::Option(a) => TypeTesterEq::Option(Box::new(a.as_eq())),
            Self::Closure(a, b) => TypeTesterEq::Closure(Box::new((a.as_eq(), b.as_eq()))),
        }
    }
    #[must_use]
    pub fn check_type(&self, v: &TypeTester) -> bool {
        self.as_eq() == v.as_eq()
    }
    pub fn check(&self, v: &Value) -> OResult<(), TypeTester> {
        match (self, v) {
            (Self::Any, _) => Ok(()),
            (Self::Char, Value::Char(_)) => Ok(()),
            (Self::Str, Value::Str(_)) => Ok(()),
            (Self::Num, Value::Num(_)) => Ok(()),
            (Self::Bool, Value::Bool(_)) => Ok(()),
            (Self::ArrayAny, Value::Array(_)) => Ok(()),
            (Self::MapAny, Value::Map(_)) => Ok(()),
            (Self::ResultAny, Value::Result(_)) => Ok(()),
            (Self::OptionAny, Value::Option(_)) => Ok(()),
            (Self::ClosureAny, Value::Closure(_)) => Ok(()),
            (Self::Array(tt), Value::Array(n)) => {
                n.iter()
                    .map(|v| tt.check(v))
                    .collect::<OResult<Vec<_>, _>>()?;
                Ok(())
            }
            (Self::Map(tt_value), Value::Map(m)) => {
                for value in m.values() {
                    tt_value.check(value)?;
                }
                Ok(())
            }
            (Self::Result(tt), Value::Result(v)) => {
                let (tt_ok, tt_err) = tt.as_ref();
                match v.as_ref() {
                    Ok(v_ok) => tt_ok.check(v_ok),
                    Err(v_err) => tt_err.check(v_err),
                }
            }
            (Self::Option(_), Value::Option(None)) => Ok(()),
            (Self::Option(tt), Value::Option(Some(v))) => tt.check(v),
            (Self::Closure(ttinput, ttoutput), Value::Closure(cl)) => {
                if let TypedFnPart::Typed(ttinput) = ttinput {
                    if cl.request_args.next.len() != ttinput.len() {
                        return Err(self.clone());
                    }
                    let outs = cl
                        .request_args
                        .next
                        .iter()
                        .map(|arg_def| &arg_def.type_check)
                        .zip(ttinput);
                    for (cl_req, tt_req) in outs {
                        // part to test VTC
                        let ok = cl_req.as_ref().is_none_or(|c| tt_req.check_type(c));
                        if !ok {
                            return Err(tt_req.clone());
                        }
                    }
                }
                if let TypedFnPart::Typed(ttoutput) = ttoutput {
                    // part to test VTC
                    let Some(outputs) = cl.output_types.as_ref() else {
                        return Ok(());
                    };
                    if outputs.len() != ttoutput.len() {
                        return Err(self.clone());
                    }
                    for (cl_in, tt_in) in outputs.iter().zip(ttoutput) {
                        let ok = cl_in.as_ref().is_none_or(|c| tt_in.check_type(c));
                        if !ok {
                            return Err(tt_in.clone());
                        }
                    }
                }
                Ok(())
            }
            (t, _) => Err(t.clone()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypedOutputs {
    pub(crate) outputs: Vec<Option<TypeTester>>,
}

pub enum TypedOutputError {
    TypeError(TypeTester, Value),
    OutputCountError { expected: usize, got: usize },
}

impl TypedOutputs {
    #[must_use]
    pub fn new(v: Vec<FnArgDef>) -> Self {
        Self {
            outputs: v.into_iter().map(|a| a.type_check).collect(),
        }
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Option<TypeTester>> {
        self.outputs.iter()
    }
    pub(crate) fn len(&self) -> usize {
        self.outputs.len()
    }
    pub fn check(&self, values: &[Value]) -> OResult<(), TypedOutputError> {
        if self.len() != values.len() {
            return Err(TypedOutputError::OutputCountError {
                expected: self.len(),
                got: values.len(),
            });
        }
        for (v, maybe_tt) in values.iter().zip(self.iter()) {
            if let Some(Err(t)) = maybe_tt.as_ref().map(|tt| tt.check(v)) {
                return Err(TypedOutputError::TypeError(t, v.clone()));
            }
        }
        Ok(())
    }
}

impl From<Vec<FnArgDef>> for TypedOutputs {
    fn from(value: Vec<FnArgDef>) -> Self {
        TypedOutputs {
            outputs: value.into_iter().map(|v| v.type_check).collect(),
        }
    }
}

impl From<Value> for TypeTester {
    fn from(value: Value) -> Self {
        match value {
            Value::Char(_) => Self::Char,
            Value::Str(_) => Self::Str,
            Value::Num(_) => Self::Num,
            Value::Bool(_) => Self::Bool,
            Value::Closure(cl) => {
                let ipts: Vec<_> = cl
                    .request_args
                    .next
                    .into_iter()
                    .map(|v| v.type_check.unwrap_or(TypeTester::Any))
                    .collect();
                let out = if let Some(out) = cl.output_types {
                    TypedFnPart::Typed(
                        out.outputs
                            .into_iter()
                            .map(|v| v.unwrap_or(TypeTester::Any))
                            .collect::<Vec<_>>(),
                    )
                } else {
                    TypedFnPart::Any
                };
                TypeTester::Closure(TypedFnPart::Typed(ipts), out)
            }
            Value::Map(_) => todo!("map"),
            Value::Array(_) => todo!("array"),
            Value::Result(_) => todo!("result"),
            Value::Option(a) => a
                .map(|tt| TypeTester::from(*tt))
                .map_or(TypeTester::OptionAny, |tt| TypeTester::Option(Box::new(tt))),
        }
    }
}
