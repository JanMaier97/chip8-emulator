mod memory;

use bits::U4;
use cpu::Cpu;
use display::Display;
use instruction::Instruction;
use memory::MEMORY_START;
use rom::Rom;

mod bits;
mod cpu;
mod display;
mod instruction;
mod rom;

fn main() {
    custom_rom();
    // ibm_rom();
}

fn custom_rom() {
    let instructions = vec![0xA000, 0xD005];

    let rom = Rom::from_raw_instructions(&instructions);
    let mut cpu = Cpu::from_rom(rom);

    cpu.tick();
    cpu.tick();
    // debug_display(&cpu.display);
    print_display(&cpu.display);
}

fn ibm_rom() {
    let file_path = "./roms/ibm-logo.ch8";
    let rom = Rom::from_file(file_path);
    let mut cpu = Cpu::from_rom(rom);

    let frequency: f64 = (1. / 900.) * 1000.;
    let duration = std::time::Duration::from_millis(frequency as u64);
    loop {
        cpu.tick();
        std::process::Command::new("clear").status().unwrap();
        print_display(&cpu.display);
        // debug_display(&cpu.display);
        std::thread::sleep(duration);
    }
}

fn debug_display(display: &Display) {
    println!("------------------------------------------------------------------");
    for row in display.pixels {
        println!("|{:0>64b}|", row);
    }
    println!("------------------------------------------------------------------");
}

fn print_display(display: &Display) {
    println!("------------------------------------------------------------------");
    for row in display.pixels {
        let mut pixel_mask = 1 << 63;
        print!("|");
        while pixel_mask > 0 {
            if (row & pixel_mask) > 0 {
                print!("#");
            } else {
                print!(" ")
            }

            pixel_mask = pixel_mask >> 1;
        }
        print!("|");
        println!();
    }
    println!("------------------------------------------------------------------");
}
