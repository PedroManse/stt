use crate::{
    RuntimeContext, RuntimeErrorKind, StckError, Value,
    runtime::{Hook, module::Module, sget, stack_pop},
};
use std::path::Path;

macro_rules! register {
    ($mod:expr, $name:ident as |$ctx:ident| $fn:block) => {
        fn $name($ctx: &mut RuntimeContext, _: &Path) -> Result<(), RuntimeErrorKind> $fn
        $mod.add_fn(format!("io${}", stringify!($name)), Hook::WithError($name));
    };
    ($mod:expr, $name:ident as |$ctx:ident, $path: ident| $fn:block) => {
        fn $name($ctx: &mut RuntimeContext, $path: &Path) -> Result<(), RuntimeErrorKind> $fn
        $mod.add_fn(format!("io${}", stringify!($name)), Hook::WithError($name));
    };
}

pub fn io_module() -> Result<Module, StckError> {
    let mut io_mod = Module::new_protected("#io".to_string())?;

    register!(io_mod, read_file as |ctx| {
        let path = stack_pop!((ctx.stack) -> str as "file path" for "io$read-file")?;
        let content = std::fs::read_to_string(path);
        let r = match content {
            Ok(o) => Ok(Value::from(o)),
            Err(e) => Err(Value::from(e.to_string())),
        };
        ctx.stack.push_this(r);
        Ok(())
    });

    register!(io_mod, write_file as |ctx| {
        let path = stack_pop!((ctx.stack) -> str as "file path" for "io$read-file")?;
        let content = stack_pop!((ctx.stack) -> str as "file content" for "io$read-file")?;
        println!("write {content} to {path}");
        Ok(())
    });

    Ok(io_mod)
}
