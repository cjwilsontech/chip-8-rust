use rand::Rng;

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
    display: [bool; 64 * 32],
}

impl Chip8 {
    pub fn new() -> Chip8 {
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
            display: [false; 64 * 32],
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
        let opcode = get_opcode(&self.memory, self.reg_pc);

        if opcode & 0xFFFF == 0x00E0 {
            // 0x00E0 (clear the screen)
            self.display = [false; 64 * 32];
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
        } else if opcode & 0xF000 == 0xC000 {
            // 0xCXNN (vx := random NN)
            let mut rng = rand::thread_rng();
            let index = ((opcode & 0x0F00) >> 8) as usize;
            let mask = (opcode & 0xFF) as u8;
            self.reg_v[index] = rng.gen::<u8>() & mask;
            self.reg_pc += 2;
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
            self.reg_i = SPRITE_START as u16 + value * SPRITE_WIDTH as u16;
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
            todo!("Unknown opcode: {:#X}", opcode);
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

const ALL_SPRITE_DATA: [u8; SPRITE_WIDTH * SPRITE_COUNT] = [
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
const SPRITE_WIDTH: usize = 5;
const STACK_SIZE: usize = 16;

#[cfg(test)]
mod tests {
    use crate::emulator::SPRITE_START;

    use super::{get_opcode, Chip8, PROG_END, PROG_START};

    #[test]
    fn can_get_opcode() {
        let mut chip8 = Chip8::new();
        chip8.memory[chip8.reg_pc as usize] = 0xF8;
        chip8.memory[(chip8.reg_pc + 1) as usize] = 0x32;
        let opcode = get_opcode(&chip8.memory, chip8.reg_pc);
        assert_eq!(opcode, 0xF832)
    }

    #[test]
    fn loads_rom_data() {
        let mut chip8 = Chip8::new();
        let data = vec![1; 3232];
        chip8.load(data).unwrap();
        assert_eq!(chip8.memory[PROG_START - 1], 0);
        assert_eq!(chip8.memory[PROG_START], 1);
        assert_eq!(chip8.memory[PROG_END - 1], 1);
        assert_eq!(chip8.memory[PROG_END], 0);
    }

    #[test]
    #[should_panic(expected = "ROM data is too large for memory.")]
    fn prevents_rom_overflow() {
        let mut chip8 = Chip8::new();
        let data = vec![1; 3233];
        chip8.load(data).unwrap();
    }

    #[test]
    fn clear() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x00;
        chip8.memory[PROG_START + 1] = 0xE0;
        chip8.display = [true; 64 * 32];
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x202);
        assert_eq!(chip8.display, [false; 64 * 32]);
    }

    #[test]
    fn calls_subroutine() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x22;
        chip8.memory[PROG_START + 1] = 0x38;
        chip8.reg_sp = 15;
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x238);
        assert_eq!(chip8.reg_sp, 16);
        assert_eq!(chip8.stack[15] as usize, PROG_START);
    }

    #[test]
    #[should_panic(expected = "Stack overflow.")]
    fn prevents_stack_overflow() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x22;
        chip8.memory[PROG_START + 1] = 0x38;
        chip8.reg_sp = 16;
        chip8.cycle();
    }

    #[test]
    fn returns_from_subroutine() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x00;
        chip8.memory[PROG_START + 1] = 0xEE;
        chip8.stack[1] = 0x2F8;
        chip8.reg_sp = 2;
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x2FA);
        assert_eq!(chip8.reg_sp, 1);
    }

    #[test]
    #[should_panic(expected = "Stack underflow.")]
    fn prevents_stack_underflow() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x00;
        chip8.memory[PROG_START + 1] = 0xEE;
        chip8.reg_sp = 0;
        chip8.cycle();
    }

    #[test]
    fn jump() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x1A;
        chip8.memory[PROG_START + 1] = 0xF8;
        chip8.memory[0x0AF8] = 1;
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x0AF8);
        assert_eq!(chip8.memory[chip8.reg_pc as usize], 1);
    }

    #[test]
    fn set_register_to_const() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x63;
        chip8.memory[PROG_START + 1] = 0x64;
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x202);
        assert_eq!(chip8.reg_v[3], 0x64);
    }

    #[test]
    fn set_i_to_const() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xA3;
        chip8.memory[PROG_START + 1] = 0x64;
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x202);
        assert_eq!(chip8.reg_i, 0x364);
    }

    #[test]
    fn enter_and_exit_subroutine() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x23;
        chip8.memory[PROG_START + 1] = 0x64;
        chip8.memory[0x364] = 0x00;
        chip8.memory[0x365] = 0xEE;
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x364);
        assert_eq!(chip8.reg_sp, 1);
        assert_eq!(chip8.stack[0], 0x200);
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x202);
        assert_eq!(chip8.reg_sp, 0);
    }

    #[test]
    fn check_key_pressed() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xE4;
        chip8.memory[PROG_START + 1] = 0x9E;
        chip8.memory[PROG_START + 2] = 0xE3;
        chip8.memory[PROG_START + 3] = 0xA1;
        chip8.memory[PROG_START + 4] = 0xE3;
        chip8.memory[PROG_START + 5] = 0x9E;
        chip8.memory[PROG_START + 8] = 0xE4;
        chip8.memory[PROG_START + 9] = 0xA1;
        chip8.reg_v[3] = 4;
        chip8.keyboard[4] = true;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 4);
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 8);
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 12);
    }

    #[test]
    fn set_delay_timer() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xF4;
        chip8.memory[PROG_START + 1] = 0x15;
        chip8.reg_v[4] = 60;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_timer_delay, 60);
    }

    #[test]
    fn set_sound_timer() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xF4;
        chip8.memory[PROG_START + 1] = 0x18;
        chip8.reg_v[4] = 60;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_timer_sound, 60);
    }

    #[test]
    fn v_not_equals_const() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x34;
        chip8.memory[PROG_START + 1] = 0x18;
        chip8.memory[PROG_START + 4] = 0x34;
        chip8.memory[PROG_START + 5] = 0x17;
        chip8.reg_v[4] = 0x18;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 4);
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 6);
    }

    #[test]
    fn v_equals_const() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x44;
        chip8.memory[PROG_START + 1] = 0x18;
        chip8.memory[PROG_START + 2] = 0x44;
        chip8.memory[PROG_START + 3] = 0x17;
        chip8.reg_v[4] = 0x18;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 6);
    }

    #[test]
    fn vx_equals_vy() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x54;
        chip8.memory[PROG_START + 1] = 0x30;
        chip8.memory[PROG_START + 4] = 0x54;
        chip8.memory[PROG_START + 5] = 0x20;
        chip8.reg_v[2] = 0x04;
        chip8.reg_v[3] = 0x18;
        chip8.reg_v[4] = 0x18;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 4);
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 6);
    }

    #[test]
    fn vx_not_equals_vy() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x94;
        chip8.memory[PROG_START + 1] = 0x30;
        chip8.memory[PROG_START + 2] = 0x94;
        chip8.memory[PROG_START + 3] = 0x20;
        chip8.reg_v[2] = 0x04;
        chip8.reg_v[3] = 0x18;
        chip8.reg_v[4] = 0x18;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 6);
    }

    #[test]
    fn generate_random() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xC4;
        chip8.memory[PROG_START + 1] = 0xFF;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    }

    #[test]
    fn bcd() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xF4;
        chip8.memory[PROG_START + 1] = 0x33;
        chip8.reg_v[4] = 245;
        chip8.reg_i = 0x2F5;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_i, 0x2F5);
        assert_eq!(chip8.reg_v[4], 245);
        assert_eq!(chip8.memory[0x2F5], 2);
        assert_eq!(chip8.memory[0x2F6], 4);
        assert_eq!(chip8.memory[0x2F7], 5);
    }

    #[test]
    fn save_vx() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xF3;
        chip8.memory[PROG_START + 1] = 0x55;
        chip8.reg_v[0] = 245;
        chip8.reg_v[1] = 0;
        chip8.reg_v[2] = 10;
        chip8.reg_v[3] = 42;
        chip8.reg_v[4] = 19;
        chip8.reg_i = 0x2F0;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_i, 0x2F4);
        assert_eq!(chip8.reg_v[0], 245);
        assert_eq!(chip8.memory[0x2F0], 245);
        assert_eq!(chip8.memory[0x2F1], 0);
        assert_eq!(chip8.memory[0x2F2], 10);
        assert_eq!(chip8.memory[0x2F3], 42);
        assert_ne!(chip8.memory[0x2F4], 19);
    }

    #[test]
    fn load_vx() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xF3;
        chip8.memory[PROG_START + 1] = 0x65;
        chip8.reg_i = 0x2F0;
        chip8.memory[chip8.reg_i as usize] = 245;
        chip8.memory[chip8.reg_i as usize + 1] = 0;
        chip8.memory[chip8.reg_i as usize + 2] = 10;
        chip8.memory[chip8.reg_i as usize + 3] = 42;
        chip8.memory[chip8.reg_i as usize + 4] = 19;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_i, 0x2F4);
        assert_eq!(chip8.memory[0x2F0], 245);
        assert_eq!(chip8.reg_v[0], 245);
        assert_eq!(chip8.reg_v[1], 0);
        assert_eq!(chip8.reg_v[2], 10);
        assert_eq!(chip8.reg_v[3], 42);
        assert_ne!(chip8.reg_v[4], 19);
    }

    #[test]
    fn add_vx_to_i() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xF4;
        chip8.memory[PROG_START + 1] = 0x1E;
        chip8.reg_i = 0x2F0;
        chip8.reg_v[4] = 3;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 3);
        assert_eq!(chip8.reg_i, 0x2F3);
    }

    #[test]
    fn add_const_to_vx() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x74;
        chip8.memory[PROG_START + 1] = 0x05;
        chip8.reg_v[4] = 3;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 8);
    }

    #[test]
    fn set_i_to_sprite_for_vx() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xF4;
        chip8.memory[PROG_START + 1] = 0x29;
        chip8.reg_v[4] = 12;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 12);
        assert_eq!(chip8.reg_i, (SPRITE_START + 60) as u16);
        assert_eq!(chip8.memory[chip8.reg_i as usize], 0xF0);
        assert_eq!(chip8.memory[chip8.reg_i as usize + 1], 0x80);
        assert_eq!(chip8.memory[chip8.reg_i as usize + 2], 0x80);
        assert_eq!(chip8.memory[chip8.reg_i as usize + 3], 0x80);
        assert_eq!(chip8.memory[chip8.reg_i as usize + 4], 0xF0);
    }

    #[test]
    #[should_panic(expected = "Vx to be within the range 0-15.")]
    fn set_i_to_sprite_for_invalid_vx() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0xF4;
        chip8.memory[PROG_START + 1] = 0x29;
        chip8.reg_v[4] = 17;
        chip8.cycle();
    }

    #[test]
    fn set_vx_to_vy() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x50;
        chip8.reg_v[5] = 17;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 17);
    }

    #[test]
    fn set_vx_to_or_vy() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x51;
        chip8.reg_v[4] = 0b1010;
        chip8.reg_v[5] = 0b0100;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 0b1110);
    }

    #[test]
    fn set_vx_to_and_vy() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x52;
        chip8.reg_v[4] = 0b1010;
        chip8.reg_v[5] = 0b1100;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 0b1000);
    }

    #[test]
    fn set_vx_to_xor_vy() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x53;
        chip8.reg_v[4] = 0b1010;
        chip8.reg_v[5] = 0b1100;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 0b0110);
    }

    #[test]
    fn add_vx_vy() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x54;
        chip8.reg_v[4] = 1;
        chip8.reg_v[5] = 2;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 3);
        assert_eq!(chip8.reg_v[15], 0)
    }

    #[test]
    fn add_vx_vy_carry() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x54;
        chip8.reg_v[4] = 255;
        chip8.reg_v[5] = 100;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 99);
        assert_eq!(chip8.reg_v[15], 1)
    }

    #[test]
    fn sub_vx_vy() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x55;
        chip8.reg_v[4] = 4;
        chip8.reg_v[5] = 2;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 2);
        assert_eq!(chip8.reg_v[15], 0)
    }

    #[test]
    fn sub_vx_vy_borrow() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x55;
        chip8.reg_v[4] = 100;
        chip8.reg_v[5] = 255;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 101);
        assert_eq!(chip8.reg_v[15], 1)
    }

    #[test]
    fn shift_right() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x56;
        chip8.reg_v[5] = 0b1001_1111;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 0b0100_1111);
        assert_eq!(chip8.reg_v[5], 0b1001_1111);
        assert_eq!(chip8.reg_v[15], 1)
    }

    #[test]
    fn shift_left() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x5E;
        chip8.reg_v[5] = 0b1001_1111;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 0b0011_1110);
        assert_eq!(chip8.reg_v[5], 0b1001_1111);
        assert_eq!(chip8.reg_v[15], 0b1000_0000)
    }

    #[test]
    fn sub_vy_vx() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x57;
        chip8.reg_v[4] = 2;
        chip8.reg_v[5] = 4;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 2);
        assert_eq!(chip8.reg_v[15], 0)
    }

    #[test]
    fn sub_vy_vx_borrow() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x84;
        chip8.memory[PROG_START + 1] = 0x57;
        chip8.reg_v[4] = 255;
        chip8.reg_v[5] = 100;
        chip8.cycle();
        assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
        assert_eq!(chip8.reg_v[4], 101);
        assert_eq!(chip8.reg_v[15], 1)
    }
}
