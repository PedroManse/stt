use crate::*;
mod runtime;
mod token;
mod typing;

trait StrVecIntoStringVec {
    fn into_strings(self) -> Vec<String>;
}

impl StrVecIntoStringVec for Vec<&str> {
    fn into_strings(self) -> Vec<String> {
        self.into_iter().map(String::from).collect()
    }
}

// like assert_eq but shows `got` and `expected`
macro_rules! test_eq {
    (got: $got:expr, expected: $expected:expr) => {{
        if $got != $expected {
            panic!(
                r"assertion failed: `got == expected`
     got: `{:?}`,
expected: `{:?}`",
                $got, $expected
            )
        }
    }};
}
use test_eq;
