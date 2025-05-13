use crate::*;
mod builtins;
mod stack;
use stack::*;
mod exec;
mod modes;
pub use modes::*;

pub enum ControlFlow {
    Continue,
    Break,
    Return,
}
