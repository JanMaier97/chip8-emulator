use anyhow::{anyhow, Result};
use std::ops::{Deref, Index, IndexMut};

use crate::{bits::U4, rom::Rom};

pub const MEMORY_START: MemoryAddress = MemoryAddress(0x200);
pub const MEMORY_SIZE: usize = 4096;

const SINGLE_FONT_BYTE_COUNT: u16 = 5;

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
    pub fn from_u16(value: u16) -> Self {
        MemoryAddress(value)
    }

    pub fn increment(&mut self) {
        self.0 += 2;
    }

    pub fn set(&mut self, value: u16) {
        self.0 = value;
    }

    pub fn add(&self, value: u16) -> MemoryAddress {
        MemoryAddress(self.0 + value)
    }
}

impl From<MemoryAddress> for usize {
    fn from(value: MemoryAddress) -> Self {
        return value.0 as usize;
    }
}

impl Deref for MemoryAddress {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Memory {
    data: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: [0; MEMORY_SIZE],
        }
    }

    pub fn from_rom(rom: Rom) -> Result<Self> {
        let rom_start = MEMORY_START.0 as usize;
        if rom.data.len() > MEMORY_SIZE - rom_start {
            return Err(anyhow!(
                "Rom data exceeds the memory limit. Allowed: {:0>4X}, Actual: {:0>4X}",
                MEMORY_SIZE - rom_start,
                rom.data.len()
            ));
        }

        let mut data = [0; MEMORY_SIZE];

        for (index, value) in FONT_DATA.iter().enumerate() {
            data[index] = *value;
        }

        for (index, rom_value) in rom.data.into_iter().enumerate() {
            data[rom_start + index] = rom_value;
        }

        Ok(Memory { data })
    }

    pub fn get_address_for_font(&self, value: U4) -> MemoryAddress {
        // only consider last nible
        let raw_address = *value as u16 * SINGLE_FONT_BYTE_COUNT;
        MemoryAddress(raw_address)
    }

    pub fn read_instruction(&self, address: MemoryAddress) -> u16 {
        let upper = self.data[address.0 as usize] as u16;
        let lower = self.data[(address.0 + 1) as usize] as u16;

        return (upper << 8) + lower;
    }

    pub fn write_slice(&mut self, start: MemoryAddress, bytes: &[u8]) -> Result<()> {
        let start = usize::from(start);
        if start + bytes.len() > MEMORY_SIZE {
            return Err(anyhow!(
                "Trying to write {} bytes at address {:0>4X} which excees valid memory",
                bytes.len(),
                start
            ));
        }

        for (offset, byte) in bytes.iter().enumerate() {
            self.data[start + offset] = *byte;
        }

        Ok(())
    }

    pub fn read_slice(&self, start: MemoryAddress, length: usize) -> Result<&[u8]> {
        let start = start.0 as usize;
        if start + length > MEMORY_SIZE {
            return Err(anyhow!(
                "Memory out of range: Cannot access memory in range 0x{:0>4X}-0x{:0>4X}",
                start,
                start + length
            ));
        }

        Ok(&self.data[start..start + length])
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
