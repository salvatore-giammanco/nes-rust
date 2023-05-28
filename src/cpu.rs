use std::collections::HashMap;

use crate::opcodes::{OpCode, OpCodeNotFound, self};



pub struct CPU {
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub register_accumulator: u8,
    pub index_register_x: u8,
    pub index_register_y: u8,
    pub status: u8,
    memory: [u8; 0xFFFF],
}

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
   Immediate,
   ZeroPage,
   ZeroPage_X,
   ZeroPage_Y,
   Absolute,
   Absolute_X,
   Absolute_Y,
   Indirect_X,
   Indirect_Y,
   NoneAddressing,
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
        let little = self.read_mem(addr);
        let big = self.read_mem(addr + 1);
        u16::from_le_bytes([little, big])
    }

    fn write_mem(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    fn write_mem_u16(&mut self, addr: u16, value: u16) {
        let bytes = u16::to_le_bytes(value);
        for i in 0..bytes.len() {
            self.write_mem(addr + i as u16, bytes[i])
        }
    }

    pub fn fetch(&self) -> u8 {
        self.memory[self.program_counter as usize]
    }

    pub fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.read_mem(self.program_counter) as u16,
            AddressingMode::ZeroPage_X => {
                let param = self.read_mem(self.program_counter);
                self.index_register_x.wrapping_add(param) as u16
            },
            AddressingMode::ZeroPage_Y => {
                let param = self.read_mem(self.program_counter);
                self.index_register_y.wrapping_add(param) as u16
            },
            AddressingMode::Absolute => self.read_mem_u16(self.program_counter),
            AddressingMode::Absolute_X => {
                let param = self.read_mem_u16(self.program_counter);
                param.wrapping_add(self.index_register_x  as u16)
            },
            AddressingMode::Absolute_Y => {
                let param = self.read_mem_u16(self.program_counter);
                param.wrapping_add(self.index_register_y  as u16)
            },
            AddressingMode::Indirect_X => {
                let param = self.read_mem(self.program_counter);
                let ptr: u8 = param.wrapping_add(self.index_register_x);
                let little: u8 = self.read_mem(ptr as u16);
                let big: u8 = self.read_mem(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([little, big])
            },
            AddressingMode::Indirect_Y => {
                let param = self.read_mem(self.program_counter);
                let little: u8 = self.read_mem(param as u16);
                let big: u8 = self.read_mem(param.wrapping_add(1) as u16);
                let deref_base: u16 = u16::from_le_bytes([little, big]);
                deref_base.wrapping_add(self.index_register_y as u16)

            },
            _ => {               
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem(addr);

        self.register_accumulator = value;
        self.update_zero_and_negative_registers(self.register_accumulator);
    }

    pub fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.write_mem(addr, self.register_accumulator);
    }

    pub fn execute(&mut self) {
        let ref opcodes: HashMap<u8, &'static OpCode> = *opcodes::CPU_OPCODES_MAP;
        loop {
            let code = self.fetch();
            self.program_counter += 1;

            let opcode = opcodes.get(&code).expect(&format!("Unknown opcode {:x}", code));

            match opcode.label {
                "BRK" => {
                    // Beak
                    return;
                },
                "LDA" => {
                    // Load Accumulator
                    self.lda(&opcode.addressing_mode);
                    self.program_counter += opcode.cycles - 1;
                },
                "STA" => {
                    // Store Accumulator
                    self.sta(&opcode.addressing_mode);
                    self.program_counter += opcode.cycles - 1;
                },
                "TAX" => {
                    // TAX - Transfer Accumulator to register X
                    self.index_register_x = self.register_accumulator;

                    self.update_zero_and_negative_registers(self.index_register_x);
                }
                "INX" => {
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

    fn update_carry_flag(&mut self,) {}

    fn update_zero_and_negative_registers(&mut self, value: u8) {
        if value == 0 {
            self.status = self.status | 0b0000_0010; // Set Z flag
        } else {
            self.status = self.status & 0b1111_1101; // Unset Z flag
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

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.write_mem(0x10, 0x55);
        cpu.load_and_execute(vec![0xa5, 0x10, 0x00]);
 
        assert_eq!(cpu.register_accumulator, 0x55);
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new();
        cpu.load_and_execute(vec![0xa9, 0x42, 0x85, 0x10]);
        assert_eq!(cpu.read_mem(0x10), 0x42);
    }

    #[test]
    fn test_get_operand_address_zero_page() {
        let mut cpu = CPU::new();
        cpu.load_program(vec![0x10]);
        cpu.reset();
        let addr = cpu.get_operand_address(&AddressingMode::ZeroPage);
        assert_eq!(addr, 0x10);
    }
}
