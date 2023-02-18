pub struct CPU {
    program_counter: u16,
    stack_pointer: u8,
    register_accumulator: u8,
    index_register_x: u8,
    index_register_y: u8,
    status: u8,
}

impl CPU {
    fn new() -> Self {
        Self {
            program_counter:0,
            stack_pointer: 0,
            register_accumulator: 0,
            index_register_x: 0,
            index_register_y: 0,
            status: 0,
        }
    }
    
    fn fetch(&self, program: &Vec<u8>) -> u8 {
        program[self.program_counter as usize]
    }

    fn execute(&mut self, program: Vec<u8>) {
        self.program_counter = 0;
        loop {
            let opcode = self.fetch(&program);
            self.program_counter += 1;
            
            match opcode {
                0x00 => {
                    return;
                },
                0xA9 => { // Load Accumulator - Addressing Mode: Immediate
                    let param = self.fetch(&program);
                    self.program_counter += 1;
                    self.register_accumulator = param;
                    if param == 0 {
                        self.status = self.status | 0b0000_0010; // Set C flag
                    } else {
                        self.status = self.status & 0b1111_1101; // Unset C flag
                    }
                    if self.register_accumulator & 0b1000_0000 != 0 {
                        self.status = self.status | 0b1000_0000; // Set N flag
                    } else {
                        self.status = self.status & 0b0111_1111; // Unset N flag
                    }
                },
                _ => todo!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_0xA9_LDA_immediate_load() {
        let mut cpu = CPU::new();
        cpu.execute(vec![0xA9, 0x42, 0x00]);
        assert_eq!(cpu.register_accumulator, 0x42);
        assert_eq!(cpu.status & 0b0000_0010, 0);
    }

    #[test]
    fn test_0xA9_LDA_zero_flag() {
        let mut cpu = CPU::new();
        cpu.execute(vec![0xA9, 0x00, 0x00]);
        assert_eq!(cpu.status & 0b0000_0010, 0b10);
    }

    #[test]
    fn test() {
        assert_eq!(1,1);
    }
}
