pub enum StatusFlag {
    Carry,            // Bit 7
    Zero,             // Bit 6
    InterruptDisable, // Bit 5
    Decimal,          // Bit 4
    Overflow,         // Bit 1
    Negative,         // Bit 0
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
        Self { status: 0 }
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

    pub fn update_zero_and_negative_registers(&mut self, value: u8) {
        if value == 0 {
            self.set_flag(StatusFlag::Zero, true);
        } else {
            self.set_flag(StatusFlag::Zero, false);
        }
        if value & 0b1000_0000 != 0b00 {
            self.set_flag(StatusFlag::Negative, true);
        } else {
            self.set_flag(StatusFlag::Negative, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_flag() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Carry, true);
        assert_eq!(p.status, 0b0000_0001);
    }
}
