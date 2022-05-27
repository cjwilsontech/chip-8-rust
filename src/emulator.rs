pub struct Chip8 {
    reg_pc: u16,
    reg_sp: u16,
    reg_i: u16,
    reg_timer_delay: u8,
    reg_timer_sound: u8,
    reg_v: [u8; 16],

    stack: [u16; 16],
    memory: [u8; 4096],
    keyboard: [bool; 16],
    display: [bool; 64 * 32],
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            reg_pc: 0x200,
            reg_sp: 0,
            reg_i: 0,
            reg_timer_delay: 0,
            reg_timer_sound: 0,
            reg_v: [0; 16],

            stack: [0; 16],
            memory: [0; 4096],
            keyboard: [false; 16],
            display: [false; 64 * 32],
        }
    }
}
