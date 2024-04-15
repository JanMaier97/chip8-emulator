use crate::bits::{join_to_u16, join_to_u8, split_u16, split_u8, U4};

pub enum Instruction {
    AddValue {
        register: U4,
        value: u8,
    },
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
}

impl Instruction {
    pub fn from_u16(raw_instruction: u16) -> Self {
        let (n1, n2, n3, n4) = split_instruction(raw_instruction);
        let res = match (*n1, *n2, *n3, *n4) {
            (0x0, 0x0, 0xE, 0x0) => Self::ClearScreen,
            (0x0, _, _, _) => panic!(
                "Recieved machine instruction 0x{:0>4X} that cannot be handled",
                raw_instruction
            ),
            (0x1, _, _, _) => Self::Jump(join_to_u16(n2, n3, n4)),
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
            (_, _, _, _) => panic!("Found invalid instruction {:#04x}", raw_instruction),
        };

        res
    }
}

fn split_instruction(instruction: u16) -> (U4, U4, U4, U4) {
    let (upper_byte, lower_byte) = split_u16(instruction);
    let upper_nibbles = split_u8(upper_byte);
    let lower_nibbles = split_u8(lower_byte);

    return (
        upper_nibbles.0,
        upper_nibbles.1,
        lower_nibbles.0,
        lower_nibbles.1,
    );
}
