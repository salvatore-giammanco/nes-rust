pub const STATUS_RESET: u8 = 0b0010_0100;

pub enum StatusFlag {
    Carry,            // Bit 0
    Zero,             // Bit 1
    InterruptDisable, // Bit 2
    Decimal,          // Bit 3
    B,                // Bit 4
    // Bit 5 (always set to 1)
    Overflow,         // Bit 6
    Negative,         // Bit 7
}

pub struct FlagMask {
    set: u8,
    unset: u8,
}

pub struct ProcessorStatus {
    pub status: u8,
}

impl ProcessorStatus {
    pub fn new() -> Self {
        Self {
            status: STATUS_RESET,
        }
    }

    fn get_mask(&self, flag: StatusFlag) -> FlagMask {
        match flag {
            StatusFlag::Carry => FlagMask {
                set: 0b0000_0001,
                unset: 0b1111_1110,
            },
            StatusFlag::Zero => FlagMask {
                set: 0b0000_0010,
                unset: 0b1111_1101,
            },
            StatusFlag::InterruptDisable => FlagMask {
                set: 0b0000_0100,
                unset: 0b1111_1011,
            },
            StatusFlag::Decimal => FlagMask {
                set: 0b0000_1000,
                unset: 0b1111_0111,
            },
            StatusFlag::B => FlagMask {
                set: 0b0001_0000,
                unset: 0b1110_1111,
            },
            StatusFlag::Overflow => FlagMask {
                set: 0b0100_0000,
                unset: 0b1011_1111,
            },
            StatusFlag::Negative => FlagMask {
                set: 0b1000_0000,
                unset: 0b0111_1111,
            },
        }
    }

    pub fn set_flag(&mut self, flag: StatusFlag, bit: bool) {
        match bit {
            true => self.status = self.status | self.get_mask(flag).set,
            false => self.status = self.status & self.get_mask(flag).unset,
        };
    }

    pub fn set_from_byte(&mut self, byte: u8) {
        self.status = byte;
    }

    pub fn get_flag(&self, flag: StatusFlag) -> bool {
        let check = self.get_mask(flag).set & self.status;
        check.count_ones() != 0
    }

    pub fn update_zero_and_negative_registers(&mut self, value: u8) {
        self.set_flag(StatusFlag::Zero, value == 0);
        self.set_flag(StatusFlag::Negative, value & 0x80 != 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_flag() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Carry, true);
        assert_eq!(p.status, 0b0010_0001);
    }
}
