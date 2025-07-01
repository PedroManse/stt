mod parse;
mod runtime;
mod token;
mod typing;

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
