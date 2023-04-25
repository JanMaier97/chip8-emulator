use std::ops::{Index, IndexMut};

pub const MEMORY_START: MemoryAddress = MemoryAddress(0x200);

const MEMORY_SIZE: usize = 4096;

const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct MemoryAddress(u16);

impl MemoryAddress {
    pub fn from_byte(value: u8) -> Self {
        MemoryAddress(value as u16)
    }
}

impl From<MemoryAddress> for usize {
    fn from(value: MemoryAddress) -> Self {
        return value.0 as usize;
    }
}

pub struct Memory {
    data: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        let mut data = [0; MEMORY_SIZE];

        for (index, value) in FONT_DATA.iter().enumerate() {
            data[index] = *value;
        }

        Memory {
            data,
        }
    }
}

impl Index<MemoryAddress> for Memory {
    type Output = u8;

    fn index(&self, index: MemoryAddress) -> &Self::Output {
        &self.data[index.0 as usize]
    }
}

impl IndexMut<MemoryAddress> for Memory {
    fn index_mut(&mut self, index: MemoryAddress) -> &mut Self::Output {
        &mut self.data[usize::from(index)]
    }
}
