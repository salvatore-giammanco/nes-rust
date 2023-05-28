#[macro_use]
extern crate lazy_static;

use crate::cpu::CPU;
pub mod cpu;
pub mod opcodes;

struct BUS {}
struct ROM {}
struct PPU {}
struct PAD {}
struct APU {}

pub struct NES {
    cpu: CPU,
    bus: BUS,
    rom: ROM,
    ppu: PPU,
    pad: PAD,
    apu: APU,
}
