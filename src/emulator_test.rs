use super::{get_opcode, Chip8, DISPLAY_HEIGHT, DISPLAY_WIDTH, PROG_END, PROG_START};
use crate::emulator::SPRITE_START;
use std::{thread, time};

#[test]
fn can_get_opcode() {
    let mut chip8 = get_emulator();
    chip8.memory[chip8.reg_pc as usize] = 0xF8;
    chip8.memory[(chip8.reg_pc + 1) as usize] = 0x32;
    let opcode = get_opcode(&chip8.memory, chip8.reg_pc);
    assert_eq!(opcode, 0xF832);
}

#[test]
fn loads_rom_data() {
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
    let data = vec![1; 3233];
    chip8.load(data).unwrap();
}

#[test]
fn clear() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0x00;
    chip8.memory[PROG_START + 1] = 0xE0;
    chip8.display = [true; 64 * 32];
    chip8.cycle();
    assert_eq!(chip8.reg_pc, 0x202);
    assert_eq!(chip8.display, [false; 64 * 32]);
}

#[test]
fn calls_subroutine() {
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0x22;
    chip8.memory[PROG_START + 1] = 0x38;
    chip8.reg_sp = 16;
    chip8.cycle();
}

#[test]
fn returns_from_subroutine() {
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0x00;
    chip8.memory[PROG_START + 1] = 0xEE;
    chip8.reg_sp = 0;
    chip8.cycle();
}

#[test]
fn jump() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0x1A;
    chip8.memory[PROG_START + 1] = 0xF8;
    chip8.memory[0x0AF8] = 1;
    chip8.cycle();
    assert_eq!(chip8.reg_pc, 0x0AF8);
    assert_eq!(chip8.memory[chip8.reg_pc as usize], 1);
}

#[test]
fn set_register_to_const() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0x63;
    chip8.memory[PROG_START + 1] = 0x64;
    chip8.cycle();
    assert_eq!(chip8.reg_pc, 0x202);
    assert_eq!(chip8.reg_v[3], 0x64);
}

#[test]
fn set_i_to_const() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xA3;
    chip8.memory[PROG_START + 1] = 0x64;
    chip8.cycle();
    assert_eq!(chip8.reg_pc, 0x202);
    assert_eq!(chip8.reg_i, 0x364);
}

#[test]
fn enter_and_exit_subroutine() {
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xF4;
    chip8.memory[PROG_START + 1] = 0x15;
    chip8.reg_v[4] = 60;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    assert_eq!(chip8.reg_timer_delay, 60);
}

#[test]
fn set_sound_timer() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xF4;
    chip8.memory[PROG_START + 1] = 0x18;
    chip8.reg_v[4] = 60;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    assert_eq!(chip8.reg_timer_sound, 60);
}

#[test]
fn v_not_equals_const() {
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xC4;
    chip8.memory[PROG_START + 1] = 0xFF;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
}

#[test]
fn bcd() {
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0x74;
    chip8.memory[PROG_START + 1] = 0x05;
    chip8.reg_v[4] = 3;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    assert_eq!(chip8.reg_v[4], 8);
}

#[test]
fn set_i_to_sprite_for_vx() {
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xF4;
    chip8.memory[PROG_START + 1] = 0x29;
    chip8.reg_v[4] = 17;
    chip8.cycle();
}

#[test]
fn set_vx_to_vy() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0x84;
    chip8.memory[PROG_START + 1] = 0x50;
    chip8.reg_v[5] = 17;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    assert_eq!(chip8.reg_v[4], 17);
}

#[test]
fn set_vx_to_or_vy() {
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
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
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0x84;
    chip8.memory[PROG_START + 1] = 0x57;
    chip8.reg_v[4] = 255;
    chip8.reg_v[5] = 100;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    assert_eq!(chip8.reg_v[4], 101);
    assert_eq!(chip8.reg_v[15], 1)
}

#[test]
fn jump0() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xBF;
    chip8.memory[PROG_START + 1] = 0x32;
    chip8.reg_v[0] = 5;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, 0x0F37);
}

#[test]
fn set_vx_delay() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xF4;
    chip8.memory[PROG_START + 1] = 0x07;
    chip8.reg_timer_delay = 8;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    assert_eq!(chip8.reg_v[4], 8)
}

#[test]
fn wait_for_key() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xF4;
    chip8.memory[PROG_START + 1] = 0x0A;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START);
    chip8.keyboard[10] = true;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    assert_eq!(chip8.reg_v[4], 10)
}

#[test]
fn draw_sprite() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xD4;
    chip8.memory[PROG_START + 1] = 0x55;
    chip8.memory[PROG_START + 2] = 0xD4;
    chip8.memory[PROG_START + 3] = 0x55;
    chip8.reg_v[4] = 5;
    chip8.reg_v[5] = 10;
    chip8.reg_i = SPRITE_START as u16 + 5;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 2);
    assert_eq!(chip8.reg_v[15], 0);
    chip8.reg_i = SPRITE_START as u16;
    chip8.cycle();
    assert_eq!(chip8.reg_pc as usize, PROG_START + 4);
    assert_eq!(chip8.reg_v[15], 1);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 10 + 4], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 10 + 5], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 10 + 6], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 10 + 7], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 10 + 8], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 10 + 9], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 11 + 4], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 11 + 5], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 11 + 6], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 11 + 7], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 11 + 8], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 11 + 9], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 12 + 4], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 12 + 5], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 12 + 6], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 12 + 7], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 12 + 8], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 12 + 9], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 13 + 4], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 13 + 5], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 13 + 6], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 13 + 7], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 13 + 8], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 13 + 9], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 14 + 4], false);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 14 + 5], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 14 + 6], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 14 + 7], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 14 + 8], true);
    assert_eq!(chip8.display[DISPLAY_WIDTH * 14 + 9], false);
}

#[test]
fn decrements_timers() {
    let mut chip8 = get_emulator();
    chip8.memory[PROG_START] = 0xF4;
    chip8.memory[PROG_START + 1] = 0x15;
    chip8.memory[PROG_START + 2] = 0xF4;
    chip8.memory[PROG_START + 3] = 0x18;
    chip8.memory[PROG_START + 4] = 0x12;
    chip8.memory[PROG_START + 5] = 0x04;
    chip8.reg_v[4] = 30;
    chip8.cycle();
    assert_eq!(chip8.reg_timer_delay, 30);
    chip8.cycle();
    assert_eq!(chip8.reg_timer_sound, 30);
    assert_eq!(chip8.reg_pc as usize, PROG_START + 4);
    let sleep_duration = time::Duration::from_secs(1) / 60;
    thread::sleep(sleep_duration);
    chip8.cycle();
    assert_eq!(chip8.reg_timer_delay, 29);
    assert_eq!(chip8.reg_timer_sound, 29);
}

fn get_emulator() -> Chip8 {
    Chip8::new(draw_screen)
}
fn draw_screen(_: &[bool; DISPLAY_WIDTH * DISPLAY_HEIGHT]) {}
