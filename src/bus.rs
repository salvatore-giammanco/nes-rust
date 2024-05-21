use crate::cpu::Mem;
use crate::rom::ROM;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const ROM_START_IN_MEMORY: u16 = 0x8000;

pub struct Bus {
    cpu_vram: [u8; 0xFFFF],
    rom: Option<ROM>,
}

impl Bus {
    pub fn new(rom: ROM) -> Self {
        Self {
            cpu_vram: [0; 0xFFFF],
            rom: Some(rom),
        }
    }

    pub fn load_rom(&mut self, rom: ROM) {
        self.rom = Some(rom);
    }
}

impl Mem for Bus {
    fn read_mem(&self, addr: u16) -> u8 {
        match addr {
            RAM ..= RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize]
            }
            PPU_REGISTERS ..= PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0x2007;
                println!("PPU register read at {:#X}", addr);
                todo!("PPU is not supported yet - read")
            }
            0x8000 ..= 0xFFFF => {
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
            RAM ..= RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0x07FF;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS ..= PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0x2007;
                println!("PPU register write at {:#X}", addr);
                todo!("PPU is not supported yet - write")
            }
            ROM_START_IN_MEMORY ..= 0xFFFF => {
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