const PROG_START: usize = 0x200;
const PROG_END: usize = 0xEA0;

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
            reg_pc: PROG_START as u16,
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
        } else if opcode & 0x1000 == 0x1000 {
            // 0x1NNN (jump)
            self.reg_pc = opcode & 0x0FFF;
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

#[cfg(test)]
mod tests {
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
    fn jump() {
        let mut chip8 = Chip8::new();
        chip8.memory[PROG_START] = 0x1A;
        chip8.memory[PROG_START + 1] = 0xF8;
        chip8.memory[0x0AF8] = 1;
        chip8.cycle();
        assert_eq!(chip8.reg_pc, 0x0AF8);
        assert_eq!(chip8.memory[chip8.reg_pc as usize], 1);
    }
}
