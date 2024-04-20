use anyhow::{anyhow, Result};
use std::fmt::Display;

use crate::{
    bits::{join_to_u16, join_to_u8, split_instruction, split_u16, split_u8, U4},
    memory::MemoryAddress,
};

#[derive(Clone, Copy)]
pub enum Instruction {
    AddValue {
        register: U4,
        value: u8,
    },
    AddRegisterToIndex {
        register: U4,
    },
    AddRegisters {
        register1: U4,
        register2: U4,
    },
    And {
        register1: U4,
        register2: U4,
    },
    CallSubroutine(MemoryAddress),
    ClearScreen,
    Draw {
        register1: U4,
        register2: U4,
        sprite_length: U4,
    },
    Jump(u16),
    JumpWithOffset(u16),
    LoadDelayTimer {
        register: U4,
    },
    LoadFont {
        register: U4,
    },
    LoadRegistersFromMemory {
        register: U4,
    },
    LoadRegisterFromRegister {
        register1: U4,
        register2: U4,
    },
    LoadSoundTimer {
        register: U4,
    },
    Or {
        register1: U4,
        register2: U4,
    },
    Random {
        register: U4,
        mask: u8,
    },
    Return,
    SetIndex(u16),
    SetValue {
        register: U4,
        value: u8,
    },
    ShiftLeft {
        register1: U4,
        register2: U4,
    },
    ShiftRight {
        register1: U4,
        register2: U4,
    },
    StoreBcdRepresentation {
        register: U4,
    },
    SubRegisters {
        register1: U4,
        register2: U4,
    },
    SubRegistersReversed {
        register1: U4,
        register2: U4,
    },
    SkipIfEqual {
        register: U4,
        value: u8,
    },
    SkipIfEqualRegisters {
        register1: U4,
        register2: U4,
    },
    SkipNotEqualByte {
        register: U4,
        value: u8,
    },
    SkipNotEqualRegisters {
        register1: U4,
        register2: U4,
    },
    WriteRegistersToMemory {
        register: U4,
    },
    Xor {
        register1: U4,
        register2: U4,
    },
}

impl Instruction {
    pub fn try_from_u16(raw_instruction: u16) -> Result<Self> {
        let (n1, n2, n3, n4) = split_instruction(raw_instruction);
        let res = match (*n1, *n2, *n3, *n4) {
            (0x0, 0x0, 0xE, 0x0) => Self::ClearScreen,
            (0x0, 0x0, 0xE, 0xE) => Self::Return,
            (0x0, _, _, _) => Err(anyhow!(
                "Unsupported instruction 0x{:0>4X} System call",
                raw_instruction
            ))?,
            (0x1, _, _, _) => Self::Jump(join_to_u16(n2, n3, n4)),
            (0x2, _, _, _) => {
                Self::CallSubroutine(MemoryAddress::from_u16(join_to_u16(n2, n3, n4)))
            }
            (0x3, _, _, _) => Self::SkipIfEqual {
                register: n2,
                value: join_to_u8(n3, n4),
            },
            (0x4, _, _, _) => Self::SkipNotEqualByte {
                register: n2,
                value: join_to_u8(n3, n4),
            },
            (0x5, _, _, 0) => Self::SkipIfEqualRegisters {
                register1: n2,
                register2: n3,
            },
            (0x6, _, _, _) => Self::SetValue {
                register: n2,
                value: join_to_u8(n3, n4),
            },
            (0x7, _, _, _) => Self::AddValue {
                register: n2,
                value: join_to_u8(n3, n4),
            },
            (0x8, _, _, 0x0) => Self::LoadRegisterFromRegister {
                register1: n2,
                register2: n3,
            },
            (0x8, _, _, 0x1) => Self::Or {
                register1: n2,
                register2: n3,
            },
            (0x8, _, _, 0x2) => Self::And {
                register1: n2,
                register2: n3,
            },
            (0x8, _, _, 0x3) => Self::Xor {
                register1: n2,
                register2: n3,
            },
            (0x8, _, _, 0x4) => Self::AddRegisters {
                register1: n2,
                register2: n3,
            },
            (0x8, _, _, 0x5) => Self::SubRegisters {
                register1: n2,
                register2: n3,
            },
            (0x8, _, _, 0x6) => Self::ShiftRight {
                register1: n2,
                register2: n3,
            },
            (0x8, _, _, 0x7) => Self::SubRegistersReversed {
                register1: n2,
                register2: n3,
            },
            (0x8, _, _, 0xE) => Self::ShiftLeft {
                register1: n2,
                register2: n3,
            },
            (0x9, _, _, 0x0) => Self::SkipNotEqualRegisters {
                register1: n2,
                register2: n3,
            },
            (0xA, _, _, _) => Self::SetIndex(join_to_u16(n2, n3, n4)),
            (0xB, _, _, _) => Self::JumpWithOffset(join_to_u16(n2, n3, n4)),
            (0xC, _, _, _) => Self::Random {
                register: n2,
                mask: join_to_u8(n3, n4),
            },
            (0xD, _, _, _) => Self::Draw {
                register1: n2,
                register2: n3,
                sprite_length: n4,
            },
            (0xF, _, 0x1, 0x5) => Self::LoadDelayTimer { register: n2 },
            (0xF, _, 0x1, 0x8) => Self::LoadSoundTimer { register: n2 },
            (0xF, _, 0x1, 0xE) => Self::AddRegisterToIndex { register: n2 },
            (0xF, _, 0x2, 0x9) => Self::LoadFont { register: n2 },
            (0xF, _, 0x3, 0x3) => Self::StoreBcdRepresentation { register: n2 },
            (0xF, _, 0x5, 0x5) => Self::WriteRegistersToMemory { register: n2 },
            (0xF, _, 0x6, 0x5) => Self::LoadRegistersFromMemory { register: n2 },
            (_, _, _, _) => Err(anyhow!("Invalid instruction 0x{:0>4X}", raw_instruction))?,
        };

        Ok(res)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Instruction::AddRegisterToIndex { register } => write!(f, "ADD I, V{:X})", **register),
            Instruction::And {
                register1,
                register2,
            } => {
                write!(f, "AND V{:X}, V{:X}", **register1, **register2)
            }
            Instruction::AddValue { register, value } => {
                write!(f, "ADD V{:X}, {:0>2X}", **register, *value)
            }
            Instruction::AddRegisters {
                register1,
                register2,
            } => {
                write!(f, "ADD V{:X}, V{:X}", **register1, **register2)
            }
            Instruction::CallSubroutine(addr) => write!(f, "CALL {:0>4X}", **addr),
            Instruction::ClearScreen => write!(f, "CLS"),
            Instruction::Draw {
                register1,
                register2,
                sprite_length,
            } => write!(
                f,
                "DRW V{:X}, V{:X}, {:X}",
                **register1, **register2, **sprite_length
            ),
            Instruction::Jump(address) => write!(f, "JP {:0>4X}", address),
            Instruction::JumpWithOffset(address) => write!(f, "JP V0, {:0>4X}", address),
            Instruction::LoadDelayTimer { register } => write!(f, "LD DT, V{:X}", **register),
            Instruction::LoadFont { register } => write!(f, "LD F, V{:x}", **register),
            Instruction::LoadRegistersFromMemory { register } => {
                write!(f, "LD V{:X}, [I]", **register)
            }
            Instruction::LoadRegisterFromRegister {
                register1,
                register2,
            } => write!(f, "LD V{:X}, V{:X}", **register1, **register2),
            Instruction::LoadSoundTimer { register } => write!(f, "LD ST, V{:X}", **register),
            Instruction::Or {
                register1,
                register2,
            } => write!(f, "OR V{:X}, V{:X}", **register1, **register2),
            Instruction::Random { register, mask } => {
                write!(f, "RND V{:X}, {:0>2X}", **register, mask)
            }
            Instruction::Return => write!(f, "RET"),
            Instruction::SetIndex(idx) => write!(f, "LD I, {:0>4X}", idx),
            Instruction::SetValue { register, value } => {
                write!(f, "LD V{:X}, {:0>2X}", **register, value)
            }
            Instruction::WriteRegistersToMemory { register } => {
                write!(f, "LD [I], V{:X}", **register)
            }
            Instruction::SkipIfEqual { register, value } => {
                write!(f, "SE V{:X}, {:0>2X}", **register, value)
            }
            Instruction::SkipIfEqualRegisters {
                register1,
                register2,
            } => {
                write!(f, "SE V{:X}, V{:X}", **register1, **register2)
            }
            Instruction::SkipNotEqualByte { register, value } => {
                write!(f, "SNE V{:X}, {:0>2X}", **register, value)
            }
            Instruction::SkipNotEqualRegisters {
                register1,
                register2,
            } => {
                write!(f, "SNE V{:X}, {:X}", **register1, **register2)
            }
            Instruction::ShiftLeft {
                register1,
                register2,
            } => {
                write!(f, "SHL V{:X} {{, V{:X}}}", **register1, **register2)
            }
            Instruction::ShiftRight {
                register1,
                register2,
            } => {
                write!(f, "SHR V{:X} {{, V{:X}}}", **register1, **register2)
            }
            Instruction::StoreBcdRepresentation { register } => {
                write!(f, "LD B, V{:X}", **register)
            }
            Instruction::SubRegisters {
                register1,
                register2,
            } => write!(f, "SUB V{:X}, V{:X}", **register1, **register2),
            Instruction::SubRegistersReversed {
                register1,
                register2,
            } => write!(f, "SUBN V{:X}, V{:X}", **register1, **register2),
            Instruction::Xor {
                register1,
                register2,
            } => write!(f, "XOR V{:X}, V{:X}", **register1, **register2),
        }
    }
}
