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

    pub fn cycle(&mut self) {
        let opcode = get_opcode(&self.memory, self.reg_pc);
        self.reg_pc += 2;
        println!("Opcode: {:#X}", opcode);
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
    use super::{get_opcode, Chip8};

    #[test]
    fn can_get_opcode() {
        let mut chip8 = Chip8::new();
        chip8.memory[chip8.reg_pc as usize] = 0xF8;
        chip8.memory[(chip8.reg_pc + 1) as usize] = 0x32;
        let opcode = get_opcode(&chip8.memory, chip8.reg_pc);
        assert_eq!(opcode, 0xF832)
    }

    #[test]
    fn emulator_cycle_increases_pc() {
        let mut chip8 = Chip8::new();
        let initial_pc = chip8.reg_pc;
        chip8.cycle();
        assert_eq!(chip8.reg_pc, initial_pc + 2);
    }
}
