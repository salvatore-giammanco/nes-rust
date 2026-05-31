use crate::ppu::BitFlags;

pub enum MaskFlags {
    Greyscale,
    ShowBackground,
    ShowSprites,
    BackgroundRendering,
    SpriteRendering,
    EmphasizeRed,
    EmphasizeGreen,
    EmphasizeBlue,
}

pub struct MaskRegister {
    pub status: u8,
}

pub struct FlagMask {
    set: u8,
    unset: u8,
}

impl BitFlags<MaskFlags> for MaskRegister {
    fn new() -> Self {
        Self { status: 0 }
    }

    fn get_mask(&self, flag: MaskFlags) -> crate::ppu::bitflags::FlagMask {
        match flag {
            MaskFlags::Greyscale => crate::ppu::bitflags::FlagMask {
                set: 0b0000_0001,
                unset: 0b1111_1110,
            },
            MaskFlags::ShowBackground => crate::ppu::bitflags::FlagMask {
                set: 0b0000_0010,
                unset: 0b1111_1101,
            },
            MaskFlags::ShowSprites => crate::ppu::bitflags::FlagMask {
                set: 0b0000_0100,
                unset: 0b1111_1011,
            },
            MaskFlags::BackgroundRendering => crate::ppu::bitflags::FlagMask {
                set: 0b0000_1000,
                unset: 0b1111_0111,
            },
            MaskFlags::SpriteRendering => crate::ppu::bitflags::FlagMask {
                set: 0b0001_0000,
                unset: 0b1110_1111,
            },
            MaskFlags::EmphasizeRed => crate::ppu::bitflags::FlagMask {
                set: 0b0010_0000,
                unset: 0b1101_1111,
            },
            MaskFlags::EmphasizeGreen => crate::ppu::bitflags::FlagMask {
                set: 0b0100_0000,
                unset: 0b1011_1111,
            },
            MaskFlags::EmphasizeBlue => crate::ppu::bitflags::FlagMask {
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
