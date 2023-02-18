pub struct CPU {
    program_counter: u16,
    stack_pointer: u8,
    register_accumulator: u8,
    index_register_x: u8,
    index_register_y: u8,
    status: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            program_counter: 0,
            stack_pointer: 0,
            register_accumulator: 0,
            index_register_x: 0,
            index_register_y: 0,
            status: 0,
        }
    }

    pub fn fetch(&self, program: &Vec<u8>) -> u8 {
        program[self.program_counter as usize]
    }

    pub fn execute(&mut self, program: Vec<u8>) {
        self.program_counter = 0;
        loop {
            let opcode = self.fetch(&program);
            self.program_counter += 1;

            match opcode {
                0x00 => {
                    return;
                }
                0xA9 => {
                    // LDA - Load Accumulator - Addressing Mode: Immediate
                    let param = self.fetch(&program);
                    self.program_counter += 1;
                    self.register_accumulator = param;

                    self.update_zero_and_negative_registers(self.register_accumulator);
                }
                0xAA => {
                    // TAX - Transfer Accumulator to register X
                    self.index_register_x = self.register_accumulator;

                    self.update_zero_and_negative_registers(self.index_register_x);
                }
                0xE8 => {
                    // INX - Increment register X
                    if self.index_register_x == 0xFF {
                        self.index_register_x = 0;
                    } else {
                        self.index_register_x += 1;
                    }

                    self.update_zero_and_negative_registers(self.index_register_x);
                }
                _ => todo!(),
            }
        }
    }

    fn update_zero_and_negative_registers(&mut self, value: u8) {
        if value == 0 {
            self.status = self.status | 0b0000_0010; // Set C flag
        } else {
            self.status = self.status & 0b1111_1101; // Unset C flag
        }
        if value & 0b1000_0000 != 0b00 {
            self.status = self.status | 0b1000_0000; // Set N flag
        } else {
            self.status = self.status & 0b0111_1111; // Unset N flag
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load() {
        let mut cpu = CPU::new();
        cpu.execute(vec![0xA9, 0x42, 0x00]);
        assert_eq!(cpu.register_accumulator, 0x42);
        assert_eq!(cpu.status & 0b0000_0010, 0);
    }

    #[test]
    fn test_0xa9_lda_immediate_negative_flag() {
        let mut cpu = CPU::new();
        cpu.execute(vec![0xA9, 0xFF, 0x00]);
        assert_eq!(cpu.status & 0b1000_0000, 0b1000_0000);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.execute(vec![0xA9, 0x00, 0x00]);
        assert_eq!(cpu.status & 0b0000_0010, 0b10);
    }

    #[test]
    fn test_0xaa_tax_immediate_load() {
        let mut cpu = CPU::new();
        cpu.register_accumulator = 0x42;
        cpu.execute(vec![0xAA, 0x00]);
        assert_eq!(cpu.index_register_x, 0x42);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.execute(vec![0xA9, 0xC0, 0xAA, 0xE8, 0x00]);

        assert_eq!(cpu.index_register_x, 0xC1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.index_register_x = 0xFF;
        cpu.execute(vec![0xE8, 0xE8, 0x00]);

        assert_eq!(cpu.index_register_x, 1)
    }

    #[test]
    fn test() {
        assert_eq!(1, 1);
    }
}
