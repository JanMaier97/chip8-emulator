mod memory;

use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use bits::U4;
use cpu::Cpu;
use display::Display;
use egui_extras::{Column, TableBuilder};
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

#[derive(PartialEq)]
enum CpuExecution {
    Paused,
    Running,
}

struct UiState {
    cpu: Cpu,
    execution: CpuExecution,
    current_rom: String,
    has_failed: bool,
    has_ticked: bool,
    output: Vec<String>,
    memory_filter: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            cpu: Cpu::default(),
            execution: CpuExecution::Paused,
            current_rom: "".to_string(),
            has_failed: true,
            has_ticked: false,
            output: Vec::new(),
            memory_filter: "".to_string(),
        }
    }
}

impl UiState {
    fn load_rom(&mut self, rom_path: &str) {
        self.clear_output();
        let rom =
            Rom::from_file(rom_path).with_context(|| format!("Failed reading rom '{}'", rom_path));

        self.handle_result(&rom);
        let Ok(rom) = rom else {
            return;
        };

        let cpu = Cpu::from_rom(rom)
            .with_context(|| format!("Failed loading rom '{}' into memory", rom_path));

        self.handle_result(&cpu);
        let Ok(cpu) = cpu else {
            return;
        };

        *self = Self {
            cpu,
            has_failed: false,
            current_rom: rom_path.to_string(),
            has_ticked: true,
            ..Default::default()
        };
    }

    fn restart(&mut self) {
        self.load_rom(&self.current_rom.clone());
        self.execution = CpuExecution::Paused;
    }

    fn handle_tick(&mut self) {
        let res = self.cpu.tick();
        self.has_ticked = true;
        self.handle_result(&res);
    }

    fn handle_result<T>(&mut self, result: &Result<T>) {
        if let Err(ref err) = result {
            self.output.push(format!("{:?}", err));
            self.has_failed = true;
        }
    }

    fn is_paused(&self) -> bool {
        !self.has_failed && self.execution == CpuExecution::Paused
    }

    fn is_running(&self) -> bool {
        !self.has_failed && self.execution == CpuExecution::Running
    }

    fn can_restart(&self) -> bool {
        self.current_rom != ""
    }

    fn clear_output(&mut self) {
        self.output.clear();
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let roms = vec![
        "./roms/ibm-logo.ch8",
        "./roms/SCTEST.ch8",
        "./roms/bc_test.ch8",
        "./roms/xxx.ch8",
    ];
    let mut state = UiState::default();

    loop {
        clear_background(RED);

        if state.is_running() {
            state.handle_tick();
        }

        draw_screen(&state.cpu.display);

        egui_macroquad::ui(|egui_ctx| {
            egui::SidePanel::right("Instructions")
                .exact_width(400.0)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    ui.separator();
                    draw_degubbing_controlls(ui, &mut state);
                    ui.separator();
                    draw_instructions(ui, &mut state);
                    ui.separator();
                    draw_register_grid(ui, &state.cpu);
                    ui.separator();
                    draw_stack(ui, &state.cpu);
                });

            egui::SidePanel::left("Roms")
                .exact_width(400.0)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    ui.separator();
                    draw_roms(ui, &mut state, &roms);
                    ui.separator();
                    draw_output(ui, &mut state);
                });

            egui::TopBottomPanel::bottom("Memory")
                .exact_height(400.0)
                .show(egui_ctx, |ui| {
                    draw_memory_grid(ui, &mut state);
                });
        });

        egui_macroquad::draw();

        next_frame().await
    }
}

fn draw_memory_grid(ui: &mut egui::Ui, state: &mut UiState) {
    let step = 16;
    let bytes = state
        .cpu
        .memory
        .read_slice(MemoryAddress::from_u16(0), MEMORY_SIZE)
        .unwrap();
    let rows_of_bytes = bytes.chunks(16);
    let mut byte_search = Vec::new();

    let parse_result = handle_byte_search_conversion(&state.memory_filter);
    let text_color = match parse_result {
        Ok(_) => None,
        Err(_) => Some(egui::Color32::RED),
    };

    if parse_result.is_ok() {
        byte_search = parse_result.unwrap();
    }

    let byte_indexes_to_highlight = compute_byte_indexes_to_highlight(&byte_search, bytes);

    ui.separator();
    ui.horizontal(|ui| {
        let text_edit = egui::TextEdit::singleline(&mut state.memory_filter)
            .desired_width(120.0)
            .text_color_opt(text_color);
        ui.label("Search:");
        ui.add(text_edit);
    });

    ui.separator();
    egui::Grid::new("memory_header")
        .num_columns(19)
        // .spacing([40.0, 4.0])
        .min_col_width(0.)
        .striped(true)
        .show(ui, |ui| {
            ui.monospace("      ");
            for i in 0..step {
                ui.monospace(format!("{:0>2X}", i));
                if i == 7 {
                    ui.label("");
                }
            }
            ui.monospace("");
            ui.end_row();
        });
    egui::ScrollArea::vertical()
        .max_height(300.)
        .show(ui, |ui| {
            egui::Grid::new("memory")
                .num_columns(18)
                // .spacing([40.0, 4.0])
                .min_col_width(0.)
                .striped(true)
                .show(ui, |ui| {
                    for (row_idx, bytes) in rows_of_bytes.enumerate() {
                        ui.monospace(format!("0x{:0>4X}", row_idx * step));
                        for (col_idx, b) in bytes.iter().enumerate() {
                            let bg_color = if byte_indexes_to_highlight
                                .contains(&(row_idx * step + col_idx))
                            {
                                egui::Color32::YELLOW
                            } else {
                                egui::Color32::TRANSPARENT
                            };
                            let text = egui::RichText::new(format!("{:0>2X}", b))
                                .monospace()
                                .background_color(bg_color);

                            ui.monospace(text);
                            if col_idx == 7 {
                                ui.label("");
                            }
                        }
                        let txt = bytes.iter().map(|b| byte_to_char(*b)).collect::<String>();
                        ui.monospace(txt);
                        ui.end_row()
                    }
                });
        });
    ui.separator();
}

fn compute_byte_indexes_to_highlight(expanded_search: &[Vec<u8>], bytes: &[u8]) -> HashSet<usize> {
    if expanded_search.len() == 0 {
        HashSet::new()
    } else {
        let window_len = expanded_search[0].len();
        expanded_search
            .into_iter()
            .flat_map(|search| {
                bytes
                    .windows(window_len)
                    .enumerate()
                    .filter(|(_, window)| *window == search)
                    .flat_map(|(idx, _)| idx..(idx + window_len))
                    .collect::<HashSet<_>>()
            })
            .collect::<HashSet<_>>()
    }
}

fn handle_byte_search_conversion(input: &str) -> Result<Vec<Vec<u8>>> {
    let expanded_values = expand_byte_search(input);

    let value = expanded_values
        .iter()
        .map(|v| parse_byte_search(v))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .collect::<Vec<_>>();

    Ok(value)
}

fn parse_byte_search(value: &str) -> Result<Vec<u8>> {
    if value == "" {
        return Ok(vec![]);
    }

    if value.chars().any(|c| c != '?' && !c.is_ascii_hexdigit()) {
        return Err(anyhow!("Invalid hex character"));
    }

    let value = if value.len() % 2 != 0 {
        format! {"0{}", value}
    } else {
        value.to_string()
    };

    let reversed = value
        .chars()
        .rev()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|chars| {
            u8::from_str_radix(format!("{}{}", chars[1], chars[0]).as_str(), 16).with_context(|| "")
        })
        .rev()
        .collect::<Result<Vec<u8>>>()?;

    let mut single_char_value = value
        .chars()
        .rev()
        .take(value.len() % 2)
        .map(|c| u8::from_str_radix(&c.to_string(), 16).with_context(|| ""))
        .collect::<Result<Vec<_>>>()?;

    single_char_value.extend(reversed);
    return Ok(single_char_value);
}

fn expand_byte_search(value: &str) -> Vec<String> {
    if value == "" {
        return vec![];
    }
    if !value.contains('?') {
        return vec![value.to_string()];
    }

    let mut result = vec![value.to_string()];
    for _ in value.chars().filter(|c| *c == '?') {
        let mut temp = Vec::new();
        for value in result.iter() {
            for x in 0..16_u8 {
                let replaced = value.replacen('?', format!("{:X}", x).as_str(), 1);
                temp.push(replaced);
            }
        }
        result = temp;
    }

    result
}

fn byte_to_char(byte: u8) -> char {
    if !byte.is_ascii() {
        return char::from(byte);
    }

    if byte.is_ascii_graphic() {
        return char::from(byte);
    }

    '.'
}

fn draw_register_grid(ui: &mut egui::Ui, cpu: &Cpu) {
    ui.heading("Registers");
    egui::Grid::new("registers")
        .num_columns(4)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| draw_register_grid_content(ui, &cpu));
}

fn draw_stack(ui: &mut egui::Ui, cpu: &Cpu) {
    ui.heading("Stack");
    egui::Grid::new("stack")
        .num_columns(2)
        .spacing([40.0, 4.0])
        .striped(true)
        .show(ui, |ui| {
            for (index, address) in cpu.stack.iter().enumerate() {
                ui.label(format!("{:>2}", index));
                ui.label(format!("{:0>4}", **address));
                ui.end_row();
            }
        });
}

fn draw_degubbing_controlls(ui: &mut egui::Ui, state: &mut UiState) {
    ui.horizontal(|ui| {
        ui.add_enabled_ui(state.is_paused(), |ui| {
            if ui.button("Step").clicked() {
                state.handle_tick();
            }
        });

        ui.add_enabled_ui(!state.has_failed, |ui| match state.execution {
            CpuExecution::Paused => {
                if ui.button("Continue").clicked() {
                    state.execution = CpuExecution::Running;
                }
            }
            CpuExecution::Running => {
                if ui.button("Pause").clicked() {
                    state.execution = CpuExecution::Paused;
                }
            }
        });

        ui.add_enabled_ui(state.can_restart(), |ui| {
            if ui.button("Restart").clicked() {
                state.restart();
            }
        });
    });
}

fn draw_roms(ui: &mut egui::Ui, state: &mut UiState, roms: &[&str]) {
    ui.heading("Roms");
    for rom in roms {
        if ui.button(*rom).clicked() {
            state.load_rom(rom);
        };
    }
}

fn draw_instructions(ui: &mut egui::Ui, state: &mut UiState) {
    let start = MemoryAddress::from_u16(0);
    let instructions = state
        .cpu
        .memory
        .read_slice(start, MEMORY_SIZE)
        .unwrap()
        .chunks(2)
        .map(|c| join_bytes(c[0], c[1]))
        .collect::<Vec<_>>();

    let start = usize::from(start);
    ui.heading("Instructions");
    let text_height = egui::TextStyle::Body
        .resolve(ui.style())
        .size
        .max(ui.spacing().interact_size.y);
    let total_rows = instructions.len();

    let mut table = TableBuilder::new(ui)
        .striped(true)
        .max_scroll_height(text_height * 22.)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(20.))
        .column(Column::exact(60.))
        .column(Column::exact(60.))
        .column(Column::remainder());

    if state.has_ticked {
        table = table.scroll_to_row(
            *state.cpu.program_counter as usize / 2,
            Some(egui::Align::TOP),
        );
        state.has_ticked = false;
    }

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.monospace("");
            });
            header.col(|ui| {
                ui.monospace("Address");
            });
            header.col(|ui| {
                ui.monospace("Value");
            });
            header.col(|ui| {
                ui.monospace("OpCode");
            });
        })
        .body(|body| {
            body.rows(text_height, total_rows, |row_index, mut row| {
                let raw_instruction = instructions[row_index];
                let current_address = start + 2 * row_index;
                row.col(|ui| {
                    if current_address == usize::from(state.cpu.program_counter) {
                        ui.label("=>");
                    } else {
                        ui.label("");
                    }
                });
                row.col(|ui| {
                    ui.monospace(format!("0x{:0>4X}", current_address));
                });

                row.col(|ui| {
                    ui.monospace(format!("0x{:0>4X}", raw_instruction));
                });
                row.col(|ui| {
                    if let Ok(instruction) = Instruction::try_from_u16(raw_instruction) {
                        ui.monospace(format!("{}", instruction));
                    } else {
                        ui.monospace("???");
                    }
                });
            });
        });
}

fn draw_register_grid_content(ui: &mut egui::Ui, cpu: &Cpu) {
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

fn draw_output(ui: &mut egui::Ui, state: &UiState) {
    ui.heading("Output");
    for line in state.output.iter() {
        ui.label(line);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_search_expanded_correctly() {
        assert_eq!(vec![vec![6]], handle_byte_search_conversion("6").unwrap());
        assert_eq!(
            vec![vec![0x76]],
            handle_byte_search_conversion("76").unwrap()
        );
        assert_eq!(
            vec![vec![0x7, 0x68]],
            handle_byte_search_conversion("768").unwrap()
        );
        assert_eq!(
            vec![
                vec![0x0],
                vec![0x1],
                vec![0x2],
                vec![0x3],
                vec![0x4],
                vec![0x5],
                vec![0x6],
                vec![0x7],
                vec![0x8],
                vec![0x9],
                vec![0xA],
                vec![0xB],
                vec![0xC],
                vec![0xD],
                vec![0xE],
                vec![0xF],
            ],
            handle_byte_search_conversion("?").unwrap()
        );
        assert_eq!(
            vec![
                vec![0xA0],
                vec![0xA1],
                vec![0xA2],
                vec![0xA3],
                vec![0xA4],
                vec![0xA5],
                vec![0xA6],
                vec![0xA7],
                vec![0xA8],
                vec![0xA9],
                vec![0xAA],
                vec![0xAB],
                vec![0xAC],
                vec![0xAD],
                vec![0xAE],
                vec![0xAF],
            ],
            handle_byte_search_conversion("A?").unwrap()
        );
    }

    #[test]
    fn compute_byte_indexes_to_highlight_correclty_finds_indexes() {
        let instructions = vec![0x6500, 0x6402];

        let cpu = Cpu::from_rom(Rom::from_raw_instructions(&instructions)).unwrap();

        let bytes = cpu.memory.read_slice(MEMORY_START, 10).unwrap();

        let filter = "2";
        let search = handle_byte_search_conversion(filter).unwrap();
        let res = Vec::from_iter(compute_byte_indexes_to_highlight(&search, bytes));
        assert_eq!(vec![3], res);

        let filter = "64";
        let search = handle_byte_search_conversion(filter).unwrap();
        let res = Vec::from_iter(compute_byte_indexes_to_highlight(&search, bytes));
        assert_eq!(vec![2], res);

        let filter = "6402";
        let search = handle_byte_search_conversion(filter).unwrap();
        let mut res = Vec::from_iter(compute_byte_indexes_to_highlight(&search, bytes));
        res.sort();
        assert_eq!(vec![2, 3], res);

        let filter = "6?";
        let search = handle_byte_search_conversion(filter).unwrap();
        let mut res = Vec::from_iter(compute_byte_indexes_to_highlight(&search, bytes));
        res.sort();
        assert_eq!(vec![0, 2], res);

        let filter = "6?0?";
        let search = handle_byte_search_conversion(filter).unwrap();
        let mut res = Vec::from_iter(compute_byte_indexes_to_highlight(&search, bytes));
        res.sort();
        assert_eq!(vec![0, 1, 2, 3], res);
    }
}
