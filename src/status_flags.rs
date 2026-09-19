pub const STATUS_RESET: u8 = 0b0010_0100;

pub enum StatusFlag {
    Carry,            // Bit 0
    Zero,             // Bit 1
    InterruptDisable, // Bit 2
    Decimal,          // Bit 3
    B,                // Bit 4
    // Bit 5 (always set to 1)
    Overflow, // Bit 6
    Negative, // Bit 7
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
        // Force bit 5 (hardwired) to 1
        self.status = byte | 0b0010_0000;
    }

    pub fn get_flag(&self, flag: StatusFlag) -> bool {
        let check = self.get_mask(flag).set & self.status;
        check.count_ones() != 0
    }

    pub fn update_zero_register(&mut self, value: u8) {
        self.set_flag(StatusFlag::Zero, value == 0);
    }

    pub fn update_zero_and_negative_registers(&mut self, value: u8) {
        self.set_flag(StatusFlag::Zero, value == 0);
        self.set_flag(StatusFlag::Negative, value & 0b1000_0000 != 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_status_is_reset() {
        let p = ProcessorStatus::new();
        assert_eq!(p.status, STATUS_RESET);
    }

    // --- set_flag / get_flag round-trip for every flag ---

    #[test]
    fn test_carry_set_and_get() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Carry, true);
        assert!(p.get_flag(StatusFlag::Carry));
        assert_eq!(p.status & 0b0000_0001, 0b0000_0001);
    }

    #[test]
    fn test_carry_unset() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Carry, true);
        p.set_flag(StatusFlag::Carry, false);
        assert!(!p.get_flag(StatusFlag::Carry));
        assert_eq!(p.status & 0b0000_0001, 0);
    }

    #[test]
    fn test_zero_set_and_get() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Zero, true);
        assert!(p.get_flag(StatusFlag::Zero));
        assert_eq!(p.status & 0b0000_0010, 0b0000_0010);
    }

    #[test]
    fn test_zero_unset() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Zero, true);
        p.set_flag(StatusFlag::Zero, false);
        assert!(!p.get_flag(StatusFlag::Zero));
        assert_eq!(p.status & 0b0000_0010, 0);
    }

    #[test]
    fn test_interrupt_disable_set_and_get() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::InterruptDisable, true);
        assert!(p.get_flag(StatusFlag::InterruptDisable));
        assert_eq!(p.status & 0b0000_0100, 0b0000_0100);
    }

    #[test]
    fn test_interrupt_disable_unset() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::InterruptDisable, false);
        assert!(!p.get_flag(StatusFlag::InterruptDisable));
        assert_eq!(p.status & 0b0000_0100, 0);
    }

    #[test]
    fn test_decimal_set_and_get() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Decimal, true);
        assert!(p.get_flag(StatusFlag::Decimal));
        assert_eq!(p.status & 0b0000_1000, 0b0000_1000);
    }

    #[test]
    fn test_decimal_unset() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Decimal, true);
        p.set_flag(StatusFlag::Decimal, false);
        assert!(!p.get_flag(StatusFlag::Decimal));
        assert_eq!(p.status & 0b0000_1000, 0);
    }

    #[test]
    fn test_b_set_and_get() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::B, true);
        assert!(p.get_flag(StatusFlag::B));
        assert_eq!(p.status & 0b0001_0000, 0b0001_0000);
    }

    #[test]
    fn test_b_unset() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::B, true);
        p.set_flag(StatusFlag::B, false);
        assert!(!p.get_flag(StatusFlag::B));
        assert_eq!(p.status & 0b0001_0000, 0);
    }

    #[test]
    fn test_overflow_set_and_get() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Overflow, true);
        assert!(p.get_flag(StatusFlag::Overflow));
        assert_eq!(p.status & 0b0100_0000, 0b0100_0000);
    }

    #[test]
    fn test_overflow_unset() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Overflow, true);
        p.set_flag(StatusFlag::Overflow, false);
        assert!(!p.get_flag(StatusFlag::Overflow));
        assert_eq!(p.status & 0b0100_0000, 0);
    }

    #[test]
    fn test_negative_set_and_get() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Negative, true);
        assert!(p.get_flag(StatusFlag::Negative));
        assert_eq!(p.status & 0b1000_0000, 0b1000_0000);
    }

    #[test]
    fn test_negative_unset() {
        let mut p = ProcessorStatus::new();
        p.set_flag(StatusFlag::Negative, true);
        p.set_flag(StatusFlag::Negative, false);
        assert!(!p.get_flag(StatusFlag::Negative));
        assert_eq!(p.status & 0b1000_0000, 0);
    }

    // --- set_flag does not disturb other flags ---

    #[test]
    fn test_set_flag_preserves_others() {
        let mut p = ProcessorStatus::new();
        p.status = 0x00;
        p.set_flag(StatusFlag::Carry, true);
        p.set_flag(StatusFlag::Negative, true);
        assert!(p.get_flag(StatusFlag::Carry));
        assert!(p.get_flag(StatusFlag::Negative));
        assert!(!p.get_flag(StatusFlag::Zero));
        assert!(!p.get_flag(StatusFlag::Overflow));
    }

    // --- set_from_byte ---

    #[test]
    fn test_set_from_byte_forces_bit5() {
        let mut p = ProcessorStatus::new();
        p.set_from_byte(0b0000_0000);
        assert_eq!(p.status, 0b0010_0000);
    }

    #[test]
    fn test_set_from_byte_preserves_bit5_if_set() {
        let mut p = ProcessorStatus::new();
        p.set_from_byte(0b0010_0000);
        assert_eq!(p.status, 0b0010_0000);
    }

    #[test]
    fn test_set_from_byte_full_value() {
        let mut p = ProcessorStatus::new();
        p.set_from_byte(0xFF);
        assert_eq!(p.status, 0xFF);
    }

    #[test]
    fn test_set_from_byte_clears_existing_flags() {
        let mut p = ProcessorStatus::new();
        p.status = 0xFF;
        p.set_from_byte(0b0000_0001);
        assert_eq!(p.status, 0b0010_0001);
    }

    // --- update_zero_register ---

    #[test]
    fn test_update_zero_register_zero_value() {
        let mut p = ProcessorStatus::new();
        p.status = 0x00;
        p.update_zero_register(0x00);
        assert!(p.get_flag(StatusFlag::Zero));
    }

    #[test]
    fn test_update_zero_register_nonzero_value() {
        let mut p = ProcessorStatus::new();
        p.status = 0x00;
        p.set_flag(StatusFlag::Zero, true);
        p.update_zero_register(0x42);
        assert!(!p.get_flag(StatusFlag::Zero));
    }

    // --- update_zero_and_negative_registers ---

    #[test]
    fn test_update_zn_zero() {
        let mut p = ProcessorStatus::new();
        p.status = 0x00;
        p.update_zero_and_negative_registers(0x00);
        assert!(p.get_flag(StatusFlag::Zero));
        assert!(!p.get_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_update_zn_negative() {
        let mut p = ProcessorStatus::new();
        p.status = 0x00;
        p.update_zero_and_negative_registers(0x80);
        assert!(!p.get_flag(StatusFlag::Zero));
        assert!(p.get_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_update_zn_positive_nonzero() {
        let mut p = ProcessorStatus::new();
        p.status = 0x00;
        p.update_zero_and_negative_registers(0x01);
        assert!(!p.get_flag(StatusFlag::Zero));
        assert!(!p.get_flag(StatusFlag::Negative));
    }

    #[test]
    fn test_update_zn_clears_previous() {
        let mut p = ProcessorStatus::new();
        p.status = 0x00;
        p.set_flag(StatusFlag::Zero, true);
        p.set_flag(StatusFlag::Negative, true);
        p.update_zero_and_negative_registers(0x40);
        assert!(!p.get_flag(StatusFlag::Zero));
        assert!(!p.get_flag(StatusFlag::Negative));
    }

    // --- bit 5 always-on invariant ---

    #[test]
    fn test_bit5_always_set_after_set_from_byte() {
        let mut p = ProcessorStatus::new();
        p.status = 0x00;
        p.set_from_byte(0x00);
        assert_eq!(p.status & 0b0010_0000, 0b0010_0000);
    }

    #[test]
    fn test_bit5_always_set_after_new() {
        let p = ProcessorStatus::new();
        assert_eq!(p.status & 0b0010_0000, 0b0010_0000);
    }
}
