use anyhow::{anyhow, Result};
use std::fmt::Display;

use crate::{
    bits::{join_to_u16, join_to_u8, split_instruction, split_u16, split_u8, U4},
    memory::MemoryAddress,
};

pub enum Instruction {
    AddValue {
        register: U4,
        value: u8,
    },
    CallSubroutine(MemoryAddress),
    ClearScreen,
    Draw {
        register1: U4,
        register2: U4,
        sprite_length: U4,
    },
    Jump(u16),
    SetIndex(u16),
    SetValue {
        register: U4,
        value: u8,
    },
    SetValuesFromMemory {
        register: U4,
    },
    SkipIfEqual {
        register: U4,
        value: u8,
    },
}

impl Instruction {
    pub fn try_from_u16(raw_instruction: u16) -> Result<Self> {
        let (n1, n2, n3, n4) = split_instruction(raw_instruction);
        let res = match (*n1, *n2, *n3, *n4) {
            (0x0, 0x0, 0xE, 0x0) => Self::ClearScreen,
            (0x0, _, _, _) => Err(anyhow!(
                "Recieved machine instruction 0x{:0>4X} that cannot be handled",
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
            (0x6, _, _, _) => Self::SetValue {
                register: n2,
                value: join_to_u8(n3, n4),
            },
            (0x7, _, _, _) => Self::AddValue {
                register: n2,
                value: join_to_u8(n3, n4),
            },
            (0xA, _, _, _) => Self::SetIndex(join_to_u16(n2, n3, n4)),
            (0xD, _, _, _) => Self::Draw {
                register1: n2,
                register2: n3,
                sprite_length: n4,
            },
            (0xF, _, 0x6, 0x5) => Self::SetValuesFromMemory { register: n2 },
            (_, _, _, _) => Err(anyhow!(
                "Found invalid instruction {:#04x}",
                raw_instruction
            ))?,
        };

        Ok(res)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Instruction::AddValue { register, value } => {
                write!(f, "ADD V{:X}, {:0>2X}", **register, *value)
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
            Instruction::SetIndex(idx) => write!(f, "LD I, {:0>4X}", idx),
            Instruction::SetValue { register, value } => {
                write!(f, "LD V{:X}, {:0>2X}", **register, value)
            }
            Instruction::SetValuesFromMemory { register } => write!(f, "LD V{:X}, [I]", **register),
            Instruction::SkipIfEqual { register, value } => {
                write!(f, "SK V{:X}, {:0>2X}", **register, value)
            }
        }
    }
}
