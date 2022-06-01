use std::{env, fs, io::stdout};

use crossterm::{
    cursor,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use emulator::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};

mod emulator;

fn main() {
    stdout()
        .execute(terminal::Clear(ClearType::All))
        .unwrap()
        .execute(cursor::MoveTo(0, 0))
        .unwrap();

    println!("CHIP-8");
    stdout()
        .execute(cursor::SavePosition)
        .expect("To save cursor position.");

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
    stdout()
        .execute(cursor::RestorePosition)
        .expect("To restore cursor position.");
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
    println!();
}
