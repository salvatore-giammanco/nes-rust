use std::collections::HashMap;
use std::env;
use std::ops::{BitAnd, BitOr, BitXor};

use crate::opcodes::{self, OpCode};
use crate::status_flags::{ProcessorStatus, StatusFlag};
use crate::bus::Bus;


const STACK: u16 = 0x100;
pub const STACK_RESET: u8 = 0xFD;

pub struct CPU {
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub register_accumulator: u8,
    pub index_register_x: u8,
    pub index_register_y: u8,
    pub status: ProcessorStatus,
    pub bus: Bus,
    pub debug: bool,
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

pub trait Mem {
    fn read_mem(&self, addr: u16) -> u8;

    fn read_mem_u16(&self, addr: u16) -> u16 {
        // Reading 2 bytes in little endian
        let little = self.read_mem(addr);
        let big = self.read_mem(addr + 1);
        u16::from_le_bytes([little, big])
    }

    fn write_mem(&mut self, addr: u16, value: u8);

    fn write_mem_u16(&mut self, addr: u16, value: u16) {
        // Writing 2 bytes in little endian
        let bytes = u16::to_le_bytes(value);
        for i in 0..bytes.len() {
            self.write_mem(addr + i as u16, bytes[i])
        }
    }
}

impl Mem for CPU {
    fn read_mem(&self, addr: u16) -> u8 {
        self.bus.read_mem(addr)
    }

    fn read_mem_u16(&self, addr: u16) -> u16 {
        self.bus.read_mem_u16(addr)
    }

    fn write_mem(&mut self, addr: u16, value: u8) {
        self.bus.write_mem(addr, value);
    }

    fn write_mem_u16(&mut self, addr: u16, value: u16) {
        self.bus.write_mem_u16(addr, value);
    }
}

impl CPU {
    pub fn new(bus: Bus) -> Self {
        let debug: bool = match env::var("DEBUG") {
            Ok(_) => true,
            Err(_) => false,
        };
        Self {
            program_counter: 0,
            stack_pointer: STACK_RESET,
            register_accumulator: 0,
            index_register_x: 0,
            index_register_y: 0,
            status: ProcessorStatus::new(),
            bus,
            debug,
        }
    }


    pub fn load_test(&mut self, program: Vec<u8>) {
        for i in 0..(program.len() as u16) {
            self.write_mem(0x0600 + i, program[i as usize]);
        }
        self.write_mem_u16(0xFFFC, 0x0600);
    }

    pub fn disassemble(&self, program: Vec<u8>) {
        let ref opcodes: HashMap<u8, &'static OpCode> = *opcodes::CPU_OPCODES_MAP;
        let mut pos: usize = 0;
        while pos < program.len() {
            let addr = 0x600 + pos;
            let opcode = opcodes.get(&program[pos]).expect(&format!("Unknown opcode {:x}", pos));
            let mut args: Vec<u8> = Vec::new();
            if opcode.bytes > 1 {
                for i in 1..(opcode.bytes) {
                    args.push(program[pos + i as usize]);
                }
            }
            pos += opcode.bytes as usize;
            println!(
                "{:#04X}| {:#04X}: {:?} ({:02X?}) - {:?}",
                addr,
                opcode.opcode,
                opcode.label,
                args,
                opcode.addressing_mode
            );
        }
    }

    pub fn load_program(&mut self, program: Vec<u8>) {
        // TODO check the length of the program
        for i in 0..(program.len() as u16) {
            self.write_mem(0x8000 + i, program[i as usize]);
        }
        self.write_mem_u16(0xFFFC, 0x8000);
    }

    pub fn reset(&mut self) {
        self.program_counter = self.read_mem_u16(0xFFFC); // Address at 0xFFFC 2 bytes little endian
        self.stack_pointer = STACK_RESET;
        self.register_accumulator = 0;
        self.index_register_x = 0;
        self.index_register_y = 0;
        self.status = ProcessorStatus::new();
    }

    pub fn load_and_execute(&mut self, program: Vec<u8>) {
        self.load_program(program);
        self.reset();
        self.execute();
    }

    pub fn fetch(&self) -> u8 {
        self.read_mem(self.program_counter)
    }

    pub fn stack_push(&mut self, value: u8) {
        if self.stack_pointer > 0 {
            let pointer: u16 = STACK + self.stack_pointer as u16;
            self.write_mem(pointer, value);
            self.stack_pointer -= 1;
        } else {
            panic!("Stack Overflow!")
        }
    }

    pub fn stack_push_u16(&mut self, value: u16) {
        let bytes = u16::to_le_bytes(value);
        self.stack_push(bytes[0]);
        self.stack_push(bytes[1]);
    }

    pub fn stack_pull(&mut self) -> u8 {
        let pointer: u16 = STACK + self.stack_pointer as u16 + 1;
        let value: u8 = self.read_mem(pointer);
        if self.stack_pointer < STACK_RESET {
            self.stack_pointer += 1;
        }
        value
    }

    pub fn stack_pull_u16(&mut self) -> u16 {
        let little: u8 = self.stack_pull();
        let big: u8 = self.stack_pull();
        u16::from_le_bytes([big, little])
    }

    pub fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.read_mem(self.program_counter) as u16,
            AddressingMode::ZeroPage_X => {
                let param = self.read_mem(self.program_counter);
                self.index_register_x.wrapping_add(param) as u16
            }
            AddressingMode::ZeroPage_Y => {
                let param = self.read_mem(self.program_counter);
                self.index_register_y.wrapping_add(param) as u16
            }
            AddressingMode::Absolute => self.read_mem_u16(self.program_counter),
            AddressingMode::Absolute_X => {
                let param = self.read_mem_u16(self.program_counter);
                param.wrapping_add(self.index_register_x as u16)
            }
            AddressingMode::Absolute_Y => {
                let param = self.read_mem_u16(self.program_counter);
                param.wrapping_add(self.index_register_y as u16)
            }
            AddressingMode::Indirect_X => {
                let param = self.read_mem(self.program_counter);
                let ptr: u8 = param.wrapping_add(self.index_register_x);
                let little: u8 = self.read_mem(ptr as u16);
                let big: u8 = self.read_mem(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([little, big])
            }
            AddressingMode::Indirect_Y => {
                let param = self.read_mem(self.program_counter);
                let little: u8 = self.read_mem(param as u16);
                let big: u8 = self.read_mem(param.wrapping_add(1) as u16);
                let deref_base: u16 = u16::from_le_bytes([little, big]);
                deref_base.wrapping_add(self.index_register_y as u16)
            }
            _ => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    pub fn load_accumulator(&mut self, value: u8) {
        self.register_accumulator = value;
        self.status
            .update_zero_and_negative_registers(self.register_accumulator);
    }

    pub fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem(addr);

        self.load_accumulator(value);
    }

    pub fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.write_mem(addr, self.register_accumulator);
    }

    pub fn add_width_carry(&mut self, value: u8) {
        let carry: u8 = self.status.get_flag(StatusFlag::Carry) as u8;
        let result: u16 = self.register_accumulator as u16 + value as u16 + carry as u16;

        let carry: bool = if result > 0xFF { true } else { false };
        let result: u8 = result as u8;

        self.status.set_flag(StatusFlag::Carry, carry);

        let overflow: bool = (value ^ result) & (result ^ self.register_accumulator) & 0x80 != 0;
        self.status.set_flag(StatusFlag::Overflow, overflow);
        self.load_accumulator(result);
    }

    pub fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem(addr);

        self.add_width_carry(value);
    }

    pub fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem(addr);

        self.add_width_carry(((value as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    pub fn asl(&mut self, value: u8) -> u8 {
        let last_bit = value & 0b1000_0000;
        let carry = last_bit.count_ones() != 0;
        self.status.set_flag(StatusFlag::Carry, carry);
        value << 1
    }

    pub fn lsr(&mut self, value: u8) -> u8 {
        let first_bit = value & 0b0000_0001;
        let carry = first_bit.count_ones() != 0;
        self.status.set_flag(StatusFlag::Carry, carry);
        value >> 1
    }

    pub fn rol(&mut self, value: u8) -> u8 {
        let last_bit = value & 0b1000_0000;
        let carry = last_bit.count_ones() != 0;
        self.status.set_flag(StatusFlag::Carry, carry);
        (value << 1) | carry as u8
    }

    pub fn ror(&mut self, value: u8) -> u8 {
        let first_bit = value & 0b0000_0001;
        let carry = first_bit.count_ones() != 0;
        self.status.set_flag(StatusFlag::Carry, carry);
        (value >> 1) | (carry as u8).reverse_bits()
    }

    pub fn branch(&mut self, condition: bool) {
        if condition {
            let relative_displacement: i8 = self.read_mem(self.program_counter) as i8;
            self.program_counter = self.program_counter
                .wrapping_add(1)
                .wrapping_add(relative_displacement as u16);
        }
    }

    pub fn compare(&mut self, mode: &AddressingMode, other: u8) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem(addr);

        self.status.set_flag(StatusFlag::Carry, other >= value);
        self.status
            .update_zero_and_negative_registers(other.wrapping_sub(value));
    }

    pub fn decrement(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        self.status.update_zero_and_negative_registers(result);
        result
    }

    pub fn increment(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        self.status.update_zero_and_negative_registers(result);
        result
    }

    pub fn execute(&mut self) {
        self.execute_with_callback(|_| {});
    }

    pub fn debug_cpu_status(&self, opcode: &OpCode) {
        if !self.debug {
            return;
        }

        let mut opcode_dump = vec![];
        opcode_dump.push(opcode.opcode);

        for i in 1..opcode.bytes {
            opcode_dump.push(self.read_mem(self.program_counter + i as u16 - 1));
        }
        let opcode_dump_str: String = opcode_dump
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<String>>()
            .join(" ");

        let mut status: String = String::new();
        let space_padding_dump = " ".repeat(10 - opcode_dump_str.len());
        let space_padding_assembly = " ".repeat(28);
        status = format!(
            "{:04X}  {}{}{}{}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.program_counter - 1,
            opcode_dump_str,
            space_padding_dump,
            opcode.label,
            space_padding_assembly,
            self.register_accumulator,
            self.index_register_x,
            self.index_register_y,
            self.status.status,
            self.stack_pointer,
        );
        println!("{}", status);
    }

    pub fn execute_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static OpCode> = *opcodes::CPU_OPCODES_MAP;
        loop {
            callback(self);
            let code = self.fetch();
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = opcodes
                .get(&code)
                .expect(&format!("Unknown opcode {:x}", code));

            self.debug_cpu_status(&opcode);

            match opcode.label {
                "ADC" => {
                    // Add with carry
                    self.adc(&opcode.addressing_mode);
                }
                "AND" => {
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    let value: u8 = self.read_mem(addr);
                    self.register_accumulator = self.register_accumulator.bitand(value);
                    self.status
                        .update_zero_and_negative_registers(self.register_accumulator);
                }
                "ASL" => {
                    // Arithmetic Shift Left
                    match opcode.addressing_mode {
                        AddressingMode::NoneAddressing => {
                            self.register_accumulator = self.asl(self.register_accumulator);
                        }
                        _ => {
                            let addr = self.get_operand_address(&opcode.addressing_mode);
                            let value = self.read_mem(addr);
                            let result = self.asl(value);
                            self.write_mem(addr, result);
                        }
                    }
                    self.status
                        .update_zero_and_negative_registers(self.register_accumulator);
                }
                "BCC" => self.branch(!self.status.get_flag(StatusFlag::Carry)),
                "BCS" => self.branch(self.status.get_flag(StatusFlag::Carry)),
                "BEQ" => self.branch(self.status.get_flag(StatusFlag::Zero)),
                "BIT" => {
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    let result = self.register_accumulator.bitand(self.read_mem(addr));
                    let overflow = result & 0x40 != 0;
                    self.status.set_flag(StatusFlag::Overflow, overflow);
                    self.status.update_zero_and_negative_registers(result);
                }
                "BMI" => self.branch(self.status.get_flag(StatusFlag::Negative)),
                "BNE" => self.branch(!self.status.get_flag(StatusFlag::Zero)),
                "BPL" => self.branch(!self.status.get_flag(StatusFlag::Negative)),
                "BRK" => {
                    // Break
                    return;
                }
                "BVC" => self.branch(!self.status.get_flag(StatusFlag::Overflow)),
                "BVS" => self.branch(self.status.get_flag(StatusFlag::Overflow)),
                "CLC" => self.status.set_flag(StatusFlag::Carry, false),
                "CLD" => self.status.set_flag(StatusFlag::Decimal, false),
                "CLI" => self.status.set_flag(StatusFlag::InterruptDisable, false),
                "CLV" => self.status.set_flag(StatusFlag::Overflow, false),
                "CMP" => self.compare(&opcode.addressing_mode, self.register_accumulator),
                "CPX" => self.compare(&opcode.addressing_mode, self.index_register_x),
                "CPY" => self.compare(&opcode.addressing_mode, self.index_register_y),
                "DEC" => {
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    let value = self.read_mem(addr);
                    let result = self.decrement(value);
                    self.write_mem(addr, result);
                }
                "DEX" => self.index_register_x = self.decrement(self.index_register_x),
                "DEY" => self.index_register_y = self.decrement(self.index_register_y),
                "EOR" => {
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    let value = self.read_mem(addr);
                    let result = self.register_accumulator.bitxor(value);
                    self.load_accumulator(result);
                }
                "INC" => {
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    let value = self.read_mem(addr);
                    let result = self.increment(value);
                    self.write_mem(addr, result);
                }
                "INX" => self.index_register_x = self.increment(self.index_register_x),
                "INY" => self.index_register_y = self.increment(self.index_register_y),
                "JMP" => {
                    // Jump
                    match opcode.addressing_mode {
                        AddressingMode::Absolute => {
                            let addr = self.get_operand_address(&opcode.addressing_mode);
                            self.program_counter = addr;
                        }
                        _ => {
                            // Indirect
                            let addr = self.read_mem_u16(self.program_counter);

                            let indirect_ref = if addr & 0x00FF == 0x00FF {
                                // 6502 page boundary bug
                                // https://www.nesdev.org/obelisk-6502-guide/reference.html#JMP
                                let little = self.read_mem(addr);
                                let big = self.read_mem(addr & 0xFF00);
                                u16::from_le_bytes([little, big])
                            } else {
                                self.read_mem_u16(addr)
                            };

                            self.program_counter = indirect_ref;
                        }
                    }
                }
                "JSR" => {
                    // Jump To Subroutine
                    self.stack_push_u16(self.program_counter + 1); // + 2 - 1
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    self.program_counter = addr;
                }
                "LDA" => {
                    // Load Accumulator
                    self.lda(&opcode.addressing_mode);
                }
                "LDX" => {
                    // Load X Register
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    let value = self.read_mem(addr);
                    self.index_register_x = value;
                    self.status.update_zero_and_negative_registers(value);
                }
                "LDY" => {
                    // Load Y Register
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    let value = self.read_mem(addr);
                    self.index_register_y = value;
                    self.status.update_zero_and_negative_registers(value);
                }
                "LSR" => {
                    // Logical Shift Right
                    match opcode.addressing_mode {
                        AddressingMode::NoneAddressing => {
                            self.register_accumulator = self.lsr(self.register_accumulator);
                        }
                        _ => {
                            let addr = self.get_operand_address(&opcode.addressing_mode);
                            let value = self.read_mem(addr);
                            let result = self.lsr(value);
                            self.write_mem(addr, result);
                        }
                    }
                    self.status
                        .update_zero_and_negative_registers(self.register_accumulator);
                }
                "NOP" => {}
                "ORA" => {
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    let value = self.read_mem(addr);
                    let result = self.register_accumulator.bitor(value);
                    self.load_accumulator(result);
                }
                "PHA" => {
                    // Push Accumulator
                    self.stack_push(self.register_accumulator);
                }
                "PHP" => {
                    // Push Processor Status
                    self.status.set_flag(StatusFlag::B, true);
                    self.stack_push(self.status.status);
                }
                "PLA" => {
                    // Pull Accumulator
                    let value = self.stack_pull();
                    self.load_accumulator(value);
                }
                "PLP" => {
                    // Pull Processor Status
                    let status: u8 = self.stack_pull();
                    self.status.set_from_byte(status);
                }
                "ROL" => {
                    // Rotate Left
                    match opcode.addressing_mode {
                        AddressingMode::NoneAddressing => {
                            self.register_accumulator = self.rol(self.register_accumulator);
                        }
                        _ => {
                            let addr = self.get_operand_address(&opcode.addressing_mode);
                            let value = self.read_mem(addr);
                            let result = self.rol(value);
                            self.write_mem(addr, result);
                        }
                    }
                    self.status
                        .update_zero_and_negative_registers(self.register_accumulator);
                }
                "ROR" => {
                    // Rotate Right
                    match opcode.addressing_mode {
                        AddressingMode::NoneAddressing => {
                            self.register_accumulator = self.ror(self.register_accumulator);
                        }
                        _ => {
                            let addr = self.get_operand_address(&opcode.addressing_mode);
                            let value = self.read_mem(addr);
                            let result = self.ror(value);
                            self.write_mem(addr, result);
                        }
                    }
                    self.status
                        .update_zero_and_negative_registers(self.register_accumulator);
                }
                "RTI" => {
                    // Return From Interrupt
                    let status: u8 = self.stack_pull();
                    self.status.set_from_byte(status);
                    let pc: u16 = self.stack_pull_u16();
                    self.program_counter = pc;
                }
                "RTS" => self.program_counter = self.stack_pull_u16() + 1,
                "SBC" => {
                    // Subtract with carry
                    self.sbc(&opcode.addressing_mode);
                }
                "SEC" => self.status.set_flag(StatusFlag::Carry, true),
                "SED" => self.status.set_flag(StatusFlag::Decimal, true),
                "SEI" => self.status.set_flag(StatusFlag::InterruptDisable, true),
                "STA" => {
                    // Store Accumulator
                    self.sta(&opcode.addressing_mode);
                }
                "STX" => {
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    self.write_mem(addr, self.index_register_x);
                }
                "STY" => {
                    let addr = self.get_operand_address(&opcode.addressing_mode);
                    self.write_mem(addr, self.index_register_y);
                }
                "TAX" => {
                    // Transfer Accumulator to register X
                    self.index_register_x = self.register_accumulator;

                    self.status
                        .update_zero_and_negative_registers(self.index_register_x);
                }
                "TAY" => {
                    // Transfer Accumulator to register Y
                    self.index_register_y = self.register_accumulator;

                    self.status
                        .update_zero_and_negative_registers(self.index_register_y);
                }
                "TSX" => {
                    // Transfer Stack Pointer to X
                    self.index_register_x = self.stack_pointer;
                    self.status.update_zero_and_negative_registers(self.stack_pointer);
                },
                "TXA" => self.load_accumulator(self.index_register_x),
                "TXS" => {
                    // Transfer X to Stack Pointer
                    self.stack_pointer = self.index_register_x;
                    self.status.update_zero_and_negative_registers(self.index_register_x);
                },
                "TYA" => self.load_accumulator(self.index_register_y),

                _ => todo!(),
            }

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.bytes - 1) as u16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rom::ROM;
    use rstest::*;
    use super::*;

    #[fixture]
    pub fn cpu() -> CPU { 
        let bus = Bus::new(ROM::empty());
        let mut cpu = CPU::new(bus);
        cpu
    }


    #[rstest]
    fn test_0xa9_lda_immediate_load(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x42, 0x00]);
        assert_eq!(cpu.register_accumulator, 0x42);
        assert_eq!(cpu.status.status & 0b0000_0010, 0);
    }

    #[rstest]
    fn test_0xa9_lda_immediate_negative_flag(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xFF, 0x00]);
        assert_eq!(cpu.status.status & 0b1000_0000, 0b1000_0000);
    }

    #[rstest]
    fn test_0xa9_lda_zero_flag(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x00, 0x00]);
        assert_eq!(cpu.status.status & 0b0000_0010, 0b10);
    }

    #[rstest]
    fn test_5_ops_working_together(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xC0, 0xAA, 0xE8, 0x00]);

        assert_eq!(cpu.index_register_x, 0xC1)
    }

    #[rstest]
    fn test_inx_overflow(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xFF, 0xAA, 0xE8, 0xE8, 0x00]);

        assert_eq!(cpu.index_register_x, 1)
    }

    #[rstest]
    fn test_lda_from_memory(mut cpu: CPU) {
        cpu.write_mem(0x10, 0x55);
        cpu.load_and_execute(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_accumulator, 0x55);
    }

    #[rstest]
    fn test_sta(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xa9, 0x42, 0x85, 0x10]);
        assert_eq!(cpu.read_mem(0x10), 0x42);
    }

    #[rstest]
    fn test_adc(mut cpu: CPU) {
        cpu.write_mem(0x10, 0x55);
        // Immediate
        cpu.load_and_execute(vec![0xA9, 0x55, 0x69, 0x10]); // LDA 0x55, ADC 0x10
        assert_eq!(cpu.register_accumulator, 0x65);
        // Zero Page
        cpu.load_and_execute(vec![0xA9, 0x55, 0x65, 0x10]);
        assert_eq!(cpu.register_accumulator, 0xAA);
    }

    #[rstest]
    fn test_adc_carry(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xFF, 0x69, 0x10]);
        assert_eq!(cpu.register_accumulator, 0x0F);
        assert_eq!(cpu.status.status & 0b0100_0000, 0);
        assert_eq!(cpu.status.status & 0b0000_0001, 1); // Carry is 1
        cpu.load_and_execute(vec![0xA9, 0xFF, 0x69, 0x10, 0x69, 0x10]);
        assert_eq!(cpu.register_accumulator, 0x20);
        assert_eq!(cpu.status.status & 0b0100_0000, 0); // Overflow is 0
    }

    #[rstest]
    fn test_adc_overflow(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x50, 0x69, 0x50]);
        assert_eq!(cpu.register_accumulator, 0xA0);
        assert_eq!(cpu.status.status & 0b0100_0000, 0b0100_0000); // Overflow is 1
    }

    #[rstest]
    fn test_sbc(mut cpu: CPU) {
        cpu.write_mem(0x10, 0x55);
        // Immediate
        cpu.load_and_execute(vec![0xA9, 0x55, 0xE9, 0x10]); // LDA 0x55, SBC 0x10
        assert_eq!(cpu.register_accumulator, 0x44);
    }

    #[rstest]
    fn test_sbc_carry(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x00, 0xE9, 0x02]);
        assert_eq!(cpu.register_accumulator, 0xFD);
        cpu.load_and_execute(vec![0xE9, 0x02]);
        assert_eq!(cpu.status.status & 0b0000_0001, 1); // Carry is 1
        assert_eq!(cpu.register_accumulator, 0xFA);
    }

    #[rstest]
    fn test_get_operand_address_zero_page(mut cpu: CPU) {
        cpu.load_program(vec![0x10]);
        cpu.reset();
        let addr = cpu.get_operand_address(&AddressingMode::ZeroPage);
        assert_eq!(addr, 0x10);
    }

    #[rstest]
    fn test_php(mut cpu: CPU) {
        cpu.load_and_execute(vec![0x08]);
        assert_eq!(cpu.read_mem(0x1FFu16), 0b0011_0000);
    }

    #[rstest]
    fn test_pha(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xFA, 0x48]);
        assert_eq!(cpu.read_mem(0x1FF), 0xFA);
    }

    #[rstest]
    fn test_plp(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xFA, 0x48, 0x28]);
        assert_eq!(cpu.status.status, 0xFA);
    }

    #[rstest]
    fn test_rti(mut cpu: CPU) {
        cpu.load_and_execute(vec![
            0xA9, 0x81, 0x48, 0xA9, 0x02, 0x48, 0xA9, 0xFA, 0x48, 0x40,
        ]);
        assert_eq!(cpu.status.status, 0xFA);
        assert_eq!(cpu.program_counter, 0x0282)
    }

    #[rstest]
    fn test_and(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xFF, 0x29, 0b0110_1001]);
        assert_eq!(cpu.register_accumulator, 0b0110_1001)
    }

    #[rstest]
    fn test_asl_a(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xF0, 0x0A]);
        assert_eq!(cpu.register_accumulator, 0b1110_0000)
    }

    #[rstest]
    fn test_asl_mem(mut cpu: CPU) {
        cpu.write_mem(0x10, 0xF0);
        cpu.load_and_execute(vec![0x06, 0x10]);
        assert_eq!(cpu.read_mem(0x10), 0b1110_0000)
    }

    #[rstest]
    fn test_bcc(mut cpu: CPU) {
        cpu.load_and_execute(vec![0x90, 0x06, 0x00]);
        assert_eq!(cpu.program_counter, 0x8009)
    }

    #[rstest]
    fn test_bcs(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0xFF, 0x69, 0x10, 0xB0, 0x06, 0x00]);
        assert_eq!(cpu.program_counter, 0x800D)
    }

    #[rstest]
    fn test_bit(mut cpu: CPU) {
        cpu.write_mem(0x10, 0xFF);
        cpu.load_and_execute(vec![0xA9, 0x0, 0x24, 0x10]);
        assert_eq!(cpu.status.get_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.status.get_flag(StatusFlag::Overflow), false);
        assert_eq!(cpu.status.get_flag(StatusFlag::Negative), false);
        cpu.load_and_execute(vec![0xA9, 0b1100_0000, 0x24, 0x10]);
        assert_eq!(cpu.status.get_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.status.get_flag(StatusFlag::Overflow), true);
        assert_eq!(cpu.status.get_flag(StatusFlag::Negative), true);
    }

    #[rstest]
    fn test_clc(mut cpu: CPU) {
        cpu.load_program(vec![0x18]);
        cpu.reset();
        cpu.status.set_flag(StatusFlag::Carry, true);
        cpu.execute();
        assert_eq!(cpu.status.get_flag(StatusFlag::Carry), false);
    }

    #[rstest]
    fn test_cld(mut cpu: CPU) {
        cpu.load_program(vec![0xD8]);
        cpu.reset();
        cpu.status.set_flag(StatusFlag::Decimal, true);
        cpu.execute();
        assert_eq!(cpu.status.get_flag(StatusFlag::Decimal), false);
    }

    #[rstest]
    fn test_cli(mut cpu: CPU) {
        cpu.load_program(vec![0x58]);
        cpu.reset();
        cpu.status.set_flag(StatusFlag::InterruptDisable, true);
        cpu.execute();
        assert_eq!(cpu.status.get_flag(StatusFlag::InterruptDisable), false);
    }

    #[rstest]
    fn test_clv(mut cpu: CPU) {
        cpu.load_program(vec![0xB8]);
        cpu.reset();
        cpu.status.set_flag(StatusFlag::Overflow, true);
        cpu.execute();
        assert_eq!(cpu.status.get_flag(StatusFlag::Overflow), false);
    }

    #[rstest]
    fn test_cmp(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x42, 0xC9, 0x42]);
        assert_eq!(cpu.status.get_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.status.get_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.status.get_flag(StatusFlag::Negative), false);

        cpu.load_and_execute(vec![0xA9, 0x43, 0xC9, 0x42]);
        assert_eq!(cpu.status.get_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.status.get_flag(StatusFlag::Carry), true);
        assert_eq!(cpu.status.get_flag(StatusFlag::Negative), false);

        cpu.load_and_execute(vec![0xA9, 0x42, 0xC9, 0xC2]);
        assert_eq!(cpu.status.get_flag(StatusFlag::Zero), false);
        assert_eq!(cpu.status.get_flag(StatusFlag::Carry), false);
        assert_eq!(cpu.status.get_flag(StatusFlag::Negative), true);
    }

    #[rstest]
    fn test_dec(mut cpu: CPU) {
        cpu.write_mem(0x10, 0x43);
        cpu.load_and_execute(vec![0xC6, 0x10]);
        assert_eq!(cpu.read_mem(0x10), 0x42);
    }

    #[rstest]
    fn test_eor(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x10, 0x49, 0x10]);
        assert_eq!(cpu.register_accumulator, 0x00);
        assert_eq!(cpu.status.get_flag(StatusFlag::Zero), true);
        assert_eq!(cpu.status.get_flag(StatusFlag::Negative), false);
    }

    #[rstest]
    fn test_inc(mut cpu: CPU) {
        cpu.write_mem(0x10, 0x41);
        cpu.load_and_execute(vec![0xE6, 0x10]);
        assert_eq!(cpu.read_mem(0x10), 0x42);
    }

    #[rstest]
    fn test_jmp(mut cpu: CPU) {
        // Absolute
        cpu.load_and_execute(vec![0x4C, 0xFD, 0xCA]);
        assert_eq!(cpu.program_counter, 0xCAFE);
        // Indirect
        cpu.write_mem_u16(0xCAFE, 0xCADA);
        cpu.load_and_execute(vec![0x6C, 0xFE, 0xCA]);
        assert_eq!(cpu.program_counter, 0xCADB);
        // Indirect with page boundary bug
        cpu.write_mem(0x0000, 0x40);
        cpu.write_mem(0x00FF, 0x50);
        cpu.write_mem(0x0100, 0x30);
        cpu.load_and_execute(vec![0x6C, 0xFF, 0x00]);
        assert_eq!(cpu.program_counter, 0x4051);
    }

    #[rstest]
    fn test_stack_u16(mut cpu: CPU) {
        cpu.stack_push_u16(0xCAFE);
        assert_eq!(cpu.stack_pull_u16(), 0xCAFE);
    }

    #[rstest]
    fn test_jsr(mut cpu: CPU) {
        cpu.load_and_execute(vec![0x20, 0xFD, 0xCA]);
        assert_eq!(cpu.stack_pull_u16(), 0x8002);
        assert_eq!(cpu.program_counter, 0xCAFE);
    }

    #[rstest]
    fn test_ldx(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA2, 0x42]);
        assert_eq!(cpu.index_register_x, 0x42);
    }

    #[rstest]
    fn test_ldy(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA0, 0x42]);
        assert_eq!(cpu.index_register_y, 0x42);
    }

    #[rstest]
    fn test_lsr(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0b1110_0011, 0x4A]);
        assert_eq!(cpu.register_accumulator, 0b0111_0001);
        assert_eq!(cpu.status.get_flag(StatusFlag::Carry), true);
    }

    #[rstest]
    fn test_ora(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0b0110_0110, 0x09, 0b1001_1000]);
        assert_eq!(cpu.register_accumulator, 0b1111_1110);
    }

    #[rstest]
    fn test_pla(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x42, 0x48, 0xA9, 0x10, 0x68]);
        assert_eq!(cpu.register_accumulator, 0x42);
    }

    #[rstest]
    fn test_rol(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0b1000_0010, 0x2A]);
        assert_eq!(cpu.register_accumulator, 0b_0000_0101);
        assert_eq!(cpu.status.get_flag(StatusFlag::Carry), true);
    }

    #[rstest]
    fn test_ror(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0b1000_0011, 0x6A]);
        assert_eq!(cpu.register_accumulator, 0b1100_0001);
        assert_eq!(cpu.status.get_flag(StatusFlag::Carry), true);
    }

    #[rstest]
    fn test_rts(mut cpu: CPU) {
        cpu.load_and_execute(vec![0x20, 0xFD, 0xCA, 0x60]);
        assert_eq!(cpu.program_counter, 0xCAFE);
    }

    #[rstest]
    fn test_stx(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA2, 0x42, 0x8E, 0xFA, 0xFA]);
        assert_eq!(cpu.read_mem_u16(0xFAFA), 0x42);
    }

    #[rstest]
    fn test_sty(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA0, 0x42, 0x8C, 0xFA, 0xFA]);
        assert_eq!(cpu.read_mem_u16(0xFAFA), 0x42);
    }

    #[rstest]
    fn test_tax(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x42, 0xAA, 0x00]);
        assert_eq!(cpu.index_register_x, 0x42);
    }
    #[rstest]
    fn test_tay(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA9, 0x42, 0xA8]);
        assert_eq!(cpu.index_register_y, 0x42);
    }

    #[rstest]
    fn test_txa(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA2, 0x42, 0x8A]);
        assert_eq!(cpu.register_accumulator, 0x42);
    }

    #[rstest]
    fn test_tya(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA0, 0x42, 0x98]);
        assert_eq!(cpu.register_accumulator, 0x42);
    }

    #[rstest]
    fn test_tsx(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xBA]);
        assert_eq!(cpu.index_register_x, 0xFF);
        cpu.load_and_execute(vec![0xA9, 0x41, 0x48, 0xBA]);
        assert_eq!(cpu.index_register_x, 0xFE);
    }

    #[rstest]
    fn test_txs(mut cpu: CPU) {
        cpu.load_and_execute(vec![0xA2, 0x42, 0x9A]);
        assert_eq!(cpu.stack_pointer, 0x42);
    }
}
