use crate::cpu::cpu::CPU;
pub mod cpu;

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
    apu: APU
}

