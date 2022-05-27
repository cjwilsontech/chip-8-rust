use std::{env, fs};

use crate::emulator::Chip8;

mod emulator;

fn main() {
    println!("CHIP-8");

    let args: Vec<String> = env::args().collect();
    let rom_path = args.get(1).expect("Expected a path to the ROM to open.");
    let data = fs::read(rom_path).expect("Unable to read file");

    let mut emulator = Chip8::new();
    let load_rom_result = emulator.load(data);
    if load_rom_result.is_err() {
        panic!("Error loading data: {}", load_rom_result.unwrap_err());
    }

    emulator.cycle();
}
