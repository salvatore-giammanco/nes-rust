use crate::control_register::ControlRegister;
use crate::rom::Mirroring;
use sdl2::touch::touch_device;

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
    pub internal_data_buffer: u8,
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
            internal_data_buffer: 0,
        }
    }
    fn write_to_ppu_address(&mut self, value: u8) {
        self.address.update(value);
    }

    fn increment_vram_address(&mut self) {
        self.address
            .increment(self.control.get_vram_address_increment());
    }

    fn read_data(&mut self) -> u8 {
        let addr = self.address.get();
        self.increment_vram_address();

        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_address(addr) as usize];
                result
            }
            0x3000..=0x3EFF => panic!(
                "Address space 0x3000..0x3EFF is not expected to be used, requested = {:02X}",
                addr
            ),
            0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize],
            _ => panic!("Unexpected access to mirrored space {:04X}", addr),
        }
    }

    fn write_data(&mut self, data: u8) {
        let addr = self.address.get();
        self.increment_vram_address();

        match addr {
            0..=0x1FFF => {
                panic!("Attempt to write on CHR ROM at address {:04}", addr);
            }
            0x2000..=0x2FFF => {
                self.vram[self.mirror_vram_address(addr) as usize] = data;
            }
            0x3000..=0x3EFF => panic!(
                "Address space 0x3000..0x3EFF is not expected to be used, requested = {:02X}",
                addr
            ),
            0x3F00..=0x3FFF => self.palette_table[(addr - 0x3F00) as usize] = data,
            _ => panic!("Unexpected access to mirrored space {:04X}", addr),
        }
    }

    fn mirror_vram_address(&self, addr: u16) -> u16 {
        // Horizontal:
        //   [ A ] [ a ]
        //   [ B ] [ b ]

        // Vertical:
        //   [ A ] [ B ]
        //   [ a ] [ b ]

        let mirrored_vram_addr = addr & 0xBFFF; // Mirror down 0x3000-0x3EFF to 0x2000 - 0x2EFF
        let vram_index = mirrored_vram_addr - 0x2000;
        let nametable_index = vram_index / 0x400;

        match &self.mirroring {
            Mirroring::Horizontal => match nametable_index {
                1 | 2 => vram_index - 0x400,
                3 => vram_index - 0x800,
                _ => vram_index,
            },
            Mirroring::Vertical => match nametable_index {
                2 | 3 => vram_index - 0x800,
                _ => vram_index,
            },
            _ => vram_index,
        }
    }
}
