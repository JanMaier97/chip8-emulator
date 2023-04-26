mod memory;

use memory::{Memory, MemoryAddress, MEMORY_START};

fn main() {
    println!("Hello, world!");
}

struct VariableRegisters {
    v0: u8,
    v1: u8,
    v2: u8,
    v3: u8,
    v4: u8,
    v5: u8,
    v6: u8,
    v7: u8,
    v8: u8,
    v9: u8,
    va: u8,
    vb: u8,
    vc: u8,
    vd: u8,
    ve: u8,
    vf: u8,
}

impl VariableRegisters {
    fn new() -> Self {
        VariableRegisters {
            v0: 0,
            v1: 0,
            v2: 0,
            v3: 0,
            v4: 0,
            v5: 0,
            v6: 0,
            v7: 0,
            v8: 0,
            v9: 0,
            va: 0,
            vb: 0,
            vc: 0,
            vd: 0,
            ve: 0,
            vf: 0,
        }
    }
}

struct CPU {
    program_counter: MemoryAddress,
    index: MemoryAddress,
    stack: Vec<MemoryAddress>,
    delay_timer: u8,
    sound_timer: u8,
    registers: VariableRegisters,
    memory: Memory,
}

impl CPU {
    fn new() -> Self {
        CPU {
            program_counter: MEMORY_START,
            index: MemoryAddress::from_byte(0),
            stack: Vec::new(),
            delay_timer: 0,
            sound_timer: 0,
            registers: VariableRegisters::new(), 
            memory: Memory::new(),
        }
    }

    fn run(&mut self) {
        let instruction = self.fetch_instruction();
    }

    fn exeucte(&mut self, instruction: u16)  {
        let (n1, n2, n3, n4) = self.split_instruction(instruction);
        match (n1, n2, n3, n4) {
            (0x0, _, _, _) => println!("Ignoring machine language instruction"),
            (0x0, 0x0, 0xE, 0x0) => todo!("clear screen"),
            (0x0, 0x0, 0xE, 0xE) => todo!("call subroutine"),
            (0x1, _, _, _) => todo!("Jump"),
            (0x2, _, _, _) => todo!("return from subroutine"),
            (0x3, _, _, _) => todo!("skip"),
            (0x4, _, _, _) => todo!("skip"),
            (0x5, _, _, 0x0) => todo!("skip"),
            (0x6, _, _, _) => todo!("set vx"),
            (0x7, _, _, _) => todo!("add vx"),
            (0x8, _, _, 0x0) => todo!("set vy"),
            (0x8, _, _, 0x1) => todo!("set binary or"),
            (0x8, _, _, 0x2) => todo!("set binary and"),
            (0x8, _, _, 0x3) => todo!("set logical xor"),
            (0x8, _, _, 0x4) => todo!("add vx and vy"),
            (0x8, _, _, 0x5) => todo!("subtract"),
            (0x8, _, _, 0x6) => todo!("shift"),
            (0x8, _, _, 0x7) => todo!("subtract"),
            (0x8, _, _, 0xE) => todo!("shift"),
            (0x9, _, _, 0x0) => todo!("skip"),
            (0xA, _, _, _) => todo!("set index"),
            (0xB, _, _, _) => todo!("Jump with offset"),
            (0xD, _, _, _) => todo!("Display"),
            (0xF, _, 0x0, 0xA) => todo!("get key"),
            (0xF, _, 0x2, 0x9) => todo!("font character"),
            (0xE, _, 0xA, 0x1) => todo!("skip if key"),
            (0xE, _, 0x9, 0xE) => todo!("skip if key"),
            (0xF, _, 0x0, 0x7) => todo!("timer"),
            (0xF, _, 0x1, 0x5) => todo!("timer"),
            (0xF, _, 0x1, 0x8) => todo!("timer"),
            (0xF, _, 0x1, 0xE) => todo!("Add to index"),
            (0xF, _, 0x3, 0x3) => todo!("decimal conversion"),
            (0xF, _, 0x5, 0x5) => todo!("store memory"),
            (0xF, _, 0x6, 0x5) => todo!("load memory"),
            (_, _, _, _) => panic!("Found invalid instruction {:#04x}", instruction)
        };
    }

    fn fetch_instruction(&mut self) -> u16 {
        let instruction = self.memory.read_instruction(self.program_counter);
        self.program_counter.increment();

        return instruction;
    }

    fn split_instruction(&self, instruction: u16) -> (u8, u8, u8, u8) {
        let (upper_byte, lower_byte) = self.split_u16(instruction);
        let upper_nibbles = self.split_u8(upper_byte);
        let lower_nibbles = self.split_u8(lower_byte);
        
        return (upper_nibbles.0, upper_nibbles.1, lower_nibbles.0, lower_nibbles.1);
    }

    fn split_u16(&self, value: u16) -> (u8, u8) {
        let upper = (value & 0b11111111_00000000) >> 8;
        let lower = value & 0b11111111;

        (upper as u8, lower as u8)
    }

    fn split_u8(&self, value: u8) -> (u8, u8) {
        let upper = (value & 0b1111_0000) >> 4;
        let lower = value & 0b1111;

        (upper, lower)
    }

}
