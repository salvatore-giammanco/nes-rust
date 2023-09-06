use crate::cpu::AddressingMode;
use std::collections::HashMap;

pub struct OpCode {
    pub opcode: u8,
    pub label: &'static str,
    pub bytes: u8,
    pub cycles: u16,
    pub addressing_mode: AddressingMode,
}

impl OpCode {
    fn new(
        opcode: u8,
        label: &'static str,
        bytes: u8,
        cycles: u16,
        addressing_mode: AddressingMode,
    ) -> Self {
        Self {
            opcode,
            label,
            bytes,
            cycles,
            addressing_mode,
        }
    }
}

lazy_static! {
    pub static ref CPU_OPCODES: Vec<OpCode> = vec![
        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NoneAddressing),
        OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA5, "LDA", 2, 2, AddressingMode::ZeroPage),
        OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBD, "LDA", 3, 4 /* +1 if page is crossed */, AddressingMode::Absolute_X),
        OpCode::new(0xB9, "LDA", 3, 4 /* +1 if page is crossed */, AddressingMode::Absolute_Y),
        OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xB1, "LDA", 2, 5 /* +1 if page is crossed */, AddressingMode::Indirect_Y),
        OpCode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x8D, "STA", 2, 4, AddressingMode::Absolute),
        OpCode::new(0x9D, "STA", 2, 5, AddressingMode::Absolute_X),
        OpCode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y),
        OpCode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),
        OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7D, "ADC", 3, 4 /* +1 if page is crossed */, AddressingMode::Absolute_X),
        OpCode::new(0x79, "ADC", 3, 4 /* +1 if page is crossed */, AddressingMode::Absolute_Y),
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0x71, "ADC", 2, 5 /* +1 if page is crossed */, AddressingMode::Indirect_Y),
        OpCode::new(0xE9, "SBC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE5, "SBC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xF5, "SBC", 2, 4, AddressingMode::ZeroPage_X),
        OpCode::new(0xED, "SBC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xFD, "SBC", 3, 4 /* +1 if page is crossed */, AddressingMode::Absolute_X),
        OpCode::new(0xF9, "SBC", 3, 4 /* +1 if page is crossed */, AddressingMode::Absolute_Y),
        OpCode::new(0xE1, "SBC", 2, 6, AddressingMode::Indirect_X),
        OpCode::new(0xF1, "SBC", 2, 5 /* +1 if page is crossed */, AddressingMode::Indirect_Y),
    ];

    pub static ref CPU_OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for op in CPU_OPCODES.iter() {
            map.insert(op.opcode, op);
        }
        map
    };
}

#[derive(Debug, Clone)]
pub struct OpCodeNotFound;
