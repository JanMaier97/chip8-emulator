mod memory;

use bits::U4;
use cpu::Cpu;
use display::Display;
use egui_macroquad::egui;
use instruction::Instruction;
use memory::{MemoryAddress, MEMORY_START};
use rom::Rom;

mod bits;
mod cpu;
mod display;
mod instruction;
mod rom;

use macroquad::prelude::*;

use crate::{bits::join_bytes, memory::MEMORY_SIZE};

#[macroquad::main(window_conf)]
async fn main() {
    let roms = vec![
        "./roms/ibm-logo.ch8",
        "./roms/SCTEST.ch8",
        "./roms/bc_test.ch8",
    ];
    let mut cpu = Cpu::from_rom(Rom::from_file(roms[0]));

    loop {
        clear_background(RED);
        draw_screen(&cpu.display);

        egui_macroquad::ui(|egui_ctx| {
            egui::SidePanel::right("Instructions")
                .exact_width(400.0)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    draw_degubbing_controlls(ui, &mut cpu);

                    draw_instructions(ui, &cpu);
                    ui.separator();
                    egui::Grid::new("registers")
                        .num_columns(4)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| draw_register_ui(ui, &cpu));
                });

            egui::SidePanel::left("Roms")
                .exact_width(400.0)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    draw_roms(ui, &mut cpu, &roms);
                });

            egui::TopBottomPanel::bottom("Memory")
                .exact_height(400.0)
                .show(egui_ctx, |ui| {});
        });

        egui_macroquad::draw();

        next_frame().await
    }
}

fn draw_degubbing_controlls(ui: &mut egui::Ui, cpu: &mut Cpu) {
    ui.horizontal(|ui| {
        if ui.button("Step").clicked() {
            cpu.tick();
        }
    });
}

fn draw_roms(ui: &mut egui::Ui, cpu: &mut Cpu, roms: &[&str]) {
    for rom in roms {
        if ui.button(*rom).clicked() {
            *cpu = Cpu::from_rom(Rom::from_file(&rom));
        };
    }
}

fn draw_instructions(ui: &mut egui::Ui, cpu: &Cpu) {
    const HALF_COUNT: u16 = 10;
    const STEP: u16 = 2;

    const MEM_OFFSET: u16 = HALF_COUNT * STEP - STEP;

    let center_address = (MEM_OFFSET)
        .max(*cpu.program_counter)
        .min(MEMORY_SIZE as u16 - MEM_OFFSET);
    let start = MemoryAddress::from_u16(center_address - MEM_OFFSET);

    let instructions = cpu
        .memory
        .read_slice(start, (HALF_COUNT * 4).into())
        .chunks(2)
        .map(|c| join_bytes(c[0], c[1]))
        .collect::<Vec<_>>();

    let start = usize::from(start);
    egui::Grid::new("instructions")
        .num_columns(4)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("");
            ui.label("Address");
            ui.label("Value");
            ui.label("OpCode");
            ui.end_row();
            for (index, raw_instruction) in instructions.iter().enumerate() {
                let current_address = start + STEP as usize * index;
                if current_address == usize::from(cpu.program_counter) {
                    ui.label("=>");
                } else {
                    ui.label("");
                }

                ui.label(format!("0x{:0>4X}", current_address));
                ui.label(format!("0x{:0>4X}", raw_instruction));
                if let Ok(instruction) = Instruction::try_from_u16(*raw_instruction) {
                    ui.label(format!("{}", instruction));
                } else {
                    ui.label("???");
                }

                ui.end_row();
            }
        });
}

fn draw_register_ui(ui: &mut egui::Ui, cpu: &Cpu) {
    ui.label("PC:");
    ui.label(format!("{:0>4X}", *cpu.program_counter));

    ui.label("I:");
    ui.label(format!("{:0>4X}", *cpu.index));

    ui.end_row();

    ui.label("V0:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(0))));

    ui.label("V1:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(1))));

    ui.end_row();

    ui.label("V2:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(2))));

    ui.label("V3:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(3))));

    ui.end_row();

    ui.label("V4:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(4))));

    ui.label("V5:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(5))));

    ui.end_row();

    ui.label("V6:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(6))));

    ui.label("V7:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(7))));

    ui.end_row();

    ui.label("DT:");
    ui.label(format!("{:0>4X}", cpu.delay_timer));

    ui.label("ST:");
    ui.label(format!("{:0>4X}", cpu.sound_timer));

    ui.end_row();

    ui.label("V8:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(8))));

    ui.label("V9:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(9))));

    ui.end_row();

    ui.label("VA:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(10))));

    ui.label("VB:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(11))));

    ui.end_row();

    ui.label("VC:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(12))));

    ui.label("VD:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(13))));

    ui.end_row();

    ui.label("VE:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(14))));

    ui.label("VF:");
    ui.label(format!("{:0>4X}", cpu.registers.get_value(U4::new(15))));
}

fn draw_screen(display: &Display) {
    const PIXEL_SIZE: f32 = 16.;
    const X_OFFSET: f32 = 448.;
    const Y_OFFSET: f32 = 84.;

    // draw_line(0., 0., 64);
    for (row_index, row) in display.pixels.iter().enumerate() {
        let mut pixel_mask = 1 << 63;
        let mut column_index = 0;

        while pixel_mask > 0 {
            let x_pos = column_index as f32 * PIXEL_SIZE + X_OFFSET;
            let y_pos = row_index as f32 * PIXEL_SIZE + Y_OFFSET;

            if (row & pixel_mask) > 0 {
                draw_rectangle(x_pos, y_pos, PIXEL_SIZE, PIXEL_SIZE, WHITE);
            } else {
                draw_rectangle(x_pos, y_pos, PIXEL_SIZE, PIXEL_SIZE, BLACK);
            }

            column_index += 1;
            pixel_mask = pixel_mask >> 1;
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Chip8 Emulator".to_owned(),
        window_height: 1080,
        window_width: 1920,
        ..Default::default()
    }
}
