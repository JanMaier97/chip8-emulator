use anyhow::{Context, Result};
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
    pub stack: Vec<MemoryAddress>,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub registers: VariableRegisters,
    pub memory: Memory,
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu {
            display: Display::new(),
            program_counter: MEMORY_START,
            index: MemoryAddress::from_u16(0),
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: VariableRegisters::new(),
            memory: Memory::new(),
        }
    }
}

impl Cpu {
    pub fn from_rom(rom: Rom) -> Result<Self> {
        let cpu = Cpu {
            memory: Memory::from_rom(rom)?,
            ..Default::default()
        };

        Ok(cpu)
    }

    pub fn tick(&mut self) -> Result<()> {
        let instruction = self
            .fetch_instruction()
            .with_context(|| "Error while fetching new instruction")?;
        self.handle_instruction(instruction)
            .with_context(|| format!("Error executing {}", instruction))?;
        Ok(())
    }

    fn handle_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::AddValue { register, value } => self.registers.add_value(register, value),
            Instruction::CallSubroutine(addr) => {
                self.stack.push(self.program_counter);
                self.program_counter = addr;
            }
            Instruction::ClearScreen => self.display.clear(),
            Instruction::Draw {
                register1,
                register2,
                sprite_length,
            } => self.handle_draw_instruction(register1, register2, sprite_length)?,
            Instruction::Jump(address) => self.program_counter.set(address),
            Instruction::SetIndex(new_index) => self.index.set(new_index),
            Instruction::SetValue { register, value } => self.registers.set_value(register, value),
            Instruction::SetValuesFromMemory { register } => {
                // get as many bytes as registers need to be filled
                let bytes = self.memory.read_slice(self.index, *register as usize + 1)?;
                for (register, byte) in bytes.iter().enumerate() {
                    let register = U4::new(register as u8);
                    self.registers.set_value(register, *byte);
                }
            }
            Instruction::SkipIfEqual { register, value } => {
                if self.registers.get_value(register) == value {
                    self.program_counter;
                    self.program_counter.increment();
                    self.program_counter;
                }
            }
        }

        Ok(())
    }

    fn fetch_instruction(&mut self) -> Result<Instruction> {
        let instruction = self.memory.read_instruction(self.program_counter);
        let instruction = Instruction::try_from_u16(instruction).with_context(|| {
            format!("Error occoured at address 0x{:0>4X}", *self.program_counter)
        })?;

        self.program_counter.increment();

        return Ok(instruction);
    }

    fn handle_draw_instruction(
        &mut self,
        x_register: U4,
        y_register: U4,
        sprite_length: U4,
    ) -> Result<()> {
        let x_pos = self.registers.get_value(x_register);
        let y_pos = self.registers.get_value(y_register);
        let sprite = self
            .memory
            .read_slice(self.index, usize::from(sprite_length))?;
        self.display.draw(x_pos, y_pos, sprite);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::bits::{join_nibbles, join_to_u8, split_instruction, split_u8};

    use super::*;

    #[test]
    fn correctly_set_index_register() {
        let instructions = vec![0xA234];
        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        println!("{:0>4X?}", cpu.program_counter);
        println!("{:X?}", cpu.memory.read_slice(MEMORY_START, 4));
        cpu.tick().unwrap();

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
        let mut cpu = Cpu::from_rom(rom).unwrap();

        for (index, (reg, value)) in registers.zip(values).enumerate() {
            cpu.tick().unwrap();
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
        let mut cpu = Cpu::from_rom(rom).unwrap();

        for (index, ((reg, start_value), value)) in
            registers.zip(start_values).zip(add_values).enumerate()
        {
            cpu.tick().unwrap();
            cpu.tick().unwrap();

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

    #[test]
    fn correctly_handles_call_subroutine_instruction() {
        let raw_instructions = vec![0x2345_u16];
        let mut cpu = Cpu::from_rom(Rom::from_raw_instructions(&raw_instructions)).unwrap();

        let original_address = *cpu.program_counter;

        cpu.tick().unwrap();

        assert_eq!(
            0x345, *cpu.program_counter,
            "Program counter has not been set to {:X}, actual {:X}",
            0x345, *cpu.program_counter
        );
        assert_eq!(
            1,
            cpu.stack.len(),
            "Expected one address to be pushed to the stack"
        );
        assert_eq!(
            original_address + 2,
            *cpu.stack[0],
            "Address pushed to the stack is wrong"
        );
    }

    #[test]
    fn correctly_handle_skip_if_equal_instruction() {
        for register in 0..16 {
            let value = 0x24;
            let (v1, v2) = split_u8(value);

            let raw_instructions = vec![
                join_nibbles(0x6, register, *v1, *v2), // load value into register
                join_nibbles(0x3, register, 0, 0),     // compare register with 0x00
                join_nibbles(0x3, register, *v1, *v2), // compare register with correct value
            ];
            let mut cpu = Cpu::from_rom(Rom::from_raw_instructions(&raw_instructions)).unwrap();

            cpu.tick().unwrap();

            let original_address = *cpu.program_counter;
            cpu.tick().unwrap();
            assert_eq!(
                original_address + 2,
                *cpu.program_counter,
                "{:X}: Expected PC to increment normally and not skip ahead",
                register
            );

            let original_address = *cpu.program_counter;
            cpu.tick().unwrap();
            assert_eq!(
                original_address + 4,
                *cpu.program_counter,
                "{:X}: Expected PC to increment twice and skip one instruction",
                register
            );
        }
    }

    #[test]
    fn correctly_handle_fx65_load_memory_into_registers() {
        for current_register in 0..16 {
            let raw_instructions = vec![
                join_nibbles(0x6, 0x1, 0x0, 0x1), // load value into register
                join_nibbles(0x6, 0x2, 0x3, 0x2), // load value into register
                join_nibbles(0x6, 0x3, 0x2, 0x5), // load value into register
                join_nibbles(0x6, 0x4, 0x1, 0x3), // load value into register
                join_nibbles(0x6, 0x5, 0x1, 0x3), // load value into register
                join_nibbles(0x6, 0x6, 0x1, 0x3), // load value into register
                join_nibbles(0x6, 0x7, 0x1, 0x3), // load value into register
                join_nibbles(0xA, 0x2, 0x0, 0x0), // set index to rom start
                join_nibbles(0xF, current_register, 0x6, 0x5), // load memory into registers V0 till V<register>
            ];

            let memory = raw_instructions
                .iter()
                .map(|inst| split_instruction(*inst))
                .flat_map(|(n1, n2, n3, n4)| vec![join_to_u8(n1, n2), join_to_u8(n3, n4)])
                .collect::<Vec<_>>();
            let mut cpu = Cpu::from_rom(Rom::from_raw_instructions(&raw_instructions)).unwrap();

            raw_instructions.iter().for_each(|_| {
                cpu.tick().unwrap();
            });

            for reg in 0..=current_register {
                let register = U4::new(reg);
                let reg_value = cpu.registers.get_value(register);
                let expected_value = memory[reg as usize];

                assert_eq!(
                    expected_value,
                    reg_value,
                    "({:X}) Expected register {:X} to have loaded a value from memory",
                    raw_instructions.last().unwrap(),
                    reg
                );
            }
        }
    }
}
