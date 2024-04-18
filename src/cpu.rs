use anyhow::{anyhow, Context, Result};
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

        self.program_counter.increment();

        self.handle_instruction(instruction)
            .with_context(|| format!("Error executing {}", instruction))?;

        Ok(())
    }

    fn handle_instruction(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::AddValue { register, value } => self.registers.add_value(register, value),
            Instruction::AddRegisters {
                register1,
                register2,
            } => {
                let value1 = self.registers.get_value(register1);
                let value2 = self.registers.get_value(register2);
                let (result, did_overflow) = value1.overflowing_add(value2);
                self.registers.set_value(register1, result);

                let flag_value = if did_overflow { 1 } else { 0 };
                self.registers.set_value(U4::new(0xF), flag_value);
            }
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
            Instruction::LoadFont { register } => {
                let value = self.registers.get_value(register);
                let value = U4::new(value & 0b00001111);
                self.index = self.memory.get_address_for_font(value);
            }
            Instruction::LoadRegisterFromRegister {
                register1,
                register2,
            } => {
                let value = self.registers.get_value(register2);
                self.registers.set_value(register1, value);
            }
            Instruction::Return => {
                let address = self.stack.pop().ok_or_else(|| {
                    anyhow!("Tried to pop an address from the stack, but stack is empty")
                })?;
                self.program_counter = address;
            }
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
            Instruction::ShiftLeft { register1, .. } => {
                let value = self.registers.get_value(register1);
                self.registers.set_value(register1, value << 1);
                self.registers.set_value(U4::new(0xF), value >> 7);
            }
            Instruction::ShiftRight { register1, .. } => {
                let value = self.registers.get_value(register1);
                self.registers.set_value(register1, value >> 1);
                self.registers.set_value(U4::new(0xF), value & 1);
            }
            Instruction::SkipIfEqual { register, value } => {
                if self.registers.get_value(register) == value {
                    self.program_counter.increment();
                }
            }
            Instruction::SkipNotEqualByte { register, value } => {
                if self.registers.get_value(register) != value {
                    self.program_counter.increment();
                }
            }
            Instruction::SubRegisters {
                register1,
                register2,
            } => {
                let value1 = self.registers.get_value(register1);
                let value2 = self.registers.get_value(register2);
                let result = self.handle_sub(value1, value2);
                self.registers.set_value(register1, result);
            }
            Instruction::SubRegistersReversed {
                register1,
                register2,
            } => {
                let value1 = self.registers.get_value(register1);
                let value2 = self.registers.get_value(register2);
                let result = self.handle_sub(value2, value1);
                self.registers.set_value(register1, result);
            }
            Instruction::StoreBcdRepresentation { register } => {
                let value = self.registers.get_value(register);
                let d0 = value / 100;
                let d1 = (value % 100) / 10;
                let d2 = value % 10;
                self.memory[self.index] = d0;
                self.memory[self.index.add(1)] = d1;
                self.memory[self.index.add(2)] = d2;
            }
            Instruction::Xor {
                register1,
                register2,
            } => {
                let value1 = self.registers.get_value(register1);
                let value2 = self.registers.get_value(register2);
                self.registers.set_value(register1, value1 ^ value2);
            }
        }

        Ok(())
    }

    fn handle_sub(&mut self, lhs: u8, rhs: u8) -> u8 {
        let flag_value = if lhs >= rhs { 1 } else { 0 };
        self.registers.set_value(U4::new(0xF), flag_value);
        lhs.wrapping_sub(rhs)
    }

    fn fetch_instruction(&mut self) -> Result<Instruction> {
        let instruction = self.memory.read_instruction(self.program_counter);
        let instruction = Instruction::try_from_u16(instruction).with_context(|| {
            format!("Error occoured at address 0x{:0>4X}", *self.program_counter)
        })?;

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
    fn correctly_handle_4xkk_skip_if_equal() {
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
    fn correctly_handles_4xkk_skip_not_equal() {
        for register in 0..16 {
            let value = 0x24;
            let (v1, v2) = split_u8(value);

            let raw_instructions = vec![
                join_nibbles(0x6, register, *v1, *v2), // load value into register
                join_nibbles(0x4, register, *v1, *v2), // compare register with correct value
                join_nibbles(0x4, register, 0, 0),     // compare register with 0x00
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

    #[test]
    fn correctly_handles_fx29_load_font() {
        let instructions = vec![
            0x6000, // load 0 into V0
            0x610F, // load F into V1
            0x62F5, // load F5 into V2, to load 5
            0xF129, // Load font using V1
            0xF029, // Load font using V0
            0xF229, // Load font using V2
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        cpu.tick().unwrap();
        assert_eq!(0x4B, *cpu.index);

        cpu.tick().unwrap();
        assert_eq!(0x0, *cpu.index);

        cpu.tick().unwrap();
        assert_eq!(0x19, *cpu.index);
    }

    #[test]
    fn correctly_handle_fx33_store_decimal_conversion() {
        let instructions = vec![
            0x60FE, // store 254 in V0
            0xA500, // set index to 0x500
            0xF033, // convert V0 value to decimal
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        let bytes = cpu.memory.read_slice(cpu.index, 3).unwrap();

        assert_eq!(2, bytes[0]);
        assert_eq!(5, bytes[1]);
        assert_eq!(4, bytes[2]);
    }

    #[test]
    fn correctly_handle_8xy4_add_registers() {
        let instructions = vec![
            0x6F01, // set vF to 0x1
            0x6012, // set v0 to 0x12
            0x6553, // set v5 to 0x53
            0x8054, // v0 + v5, no carry
            0x61FF, // set v1 to 0xFF
            0x6201, // set v2 to 0x01
            0x8124, // add v1 and v2, with otherflow
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0x12 + 0x53,
            cpu.registers.get_value(U4::new(0)),
            "Registers have not been added correctly."
        );
        assert_eq!(
            0,
            cpu.registers.get_value(U4::new(0xF)),
            "Flag register needs to be zero"
        );

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0,
            cpu.registers.get_value(U4::new(1)),
            "Addition needs to wrap when overflow happens"
        );
        assert_eq!(
            1,
            cpu.registers.get_value(U4::new(0xF)),
            "Flag register needs to be set to 1 if addition causes overflow"
        );
    }

    #[test]
    fn correctly_handle_8xy5_sub_registers() {
        let instructions = vec![
            0x6F00, // set flag register to 0
            0x6001, // set v0 to 0x1
            0x6501, // set v5 to 0x1
            0x8055, // v0 - v5
            0x6100, // set v1 to 0xFF
            0x6201, // set v2 to 0x01
            0x8125, // add v1 and v2, with otherflow
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0x0,
            cpu.registers.get_value(U4::new(0)),
            "Registers have not been subtracted correctly."
        );
        assert_eq!(
            1,
            cpu.registers.get_value(U4::new(0xF)),
            "Flag register needs to be set to 1, if no underflow happens"
        );

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0xFF,
            cpu.registers.get_value(U4::new(1)),
            "Subtractions needs to wrap when overflow happens"
        );
        assert_eq!(
            0,
            cpu.registers.get_value(U4::new(0xF)),
            "Flag register must be 0, if an underflow happens"
        );
    }

    #[test]
    fn correctly_handle_8xy7_sub_registers_reverse() {
        let instructions = vec![
            0x6F00, // set flag register to 0
            0x6001, // set v0 to 0x1
            0x6501, // set v5 to 0x1
            0x8057, // v5 - v0
            0x6101, // set v1 to 0xFF
            0x6200, // set v2 to 0x01
            0x8127, // subn v1 and v2, with otherflow
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0x0,
            cpu.registers.get_value(U4::new(0)),
            "Registers have not been subtracted correctly."
        );
        assert_eq!(
            1,
            cpu.registers.get_value(U4::new(0xF)),
            "Flag register needs to be set to 1, if no underflow happens"
        );

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0xFF,
            cpu.registers.get_value(U4::new(1)),
            "Subtractions needs to wrap when overflow happens"
        );
        assert_eq!(
            0,
            cpu.registers.get_value(U4::new(0xF)),
            "Flag register must be 0, if an underflow happens"
        );
    }

    #[test]
    fn correctly_handle_00ee_return_from_subroutine() {
        let instructions = vec![
            0x2204, // call subroutine
            0x6000, // set v0 to 0
            0x00EE, // return immediately
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(0, cpu.stack.len(), "Stack should have been empty");
        assert_eq!(
            0x202, *cpu.program_counter,
            "Program counter is at the wrong address after returning"
        );
    }

    #[test]
    fn correctly_handle_8xy0_load_register_from_register() {
        let instructions = vec![
            0x61E4, // set V1
            0x8310, // set V3 from V1
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0xE4,
            cpu.registers.get_value(U4::new(0x3)),
            "V3 should have the same value as V1"
        );
    }

    #[test]
    fn correctly_handle_8xy7_load_register_from_register() {
        let instructions = vec![
            0x61E4, // set V1
            0x8310, // set V3 from V1
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0xE4,
            cpu.registers.get_value(U4::new(0x3)),
            "V3 should have the same value as V1"
        );
    }

    #[test]
    fn correctly_handle_8xy6_shift_register_right() {
        let instructions = vec![
            0x63FF, // set V1
            0x61E1, // set V1
            0x8136, // right shift
            0x61E0, // set V1
            0x8136, // right shift
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0xE1 >> 1,
            cpu.registers.get_value(U4::new(0x1)),
            "V1 has not been shifted correctly"
        );
        assert_eq!(
            1,
            cpu.registers.get_value(U4::new(0xF)),
            "VF has to be 1 if a bit has been shifted out"
        );

        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0xE0 >> 1,
            cpu.registers.get_value(U4::new(0x1)),
            "V1 has not been shifted correctly"
        );
        assert_eq!(
            0,
            cpu.registers.get_value(U4::new(0xF)),
            "VF has to be 0 if a bit has not been shifted out"
        );
    }

    #[test]
    fn correctly_handle_8xye_shift_register_left() {
        let instructions = vec![
            0x63FF, // set V3
            0x6187, // set V1
            0x813E, // left shift
            0x6177, // set V1
            0x813E, // left shift
        ];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0x87 << 1,
            cpu.registers.get_value(U4::new(0x1)),
            "V1 has not been shifted correctly"
        );
        assert_eq!(
            1,
            cpu.registers.get_value(U4::new(0xF)),
            "VF has to be 1 if a bit has been shifted out"
        );

        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0x77 << 1,
            cpu.registers.get_value(U4::new(0x1)),
            "V1 has not been shifted correctly"
        );
        assert_eq!(
            0,
            cpu.registers.get_value(U4::new(0xF)),
            "VF has to be 0 if a bit has not been shifted out"
        );
    }

    #[test]
    fn correctly_handle_8xy3_xor_registers() {
        let instructions = vec![0x61EE, 0x62A3, 0x8123];

        let rom = Rom::from_raw_instructions(&instructions);
        let mut cpu = Cpu::from_rom(rom).unwrap();

        cpu.tick().unwrap();
        cpu.tick().unwrap();
        cpu.tick().unwrap();

        assert_eq!(
            0xEE ^ 0xA3,
            cpu.registers.get_value(U4::new(1)),
            "Register values have to be xor-ed and stored to Vx"
        );
    }
}
