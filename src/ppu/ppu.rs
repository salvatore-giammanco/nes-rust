use super::control_register::ControlRegister;
use crate::ppu::mask_register::MaskRegister;
use crate::ppu::status_register::PPUStatusRegister;
use crate::ppu::BitFlags;
use crate::rom::Mirroring;

pub struct AddressRegister {
    value: (u8, u8),
    hi_ptr: bool,
}

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub mirroring: Mirroring,
    pub internal_data_buffer: u8,
    // write_toggle serves both address and scroll registers (shared bit) - also called latch
    write_toggle: bool,

    // Registers

    // PPUCTRL
    pub control: ControlRegister,
    // PPUMASK
    pub mask: MaskRegister,
    // PPUSTATUS
    pub status: PPUStatusRegister,
    // OAMADDR
    oam_address: u8,
    // OAMDATA
    pub oam_data: [u8; 256],
    // PPUSCROLL
    scroll: (u8, u8),
    // PPUADDR
    address: (u8, u8),
    // PPUDATA
    pub vram: [u8; 2048],
    // OAMDMA
    // ...
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            chr_rom,
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            mirroring,
            internal_data_buffer: 0,
            address: (0, 0),
            control: ControlRegister::new(),
            mask: MaskRegister::new(),
            status: PPUStatusRegister::new(),
            oam_address: 0,
            scroll: (0, 0),
            write_toggle: true,
        }
    }

    pub fn set_address_register(&mut self, data: u16) {
        self.address.0 = (data >> 8) as u8;
        self.address.0 = (data & 0xFF) as u8;
    }

    pub fn get_address_register(&self) -> u16 {
        ((self.address.0 as u16) << 8) & self.address.1 as u16
    }

    pub fn update_address_register(&mut self, data: u8) {
        if self.write_toggle {
            self.address.0 = data;
        } else {
            self.address.1 = data;
        }

        // Mirror down addresses above 0x3FFF
        self.set_address_register(self.get_address_register() & 0x3FFF);

        self.write_toggle = !self.write_toggle;
    }

    pub fn increment_address_register(&mut self, inc: u8) {
        let current = self.get_address_register();
        let new = current.wrapping_add(inc as u16);

        // Set and mirror down addresses above 0x3FFF
        self.set_address_register(new & 0x3FFF);
    }

    pub fn set_scroll_register(&mut self, data: u16) {
        self.scroll.0 = (data >> 8) as u8;
        self.scroll.0 = (data & 0xFF) as u8;
    }

    pub fn get_scroll_register(&self) -> u16 {
        ((self.scroll.0 as u16) << 8) & self.scroll.1 as u16
    }

    pub fn update_scroll_register(&mut self, data: u8) {
        if self.write_toggle {
            self.scroll.0 = data;
        } else {
            self.scroll.1 = data;
        }
        self.write_toggle = !self.write_toggle;
    }

    pub fn increment_scroll_register(&mut self, inc: u8) {
        let current = self.get_scroll_register();
        let new = current.wrapping_add(inc as u16);
    }

    fn increment_vram_address(&mut self) {
        self.increment_address_register(self.control.get_vram_address_increment());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.get_address_register();
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

    pub fn write_data(&mut self, data: u8) {
        let addr = self.get_address_register();
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

    pub fn write_to_oam_address(&mut self, value: u8) {
        // TODO: Validate address?
        self.oam_address = value;
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_address as usize]
    }

    pub fn write_oam_data(&mut self, data: u8) {
        self.oam_data[self.oam_address as usize] = data;
        self.oam_address = self.oam_address.wrapping_add(1);
    }
}
