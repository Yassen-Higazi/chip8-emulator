use sdl2::pixels::Color;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub const NUM_REGS: usize = 16;

pub const NUM_KEYS: usize = 16;

pub const RAM_SIZE: usize = 4096;

pub const STACK_SIZE: usize = 16;
pub const START_ADDR: u16 = 0x200;

pub const FONTSET_SIZE: usize = 80;
pub const FONTSET: [u8; FONTSET_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const SOUND_FILE: &str = "./sounds/soft-piano-100-bpm-121529.mp3";

pub const SCALE: u32 = 30;

pub const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;

pub const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

pub const BLACK_COLOR: Color = Color::RGB(0, 0, 0);
pub const WHITE_COLOR: Color = Color::RGB(255, 255, 255);

pub const TICKS_PER_FRAME: u8 = 7;