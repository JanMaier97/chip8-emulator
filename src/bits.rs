use std::ops::Deref;

pub struct U4(u8);

impl U4 {
    pub fn new(value: u8) -> Self {
        if value > 0xF {
            panic!("Tried instancing u4 with value {}", value);
        }

        U4(value)
    }
}

impl Deref for U4 {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<U4> for usize {
    fn from(value: U4) -> Self {
        return value.0 as usize;
    }
}

pub fn split_u16(value: u16) -> (u8, u8) {
    let upper = (value & 0b11111111_00000000) >> 8;
    let lower = value & 0b11111111;

    (upper as u8, lower as u8)
}

pub fn split_u8(value: u8) -> (U4, U4) {
    let upper = (value & 0b1111_0000) >> 4;
    let lower = value & 0b1111;

    (U4::new(upper), U4::new(lower))
}

pub fn join_to_u16(n1: U4, n2: U4, n3: U4) -> u16 {
    let n1 = *n1 as u16;
    let n2 = *n2 as u16;
    let n3 = *n3 as u16;

    (n1 << 8) + (n2 << 4) + n3
}

pub fn join_bytes(b1: u8, b2: u8) -> u16 {
    let b1 = b1 as u16;
    let b2 = b2 as u16;
    (b1 << 8) + b2
}

pub fn join_to_u8(n1: U4, n2: U4) -> u8 {
    (*n1 << 4) + *n2
}

pub fn join_nibbles(n1: u8, n2: u8, n3: u8, n4: u8) -> u16 {
    let n1 = n1 as u16;
    let n2 = n2 as u16;
    let n3 = n3 as u16;
    let n4 = n4 as u16;
    (n1 << 12) + (n2 << 8) + (n3 << 4) + n4
}

pub fn split_instruction(instruction: u16) -> (U4, U4, U4, U4) {
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
