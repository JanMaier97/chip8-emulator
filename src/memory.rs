use std::ops::{Index, IndexMut};

use crate::{bits::U4, rom::Rom};

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

#[derive(Clone, Copy, Debug)]
pub struct MemoryAddress(u16);

impl MemoryAddress {
    pub fn from_byte(value: u8) -> Self {
        MemoryAddress(value as u16)
    }

    pub fn increment(&mut self) {
        self.0 += 2;
    }

    pub fn set(&mut self, value: u16) {
        self.0 = value;
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
    pub fn from_rom(rom: Rom) -> Self {
        let rom_start = MEMORY_START.0 as usize;
        if rom.data.len() > MEMORY_SIZE - rom_start {
            panic!("Rom is too large")
        }

        let mut data = [0; MEMORY_SIZE];

        for (index, value) in FONT_DATA.iter().enumerate() {
            data[index] = *value;
        }

        for (index, rom_value) in rom.data.into_iter().enumerate() {
            data[rom_start + index] = rom_value;
        }

        Memory { data }
    }

    pub fn read_instruction(&self, address: MemoryAddress) -> u16 {
        let upper = self.data[address.0 as usize] as u16;
        let lower = self.data[(address.0 + 1) as usize] as u16;

        return (upper << 8) + lower;
    }

    pub fn read_slice(&self, start: MemoryAddress, length: U4) -> &[u8] {
        let start = start.0 as usize;
        let length = *length as usize;
        if start + length > MEMORY_SIZE {
            panic!(
                "Trying to access memory in range {}-{}, which is invalid",
                start,
                start + length
            )
        }

        &self.data[start..start + length]
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
