use std::fmt;

use crate::rom::Rom;
use crate::Instruction;
use crate::MEMORY_START;
use crate::U4;
use crate::{
    display::Display,
    memory::{Memory, MemoryAddress},
};

pub struct VariableRegisters {
    registers: [u8; 16],
}

impl VariableRegisters {
    fn new() -> Self {
        VariableRegisters { registers: [0; 16] }
    }

    fn set_value(&mut self, register: U4, value: u8) {
        let idx = *register as usize;
        self.registers[idx] = value;
    }

    fn add_value(&mut self, register: U4, value: u8) {
        let idx = *register as usize;
        self.registers[idx] = self.registers[idx].wrapping_add(value);
    }

    pub fn get_value(&self, register: U4) -> u8 {
        let idx = *register as usize;
        self.registers[idx]
    }
}

impl fmt::Debug for VariableRegisters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("VariableRegisteres");
        for (index, value) in self.registers.iter().enumerate() {
            let reg_name = format!("V{:1X}", index).to_string();
            dbg.field(&reg_name, &format_args!("0x{:0>2X}", value));
        }

        dbg.finish()
    }
}

pub struct Cpu {
    pub display: Display,
    pub program_counter: MemoryAddress,
    pub index: MemoryAddress,
    stack: Vec<MemoryAddress>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub registers: VariableRegisters,
    pub memory: Memory,
}

impl Cpu {
    pub fn from_rom(rom: Rom) -> Self {
        Cpu {
            display: Display::new(),
            program_counter: MEMORY_START,
            index: MemoryAddress::from_u16(0),
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: VariableRegisters::new(),
            memory: Memory::from_rom(rom),
        }
    }

    pub fn tick(&mut self) {
        let instruction = self.fetch_instruction();

        match instruction {
            Instruction::AddValue { register, value } => self.registers.add_value(register, value),
            Instruction::ClearScreen => self.display.clear(),
            Instruction::Draw {
                register1,
                register2,
                sprite_length,
            } => self.handle_draw_instruction(register1, register2, sprite_length),
            Instruction::Jump(address) => self.program_counter.set(address),
            Instruction::SetIndex(new_index) => self.index.set(new_index),
            Instruction::SetValue { register, value } => self.registers.set_value(register, value),
        }
    }

    fn fetch_instruction(&mut self) -> Instruction {
        let instruction = self.memory.read_instruction(self.program_counter);
        let instruction = Instruction::try_from_u16(instruction);

        self.program_counter.increment();

        return instruction.unwrap();
    }

    fn handle_draw_instruction(&mut self, x_register: U4, y_register: U4, sprite_length: U4) {
        let x_pos = self.registers.get_value(x_register);
        let y_pos = self.registers.get_value(y_register);
        let sprite = self
            .memory
            .read_slice(self.index, usize::from(sprite_length));
        self.display.draw(x_pos, y_pos, sprite);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correctly_set_index_register() {
        let instructions = vec![0xA234];
        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom);

        println!("{:0>4X?}", cpu.program_counter);
        println!("{:X?}", cpu.memory.read_slice(MEMORY_START, 4));
        cpu.tick();

        assert_eq!(usize::from(cpu.index), 0x234);
    }

    #[test]
    fn correctly_set_value_registers() {
        let registers = 0..16;
        let values = 16..32;

        let instructions = registers
            .clone()
            .zip(values.clone())
            .map(|(reg, value)| (0x6 << 12) + (reg << 8) + value)
            .collect::<Vec<_>>();
        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom);

        for (index, (reg, value)) in registers.zip(values).enumerate() {
            cpu.tick();
            println!("Iteration {:0>2}: {:?}", index, cpu.registers);

            let register_value = cpu.registers.get_value(U4::new(reg as u8)) as u16;
            assert_eq!(
                value, register_value,
                "Expected {:0>4X} but got {:0>4X}",
                value, register_value
            );
        }
    }

    #[test]
    fn correctly_add_value_to_registers() {
        let registers = 0..16;
        let start_values = 16..32;
        let add_values = 32..48;

        // for each register define 2 instructions:
        // set register, add to register
        let instructions = registers
            .clone()
            .zip(start_values.clone())
            .zip(add_values.clone())
            .flat_map(|((reg, start_val), add_val)| {
                vec![
                    (0x6 << 12) + (reg << 8) + start_val,
                    (0x7 << 12) + (reg << 8) + add_val,
                ]
            })
            .collect::<Vec<_>>();
        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom);

        for (index, ((reg, start_value), value)) in
            registers.zip(start_values).zip(add_values).enumerate()
        {
            cpu.tick();
            cpu.tick();

            println!("Iteration {:0>2}: {:?}", index, cpu.registers);

            let actual_value = cpu.registers.get_value(U4::new(reg as u8)) as u16;
            let expected_value = start_value + value;
            assert_eq!(
                expected_value, actual_value,
                "Expected {:0>4X} but got {:0>4X}",
                expected_value, actual_value
            );
        }
    }
}
