use crate::cpu::Mem;
use crate::ppu::BitFlags;
use crate::ppu::PPU;
use crate::rom::ROM;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const ROM_START_IN_MEMORY: u16 = 0x8000;

pub struct Bus {
    cpu_vram: [u8; 0xFFFF],
    rom: Option<ROM>,
    ppu: PPU,
}

impl Bus {
    pub fn new(rom: ROM) -> Self {
        let ppu = PPU::new(rom.chr_rom.clone(), rom.screen_mirroring.clone());
        Self {
            cpu_vram: [0; 0xFFFF],
            rom: Some(rom),
            ppu,
        }
    }

    pub fn load_rom(&mut self, rom: ROM) {
        self.rom = Some(rom);
    }
}

impl Mem for Bus {
    fn read_mem(&mut self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
            }
            0x2002 => self.ppu.status.status,
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0x2007;
                println!("PPU register read at {:#X}", addr);
                todo!("PPU is not supported yet - read")
            }
            0x8000..=0xFFFF => {
                let rom = self.rom.as_ref().unwrap();
                let mut addr = addr - 0x8000;

                // Mirroring for 16KB PRG ROM
                if rom.prg_rom.len() == 0x4000 && addr >= 0x4000 {
                    addr = addr % 0x4000;
                }
                rom.prg_rom[addr as usize]
            }
            _ => {
                println!("Ignoring mem access at {:#X}", addr);
                0
            }
        }
    }

    fn write_mem(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 => self.ppu.control.set_from_byte(data),
            0x2001 => self.ppu.control.set_from_byte(data),
            0x2002 => panic!("Attempt to write on read only PPU status register ($2002)"),
            0x2003 => self.ppu.write_to_oam_address(data),
            0x2004 => self.ppu.write_oam_data(data),
            0x2006 => self.ppu.write_to_ppu_address(data),
            0x2007 => self.ppu.write_data(data),
            0x2008..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0x2007;
                println!("PPU register write at {:#X}", addr);
                todo!("PPU is not supported yet - write")
            }
            ROM_START_IN_MEMORY..=0xFFFF => {
                // TODO: Add unsafe mode to explicitly allow writing to ROM
                // panic!("Write to ROM at {:#X}: {:#X}", addr, data);
                self.rom.as_mut().unwrap().prg_rom[(addr - ROM_START_IN_MEMORY) as usize] = data;
            }
            _ => {
                println!("Ignoring mem write-access at {:#X}: {:#X}", addr, data);
            }
        }
    }
}
