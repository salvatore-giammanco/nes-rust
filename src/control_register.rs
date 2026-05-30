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

pub struct FlagMask {
    set: u8,
    unset: u8,
}

impl ControlRegister {
    pub fn new() -> Self {
        Self { status: 0 }
    }

    pub fn get_vram_address_increment(&self) -> u8 {
        if self.get_flag(ControlFlags::VramAddIncrement) {
            32
        } else {
            1
        }
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

    pub fn set_flag(&mut self, flag: ControlFlags, bit: bool) {
        match bit {
            true => self.status = self.status | self.get_mask(flag).set,
            false => self.status = self.status & self.get_mask(flag).unset,
        }
    }

    pub fn set_from_byte(&mut self, byte: u8) {
        // Force bit 5 (hardwired) to 1
        self.status = byte | 0b0010_0000;
    }

    pub fn get_flag(&self, flag: ControlFlags) -> bool {
        let check = self.get_mask(flag).set & self.status;
        check.count_ones() != 0
    }
}
