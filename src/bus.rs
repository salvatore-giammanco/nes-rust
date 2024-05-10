use crate::cpu::Mem;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub struct Bus {
    pub cpu_vram: [u8; 0xFFFF],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            cpu_vram: [0; 0xFFFF],
        }
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
            _ => {
                // println!("Ignoring mem access at {:#X}", addr);
                // Bypassing because there's no ROM emulation yet
                self.cpu_vram[addr as usize]
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
            _ => {
                // println!("Ignoring mem write-access at {:#X}: {:#X}", addr, data);
                // Bypassing because there's no ROM emulation yet
                self.cpu_vram[addr as usize] = data;
            }
        }
    }
}