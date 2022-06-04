use core::time;
use std::{
    collections::HashMap,
    env, fs,
    io::stdout,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
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

    let keyboard_mapping = HashMap::from([
        (KeyCode::Char('1'), 0),
        (KeyCode::Char('2'), 1),
        (KeyCode::Char('3'), 2),
        (KeyCode::Char('4'), 3),
        (KeyCode::Char('q'), 4),
        (KeyCode::Char('w'), 5),
        (KeyCode::Char('e'), 6),
        (KeyCode::Char('r'), 7),
        (KeyCode::Char('a'), 8),
        (KeyCode::Char('s'), 9),
        (KeyCode::Char('d'), 10),
        (KeyCode::Char('f'), 11),
        (KeyCode::Char('z'), 12),
        (KeyCode::Char('x'), 13),
        (KeyCode::Char('c'), 14),
        (KeyCode::Char('v'), 15),
    ]);

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

        emulator.clear_keyboard();
        if poll_for_keyboard_input(&mut emulator, &keyboard_mapping, thread_sleep_duration).is_err()
        {
            break;
        }
    }
}

fn poll_for_keyboard_input(
    emulator: &mut Chip8,
    keyboard_mapping: &HashMap<KeyCode, usize>,
    duration: Duration,
) -> Result<(), ()> {
    // Set raw mode so we can detect input without requiring Enter to be pressed.
    terminal::enable_raw_mode().expect("To enable raw mode.");

    let start = Instant::now();
    let mut duration_since_start = Instant::now().duration_since(start);

    while duration_since_start < duration {
        if event::poll(duration - duration_since_start).expect("Failed to poll.") {
            if let Event::Key(event) = event::read().expect("Failed to read line.") {
                // Check for key combinations for terminating the application.
                if (event.code == KeyCode::Char('c') || event.code == KeyCode::Char('z'))
                    && event.modifiers == event::KeyModifiers::CONTROL
                {
                    terminal::disable_raw_mode().expect("To disable raw mode.");
                    return Err(());
                }

                let index = keyboard_mapping.get(&event.code);
                if index.is_some() {
                    emulator.set_keyboard_key(*index.unwrap(), true);
                }
            };
        }

        duration_since_start = Instant::now().duration_since(start);
    }

    terminal::disable_raw_mode().expect("To disable raw mode.");
    Ok(())
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
