mod io;
pub mod oficial {
    pub use super::io::io_module;
}

use crate::{FnName, StckError};
use std::collections::HashMap;

use super::Hook;

#[derive(Clone)]
pub struct Module {
    pub(crate) name: String,
    pub(crate) funcs: HashMap<FnName, Hook>,
}

impl Module {
    pub fn new(name: String) -> Result<Module, StckError> {
        if name.starts_with('#') {
            Err(StckError::UserModuleWithBang(name))
        } else {
            Ok(Module {
                name,
                funcs: HashMap::new(),
            })
        }
    }
    fn new_protected(name: String) -> Result<Module, StckError> {
        if name.starts_with('#') {
            Ok(Module {
                name,
                funcs: HashMap::new(),
            })
        } else {
            Err(StckError::ProtectedModuleWithoutBang(name))
        }
    }
    pub fn add_fn(&mut self, name: impl Into<String>, fnc: Hook) -> Option<Hook> {
        self.funcs.insert(name.into(), fnc)
    }
}
