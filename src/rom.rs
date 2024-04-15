use std::io::BufReader;
use std::{fs::File, io::Read};

use crate::bits::{join_bytes, join_to_u16, split_u16};

pub struct Rom {
    pub data: Vec<u8>,
}

impl Rom {
    pub fn from_raw_instructions(data: &[u16]) -> Self {
        let mut rom_data = Vec::with_capacity(data.len() * 2);
        data.iter().map(|&i| split_u16(i)).for_each(|(b1, b2)| {
            rom_data.push(b1);
            rom_data.push(b2);
        });

        Self { data: rom_data }
    }

    pub fn from_file(file_path: &str) -> Self {
        let mut file = File::open(file_path).unwrap();

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        Self { data }
    }
}

impl std::fmt::Debug for Rom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_list();
        self.data
            .chunks(2)
            .map(|chunk| join_bytes(chunk[0], chunk[1]))
            .for_each(|value| {
                dbg.entry(&format_args!("0x{:0>4X}", value));
            });

        dbg.finish()
    }
}
