use super::bitflags::BitFlags;
use super::bitflags::FlagMask;

pub enum PPUStatusFlags {
    VBlank,
    Sprite,
    SpriteOverflow,
}

pub struct PPUStatusRegister {
    pub status: u8,
}

impl BitFlags<PPUStatusFlags> for PPUStatusRegister {
    fn new() -> Self {
        Self { status: 0 }
    }

    fn get_mask(&self, flag: PPUStatusFlags) -> FlagMask {
        match flag {
            PPUStatusFlags::SpriteOverflow => FlagMask {
                set: 0b0010_0000,
                unset: 0b1101_1111,
            },
            PPUStatusFlags::Sprite => FlagMask {
                set: 0b0100_0000,
                unset: 0b1011_1111,
            },
            PPUStatusFlags::VBlank => FlagMask {
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
