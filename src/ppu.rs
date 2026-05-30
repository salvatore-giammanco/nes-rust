use crate::control_register::ControlRegister;
use crate::rom::Mirroring;

pub struct AddressRegister {
    value: (u8, u8),
    hi_ptr: bool,
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            value: (0, 0), // High byte first, lo byte second
            hi_ptr: true,
        }
    }

    pub fn set(&mut self, data: u16) {
        self.value.0 = (data >> 8) as u8;
        self.value.0 = (data & 0xFF) as u8;
    }

    pub fn get(&self) -> u16 {
        ((self.value.0 as u16) << 8) & self.value.1 as u16
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.value.0 = data;
        } else {
            self.value.1 = data;
        }

        // Mirror down addresses above 0x3FFF
        self.set(self.get() & 0x3FFF);

        self.hi_ptr = !self.hi_ptr;
    }

    pub fn increment(&mut self, inc: u8) {
        let current = self.get();
        let new = current.wrapping_add(inc as u16);

        // Set and mirror down addresses above 0x3FFF
        self.set(new & 0x3FFF);
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }
}
pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub mirroring: Mirroring,
    pub address: AddressRegister,
    pub control: ControlRegister,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            chr_rom,
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            mirroring,
            address: AddressRegister::new(),
            control: ControlRegister::new(),
        }
    }
    fn write_to_ppu_address(&mut self, value: u8) {
        self.address.update(value);
    }
}
