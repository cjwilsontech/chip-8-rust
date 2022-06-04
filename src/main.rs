use core::time;
use std::{env, fs, io::stdout, thread};

use crossterm::{
    cursor,
    terminal::{self, ClearType},
    ExecutableCommand,
};
use emulator::{Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH};

mod emulator;

const CLOCK_RATE: u32 = 500;

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

    let thread_sleep_duration = time::Duration::from_secs(1) / CLOCK_RATE;

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();
    sink.pause();
    sink.append(rodio::source::SineWave::new(400.0));

    loop {
        emulator.cycle();

        if emulator.should_play_sound() {
            if sink.is_paused() {
                sink.play();
            }
        } else if !sink.is_paused() {
            sink.pause();
        }

        thread::sleep(thread_sleep_duration);
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
