#[macro_use]
extern crate lazy_static;

use crate::bus::Bus;
use crate::cpu::CPU;
pub mod bus;
pub mod cpu;
pub mod opcodes;
mod status_flags;

struct ROM {}
struct PPU {}
struct PAD {}
struct APU {}

pub struct NES {
    cpu: CPU,
    bus: Bus,
    rom: ROM,
    ppu: PPU,
    pad: PAD,
    apu: APU,
}
