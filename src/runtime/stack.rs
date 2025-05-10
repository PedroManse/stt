macro_rules! sget {
    (num) => {
        (Value::get_num, Value::get_ref_num, "Number")
    };
    (str) => {
        (Value::get_str, Value::get_ref_str, "String")
    };
    (bool) => {
        (Value::get_bool, Value::get_ref_bool, "Boolean")
    };
    (arr) => {
        (Value::get_arr, Value::get_ref_arr, "Array")
    };
    (map) => {
        (Value::get_map, Value::get_ref_map, "Map")
    };
    (result) => {
        (Value::get_result, Value::get_ref_result, "Result")
    };
    (option) => {
        (Value::get_option, Value::get_ref_option, "Option")
    };
}
pub(crate) use sget;

macro_rules! stack_pop {
    (($stack:expr) -> $type:ident as $this_arg:literal for $fn_name:expr) => {
        $stack
            .pop_this(sget!($type).0)
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: format!( "[{}: {}]", $this_arg, sget!($type).2 ),
                this_arg: $this_arg,
            })
            .map(|got_v|{
                got_v.map_err(|got|{
                    SttError::WrongTypeForBuiltin {
                        for_fn: $fn_name.to_owned(),
                        args: stringify!( [ $this_arg: $type ] ),
                        this_arg: $this_arg,
                        got,
                        expected: sget!($type).2
                    }
                })
            })
    };
    (($stack:expr) -> $type:ident? as $this_arg:literal for $fn_name:expr) => {
        $stack
            .pop_this(sget!($type).0)
            .map(|got_v|{
                got_v.map_err(|got|{
                    SttError::WrongTypeForBuiltin {
                        for_fn: $fn_name.to_owned(),
                        args: stringify!( [ $this_arg: $ty ] ),
                        this_arg: $this_arg,
                        got,
                        expected: sget!($type).2
                    }
                })
            }).transpose()
    };
    (($stack:expr) -> * as $this_arg:literal for $fn_name:expr) => {
        $stack
            .pop()
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: format!( "[{}]", $this_arg ),
                this_arg: $this_arg,
            })
    };
    (($stack:expr) -> &$type:ident as $this_arg:literal for $fn_name:expr) => {
        $stack
            .peek_this(sget!($type).1)
            .ok_or(SttError::MissingValueForBuiltin{
                for_fn: $fn_name.to_owned(),
                args: format!( "[{}: {}]", $this_arg, sget!($type).2 ),
                this_arg: $this_arg,
            })
            .map(|got_v|{
                got_v.map_err(|got|{
                    SttError::WrongTypeForBuiltin {
                        for_fn: $fn_name.to_owned(),
                        args: stringify!( [ $this_arg: $ty ] ),
                        this_arg: $this_arg,
                        got: got.clone(),
                        expected: sget!($type).2
                    }
                })
            })
    };
}
pub (crate) use stack_pop;

