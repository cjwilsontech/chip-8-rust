use std::time;

use rand::Rng;

pub const DISPLAY_WIDTH: usize = 64;
pub const DISPLAY_HEIGHT: usize = 32;
pub type Display = [[bool; DISPLAY_WIDTH]; DISPLAY_HEIGHT];

pub struct Chip8 {
    reg_pc: u16,
    reg_sp: u8,
    reg_i: u16,
    reg_timer_delay: u8,
    reg_timer_sound: u8,
    reg_v: [u8; 16],

    stack: [u16; STACK_SIZE],
    memory: [u8; 4096],
    keyboard: [bool; 16],
    display: Display,
    timer_start: time::Instant,
    timer_duration: time::Duration,

    redraw: fn(&Display) -> (),
}

impl Chip8 {
    pub fn new(redraw: fn(&Display) -> ()) -> Chip8 {
        Chip8 {
            reg_pc: PROG_START as u16,
            reg_sp: 0,
            reg_i: 0,
            reg_timer_delay: 0,
            reg_timer_sound: 0,
            reg_v: [0; 16],

            stack: [0; STACK_SIZE],
            memory: initialize_memory(),
            keyboard: [false; 16],
            display: [[false; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
            timer_start: time::Instant::now(),
            timer_duration: time::Duration::from_secs(1) / TIMER_CLOCK,

            redraw,
        }
    }

    pub fn load(&mut self, data: Vec<u8>) -> Result<(), &str> {
        if data.len() > PROG_END - PROG_START {
            return Err("ROM data is too large for memory.");
        }

        for (index, data_byte) in data.into_iter().enumerate() {
            self.memory[PROG_START + index] = data_byte;
        }

        Ok(())
    }

    pub fn cycle(&mut self) {
        self.process_timers();
        let opcode = get_opcode(&self.memory, self.reg_pc);

        if opcode & 0xFFFF == 0x00E0 {
            // 0x00E0 (clear the screen)
            for row in 0..DISPLAY_HEIGHT {
                for col in 0..DISPLAY_WIDTH {
                    self.display[row][col] = false;
                }
            }
            self.reg_pc += 2;
        } else if opcode & 0xFFFF == 0x00EE {
            // 0x00EE (return from subroutine)
            match u8::checked_sub(self.reg_sp, 1) {
                Some(sp) => self.reg_sp = sp,
                None => panic!("Stack underflow."),
            }
            self.reg_pc = *self
                .stack
                .get(self.reg_sp as usize)
                .expect("Stack underflow.");
            self.reg_pc += 2;
        } else if opcode & 0xF000 == 0x1000 {
            // 0x1NNN (jump)
            self.reg_pc = opcode & 0x0FFF;
        } else if opcode & 0xF000 == 0x2000 {
            // 0x2NNN (call subroutine)
            if self.reg_sp >= STACK_SIZE as u8 {
                panic!("Stack overflow.")
            }
            self.stack[self.reg_sp as usize] = self.reg_pc;
            self.reg_sp += 1;
            self.reg_pc = opcode & 0x0FFF;
        } else if opcode & 0xF000 == 0x3000 {
            // 0x3XNN (if vx != NN then)
            let value = (opcode & 0xFF) as u8;
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let register_value = *self.reg_v.get(index).expect("V index to be in bounds.");
            if value == register_value {
                self.reg_pc += 2;
            }
            self.reg_pc += 2;
        } else if opcode & 0xF000 == 0x4000 {
            // 0x4XNN (if vx == NN then)
            let value = (opcode & 0xFF) as u8;
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let register_value = *self.reg_v.get(index).expect("V index to be in bounds.");
            if value != register_value {
                self.reg_pc += 2;
            }
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x5000 {
            // 0x5XY0 (if vx != vy then)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vx_value = self.reg_v.get(index_x).expect("V index to be in bounds.");
            let vy_value = self.reg_v.get(index_y).expect("V index to be in bounds.");
            if vx_value == vy_value {
                self.reg_pc += 2;
            }
            self.reg_pc += 2;
        } else if opcode & 0xF000 == 0x6000 {
            // 0x6XNN (vx := NN)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            self.reg_v[index] = opcode as u8;
            self.reg_pc += 2;
        } else if opcode & 0xF000 == 0x7000 {
            // 0x7XNN (vx += NN)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let value = (opcode & 0xFF) as u8;
            self.reg_v[index] = u8::wrapping_add(self.reg_v[index], value);
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x8000 {
            // 0x8XY0 (vx := vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vy_value = *self.reg_v.get(index_y).expect("V index to be in bounds.");
            self.reg_v[index_x] = vy_value;
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x8001 {
            // 0x8XY1 (vx |= vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vx_value = self.reg_v.get(index_x).expect("V index to be in bounds.");
            let vy_value = self.reg_v.get(index_y).expect("V index to be in bounds.");
            self.reg_v[index_x] = vx_value | vy_value;
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x8002 {
            // 0x8XY2 (vx &= vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vx_value = self.reg_v.get(index_x).expect("V index to be in bounds.");
            let vy_value = self.reg_v.get(index_y).expect("V index to be in bounds.");
            self.reg_v[index_x] = vx_value & vy_value;
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x8003 {
            // 0x8XY3 (vx ^= vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vx_value = self.reg_v.get(index_x).expect("V index to be in bounds.");
            let vy_value = self.reg_v.get(index_y).expect("V index to be in bounds.");
            self.reg_v[index_x] = vx_value ^ vy_value;
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x8004 {
            // 0x8XY4 (vx += vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vx_value = *self.reg_v.get(index_x).expect("V index to be in bounds.");
            let vy_value = *self.reg_v.get(index_y).expect("V index to be in bounds.");

            let (new_value, did_overflow) = u8::overflowing_add(vx_value, vy_value);
            self.reg_v[index_x] = new_value;
            self.reg_v[15] = if did_overflow { 1 } else { 0 };
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x8005 {
            // 0x8XY5 (vx -= vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vx_value = *self.reg_v.get(index_x).expect("V index to be in bounds.");
            let vy_value = *self.reg_v.get(index_y).expect("V index to be in bounds.");

            let (new_value, did_overflow) = u8::overflowing_sub(vx_value, vy_value);
            self.reg_v[index_x] = new_value;
            self.reg_v[15] = if did_overflow { 1 } else { 0 };
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x8006 {
            // 0x8XY6 (vx >>= vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vy_value = *self.reg_v.get(index_y).expect("V index to be in bounds.");

            self.reg_v[index_x] = vy_value >> 1;
            self.reg_v[15] = vy_value & 1;
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x8007 {
            // 0x8XY7 (vx =- vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vx_value = *self.reg_v.get(index_x).expect("V index to be in bounds.");
            let vy_value = *self.reg_v.get(index_y).expect("V index to be in bounds.");

            let (new_value, did_overflow) = u8::overflowing_sub(vy_value, vx_value);
            self.reg_v[index_x] = new_value;
            self.reg_v[15] = if did_overflow { 1 } else { 0 };
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x800E {
            // 0x8XYE (vx <<= vy)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vy_value = *self.reg_v.get(index_y).expect("V index to be in bounds.");

            self.reg_v[index_x] = vy_value << 1;
            self.reg_v[15] = vy_value & 0b1000_0000;
            self.reg_pc += 2;
        } else if opcode & 0xF00F == 0x9000 {
            // 0x9XY0 (if vx == vy then)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let vx_value = self.reg_v.get(index_x).expect("V index to be in bounds.");
            let vy_value = self.reg_v.get(index_y).expect("V index to be in bounds.");
            if vx_value != vy_value {
                self.reg_pc += 2;
            }
            self.reg_pc += 2;
        } else if opcode & 0xF000 == 0xA000 {
            // 0xANNN (i := NNN)
            self.reg_i = opcode & 0x0FFF;
            self.reg_pc += 2;
        } else if opcode & 0xF000 == 0xB000 {
            // 0xBNNN (jump0 NNN)
            let value = opcode & 0x0FFF;
            self.reg_pc = value + self.reg_v[0] as u16;
        } else if opcode & 0xF000 == 0xC000 {
            // 0xCXNN (vx := random NN)
            let mut rng = rand::thread_rng();
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let mask = (opcode & 0xFF) as u8;
            self.reg_v[index] = rng.gen::<u8>() & mask;
            self.reg_pc += 2;
        } else if opcode & 0xF000 == 0xD000 {
            // 0xDXYN (sprite vx vy N)
            let index_x = ((opcode & 0x0F00) >> 8) as usize;
            let index_y = ((opcode & 0x00F0) >> 4) as usize;
            let byte_count = (opcode & 0x0F) as usize;
            let vx_value = *self.reg_v.get(index_x).expect("V index to be in bounds.") as usize;
            let vy_value = *self.reg_v.get(index_y).expect("V index to be in bounds.") as usize;

            self.reg_v[15] = 0;
            for row in 0..byte_count {
                let y = (vy_value + row) % DISPLAY_HEIGHT;
                for col in 0..8 {
                    let x = (vx_value + col) % DISPLAY_WIDTH;

                    // The pixel we should show will be the XOR'd value of the current display pixel and the bit in memory.
                    let value = self.display[y][x]
                        ^ ((self.memory[self.reg_i as usize + row] & u8::pow(2, 7 - col as u32))
                            != 0);

                    // If a pixel was erased, set VF to 1.
                    if self.display[y][x] && !value {
                        self.reg_v[15] = 1;
                    }

                    self.display[y][x] = value;
                }
            }

            self.reg_pc += 2;
            (self.redraw)(&self.display);
        } else if opcode & 0xF0FF == 0xE09E {
            // 0xEX9E (if vx -key then)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let key_index = *self.reg_v.get(index).expect("V index to be in bounds.") as usize;
            if *self
                .keyboard
                .get(key_index)
                .expect("Keyboard index to be in bounds.")
            {
                self.reg_pc += 2;
            }
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xE0A1 {
            // 0xEXA1 (if vx key then)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let key_index = *self.reg_v.get(index).expect("V index to be in bounds.") as usize;
            if !*self
                .keyboard
                .get(key_index)
                .expect("Keyboard index to be in bounds.")
            {
                self.reg_pc += 2;
            }
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xF007 {
            // FX07 (vx := delay)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            self.reg_v[index] = self.reg_timer_delay;
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xF00A {
            // 0xFX0A (vx := key)
            let active_key = self.keyboard.iter().enumerate().find(|key| *key.1);
            if active_key.is_some() {
                let index = ((opcode & 0x0F00) >> 8) as usize;
                self.reg_v[index] = active_key.unwrap().0 as u8;
                self.reg_pc += 2;
            }
        } else if opcode & 0xF0FF == 0xF015 {
            // 0xFX15 (delay := vx)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            self.reg_timer_delay = *self.reg_v.get(index).expect("V index to be in bounds.");
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xF018 {
            // 0xFX18 (buzzer := vx)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            self.reg_timer_sound = *self.reg_v.get(index).expect("V index to be in bounds.");
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xF01E {
            // 0xFX1E (i += vx)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let value = *self.reg_v.get(index).expect("V index to be in bounds.") as u16;
            self.reg_i += value;
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xF029 {
            // 0xFX29 (i := hex vx)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let value = *self.reg_v.get(index).expect("V index to be in bounds.") as u16;
            if value > 15 {
                panic!("Vx to be within the range 0-15.");
            }
            self.reg_i = SPRITE_START as u16 + value * SPRITE_BYTE_WIDTH as u16;
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xF033 {
            // 0xFX33 (bcd vx)
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let value = self.reg_v.get(index).expect("V index to be in bounds.");
            self.memory[self.reg_i as usize] = value / 100;
            self.memory[self.reg_i as usize + 1] = (value / 10) % 10;
            self.memory[self.reg_i as usize + 2] = value % 10;
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xF055 {
            // 0xFX55 (save vx)
            let max_index = ((opcode & 0x0F00) >> 8) as usize;
            for index in 0..=max_index {
                let value = *self.reg_v.get(index).expect("V index to be in bounds.");
                self.memory[self.reg_i as usize] = value;
                self.reg_i += 1;
            }
            self.reg_pc += 2;
        } else if opcode & 0xF0FF == 0xF065 {
            // 0xFX65 (load vx)
            let max_index = ((opcode & 0x0F00) >> 8) as usize;
            for index in 0..=max_index {
                let value = *self
                    .memory
                    .get(self.reg_i as usize)
                    .expect("V index to be in bounds.");
                self.reg_v[index] = value;
                self.reg_i += 1;
            }
            self.reg_pc += 2;
        } else {
            panic!("Unknown opcode: {:#X}", opcode);
        }
    }

    pub fn should_play_sound(&self) -> bool {
        self.reg_timer_sound > 1
    }

    pub fn set_keyboard_key(&mut self, index: usize, is_pressed: bool) {
        if index > 15 {
            panic!("Expected index to be <= 15");
        }
        self.keyboard[index] = is_pressed;
    }

    pub fn clear_keyboard(&mut self) {
        self.keyboard = [false; 16];
    }

    fn process_timers(&mut self) {
        let now = time::Instant::now();
        if now.saturating_duration_since(self.timer_start) >= self.timer_duration {
            self.reg_timer_delay = u8::saturating_sub(self.reg_timer_delay, 1);
            self.reg_timer_sound = u8::saturating_sub(self.reg_timer_sound, 1);
            self.timer_start = now;
        }
    }
}

fn get_opcode(memory: &[u8; 4096], pc: u16) -> u16 {
    // Encoding is in Big Endian.
    let big: u16 = (memory
        .get(pc as usize)
        .expect("The PC to not be OOB.")
        .clone() as u16)
        << 8;

    let little: u16 = memory
        .get((pc + 1) as usize)
        .expect("The PC + 1 to not be OOB.")
        .clone() as u16;

    big | little
}

fn initialize_memory() -> [u8; 4096] {
    let mut memory = [0; 4096];

    // Set sprite data.
    for (index, data) in ALL_SPRITE_DATA.into_iter().enumerate() {
        memory[SPRITE_START + index] = data;
    }

    memory
}

const ALL_SPRITE_DATA: [u8; SPRITE_BYTE_WIDTH * SPRITE_COUNT] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // Zero
    0x20, 0x60, 0x20, 0x20, 0x70, // One
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // Two
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // Three
    0x90, 0x90, 0xF0, 0x10, 0x10, // Four
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // Five
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // Six
    0xF0, 0x10, 0x20, 0x40, 0x40, // Seven
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // Eight
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // Nine
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];
const PROG_START: usize = 0x200;
const PROG_END: usize = 0xEA0;
const SPRITE_COUNT: usize = 16;
const SPRITE_START: usize = 0;
const SPRITE_BYTE_WIDTH: usize = 5;
const STACK_SIZE: usize = 16;
const TIMER_CLOCK: u32 = 60;

#[cfg(test)]
#[path = "./emulator_test.rs"]
mod emulator_test;
