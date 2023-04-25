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
}
