use crate::U4;

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

pub struct Display {
    pub pixels: [u64; DISPLAY_HEIGHT],
}

impl Display {
    pub fn new() -> Self {
        Self {
            pixels: [0; DISPLAY_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [0; DISPLAY_HEIGHT];
    }

    pub fn draw(&mut self, x_pos: u8, y_pos: u8, sprite: &[u8]) -> bool {
        let x_pos = x_pos as usize % DISPLAY_WIDTH;
        let y_pos = y_pos as usize % DISPLAY_HEIGHT;

        let mut has_turned_of_any_pixel = false;
        for (row_idx, &sprite_row) in sprite.into_iter().enumerate() {
            let current_y = y_pos + row_idx;
            if current_y > DISPLAY_HEIGHT {
                break;
            }

            let shifted_sprite_row = self.shift_sprite_row(x_pos as u64, sprite_row as u64);
            if (shifted_sprite_row & self.pixels[current_y]) > 0 {
                has_turned_of_any_pixel = true;
            }

            self.pixels[current_y] = self.pixels[current_y] ^ shifted_sprite_row;
        }

        has_turned_of_any_pixel
    }

    fn shift_sprite_row(&self, x_pos: u64, sprite_row: u64) -> u64 {
        let pos = 64 - 8;
        if x_pos <= pos {
            return sprite_row << (pos - x_pos);
        }

        return sprite_row >> (x_pos - pos);
    }
}
