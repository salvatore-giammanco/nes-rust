#[macro_use]
extern crate lazy_static;

pub mod bus;
pub mod cpu;
pub mod opcodes;
pub mod ppu;
pub mod rom;
mod status_flags;

mod control_register;
