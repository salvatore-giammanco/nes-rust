use super::bitflags::BitFlags;
use super::bitflags::FlagMask;
pub enum ControlFlags {
    NameTable1,
    NameTable2,
    VramAddIncrement,
    SpritePatternAddr,
    BackgroundPatternAddr,
    SpriteSize,
    MasterSlaveSelect,
    GenerateNMI,
}

pub struct ControlRegister {
    pub status: u8,
}

impl ControlRegister {
    pub fn get_vram_address_increment(&self) -> u8 {
        if self.get_flag(ControlFlags::VramAddIncrement) {
            32
        } else {
            1
        }
    }
}

impl BitFlags<ControlFlags> for ControlRegister {
    fn new() -> Self {
        Self { status: 0 }
    }
    fn get_mask(&self, flag: ControlFlags) -> FlagMask {
        match flag {
            ControlFlags::NameTable1 => FlagMask {
                set: 0b0000_0001,
                unset: 0b1111_1110,
            },
            ControlFlags::NameTable2 => FlagMask {
                set: 0b0000_0010,
                unset: 0b1111_1101,
            },
            ControlFlags::VramAddIncrement => FlagMask {
                set: 0b0000_0100,
                unset: 0b1111_1011,
            },
            ControlFlags::SpritePatternAddr => FlagMask {
                set: 0b0000_1000,
                unset: 0b1111_0111,
            },
            ControlFlags::BackgroundPatternAddr => FlagMask {
                set: 0b0001_0000,
                unset: 0b1110_1111,
            },
            ControlFlags::SpriteSize => FlagMask {
                set: 0b0010_0000,
                unset: 0b1101_1111,
            },
            ControlFlags::MasterSlaveSelect => FlagMask {
                set: 0b0100_0000,
                unset: 0b1011_1111,
            },
            ControlFlags::GenerateNMI => FlagMask {
                set: 0b1000_0000,
                unset: 0b0111_1111,
            },
        }
    }

    fn get_status(&self) -> u8 {
        self.status
    }

    fn set_from_byte(&mut self, byte: u8) {
        self.status = byte;
    }
}
