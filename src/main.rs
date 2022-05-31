use std::{env, fs};

use emulator::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

use crate::emulator::Chip8;

mod emulator;

fn main() {
    println!("CHIP-8");

    let args: Vec<String> = env::args().collect();
    let rom_path = args.get(1).expect("Expected a path to the ROM to open.");
    let data = fs::read(rom_path).expect("Unable to read file");

    let mut emulator = Chip8::new(draw_screen);
    let load_rom_result = emulator.load(data);
    if load_rom_result.is_err() {
        panic!("Error loading data: {}", load_rom_result.unwrap_err());
    }

    loop {
        emulator.cycle();
    }
}

fn draw_screen(display: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {
    for row in 0..DISPLAY_HEIGHT {
        for col in 0..DISPLAY_WIDTH {
            print!(
                "{}",
                if display[row * DISPLAY_WIDTH + col] {
                    '\u{2588}'
                } else {
                    ' '
                }
            );
        }
        println!();
    }
}
