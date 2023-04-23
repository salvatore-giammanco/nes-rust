pub struct CPU {
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub register_accumulator: u8,
    pub index_register_x: u8,
    pub index_register_y: u8,
    pub status: u8,
    memory: [u8; 0xFFFF],
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
            memory: [0; 0xFFFF],
        }
    }

    pub fn load_program(&mut self, program: Vec<u8>) {
        // TODO check the length of the program
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.write_mem_u16(0xFFFC, 0x8000);
    }

    pub fn reset(&mut self) {
        self.program_counter = self.read_mem_u16(0xFFFC); // Address at 0xFFFC 2 bytes little endian
        self.stack_pointer = 0;
        self.register_accumulator = 0;
        self.index_register_x = 0;
        self.index_register_y = 0;
        self.status = 0;
    }

    pub fn load_and_execute(&mut self, program: Vec<u8>) {
        self.load_program(program);
        self.reset();
        self.execute();
    }

    fn read_mem(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    pub fn read_mem_u16(&self, addr: u16) -> u16 {
        // Reading 2 bytes in little endian
        let little = self.read_mem(addr) as u16;
        let big = self.read_mem(addr + 1) as u16;
        big << 8 | little as u16
    }

    fn write_mem(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    fn write_mem_u16(&mut self, addr: u16, value: u16) {
        let little = (value & 0xff) as u8;
        let big = (value >> 8) as u8;
        self.write_mem(addr, little);
        self.write_mem(addr + 1, big);
    }

    pub fn fetch(&self) -> u8 {
        self.memory[self.program_counter as usize]
    }

    pub fn execute(&mut self) {
        loop {
            let opcode = self.fetch();
            self.program_counter += 1;

            match opcode {
                0x00 => {
                    return;
                }
                0xA9 => {
                    // LDA - Load Accumulator - Addressing Mode: Immediate
                    let param = self.fetch();
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
        cpu.load_and_execute(vec![0xA9, 0x42, 0x00]);
        assert_eq!(cpu.register_accumulator, 0x42);
        assert_eq!(cpu.status & 0b0000_0010, 0);
    }

    #[test]
    fn test_0xa9_lda_immediate_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_execute(vec![0xA9, 0xFF, 0x00]);
        assert_eq!(cpu.status & 0b1000_0000, 0b1000_0000);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_execute(vec![0xA9, 0x00, 0x00]);
        assert_eq!(cpu.status & 0b0000_0010, 0b10);
    }

    #[test]
    fn test_0xaa_tax_immediate_load() {
        let mut cpu = CPU::new();
        cpu.load_and_execute(vec![0xA9, 0x42, 0xAA, 0x00]);
        assert_eq!(cpu.index_register_x, 0x42);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_execute(vec![0xA9, 0xC0, 0xAA, 0xE8, 0x00]);

        assert_eq!(cpu.index_register_x, 0xC1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_execute(vec![0xA9, 0xFF, 0xAA, 0xE8, 0xE8, 0x00]);

        assert_eq!(cpu.index_register_x, 1)
    }
}
