use crate::emulator::Chip8;

mod emulator;

fn main() {
    println!("CHIP-8");

    let mut emulator = Chip8::new();
    emulator.cycle();
}
