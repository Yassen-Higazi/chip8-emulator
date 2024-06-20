use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;
use rodio::{Decoder, OutputStream, Sink, Source};

use crate::chip8::constants::{
    FONTSET, FONTSET_SIZE, NUM_KEYS, NUM_REGS, RAM_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, SOUND_FILE,
    STACK_SIZE, START_ADDR,
};

pub struct Chip8 {
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // 63x32 monochrome display; sprites are 8 pixels wide but between 1 and 16 pixels tall
    memory: [u8; RAM_SIZE],                       // RAM = 4KB
    v_reg: [u8; NUM_REGS],                        // general purpose registers V0-VF
    pc: u16,                                      // Program Counter
    i_reg: u16,                                   // memory access I Register
    delay_timer_reg: u8,                          // special register for delay timer
    sound_timer_reg: u8,                          // special register for sound timer
    stack: [u16; STACK_SIZE],                     // stack for subroutines calls and returns
    stack_pointer: u16,                           // a var that points to the top of the stack
    keyboard: [bool; NUM_KEYS],                   // a 16 key layout keyboard

    // Random number generator
    rng: ThreadRng,
}

impl Debug for Chip8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Chip8[program_counter: {}, stack_pointer: {}]",
            self.pc, self.stack_pointer
        )
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self {
            pc: START_ADDR,
            memory: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            keyboard: [false; NUM_KEYS],
            delay_timer_reg: 0,
            sound_timer_reg: 0,
            rng: thread_rng(),
        };

        chip8.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        return chip8;
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.memory = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.stack_pointer = 0;
        self.stack = [0; STACK_SIZE];
        self.keyboard = [false; NUM_KEYS];
        self.delay_timer_reg = 0;
        self.sound_timer_reg = 0;
        self.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn get_screen(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keyboard[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;

        let end = (START_ADDR as usize) + data.len();

        self.memory[start..end].copy_from_slice(data);
    }

    fn push(&mut self, val: u16) {
        // TODO: check if the stack is full
        self.stack[self.stack_pointer as usize] = val;
        self.stack_pointer += 1;
    }

    fn pop(&mut self) -> u16 {
        // TODO: check if the stack is empty
        self.stack_pointer -= 1;
        self.stack[self.stack_pointer as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.get_operation_code();

        // Decode & Execute
        self.execute(op);
    }

    fn get_operation_code(&mut self) -> u16 {
        let higher_byte = self.memory[self.pc as usize] as u16;
        let lower_byte = self.memory[(self.pc + 1) as usize] as u16;

        // << is a left shift by 8 bits, filling the remaining digits with 0s
        // | is a bitwise or operation that performs boolean OR on each bit of integer arguments
        // we are effectively combining the values as Big Endian
        let op = (higher_byte << 8) | lower_byte;

        //increment pc by 2 bytes to factor in program counter
        self.pc += 2;

        return op;
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // 0000 - No Operation
            (0, 0, 0, 0) => return,

            // 00E0 - Clear display
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }

            // 00EE - RET (Return from a subroutine.)
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }

            // 1nnn - JP addr (Jump to location nnn.)
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            // 2nnn - CALL addr (Call subroutine at nnn.)
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }

            // 3xkk - SE Vx, (byte Skip next instruction if Vx = kk.)
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // 4xkk - SNE Vx, byte (Skip next instruction if Vx != kk.)
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // 5xy0 - SE Vx, Vy (Skip next instruction if Vx = Vy.)
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // 6xkk - LD Vx, byte (Set Vx = kk.)
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                self.v_reg[x] = nn
            }

            // 7xkk - ADD Vx, byte (Set Vx = Vx + kk.)
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // 8xy0 - LD Vx, Vy (Set Vx = Vy.)
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] = self.v_reg[y];
            }

            // 8xy1 - OR Vx, Vy (Set Vx = Vx OR Vy.)
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                // bitwise OR
                self.v_reg[x] = self.v_reg[x] | self.v_reg[y];
            }

            // 8xy2 - AND Vx, Vy (Set Vx = Vx AND Vy.)
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                // bitwise AND
                self.v_reg[x] = self.v_reg[x] & self.v_reg[y];
            }

            // 8xy3 - XOR Vx, Vy (Set Vx = Vx XOR Vy.)
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                // bitwise XOR
                self.v_reg[x] = self.v_reg[x] ^ self.v_reg[y];
            }

            // 8xy4 - ADD Vx, Vy (Set Vx = Vx + Vy, set VF = carry.)
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // 8xy5 - SUB Vx, Vy (Set Vx = Vx - Vy, set VF = NOT borrow.)
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // 8xy6 - SHR Vx {, Vy} (Set Vx = Vx SHR 1.)
            (8, _, _, 6) => {
                let x = digit2 as usize;

                // get the least significant bit
                let lsb = self.v_reg[x] & 1;

                // right shift by 1 (equivalent to dividing by 2)
                self.v_reg[x] = self.v_reg[x] >> 1;

                self.v_reg[0xF] = lsb;
            }

            // 8xy4 - ADD Vx, Vy (Set Vx = Vx + Vy, set VF = carry.)
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }

            // 8xyE - SHL Vx {, Vy} (Set Vx = Vx SHL 1.)
            (8, _, _, 0xE) => {
                let x = digit2 as usize;

                // get the most significant bit
                let msb = (self.v_reg[x] >> 7) & 1;

                // left shift by 1 (equivalent to multiplying by 2)
                self.v_reg[x] = self.v_reg[x] << 1;

                self.v_reg[0xF] = msb;
            }

            // 9xy0 - SNE Vx, Vy (Skip next instruction if Vx != Vy.)
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // Annn - LD I, addr (Set I = nnn.)
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;

                self.i_reg = nnn;
            }

            // Bnnn - JP V0, addr (Jump to location nnn + V0.)
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;

                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            // Cxkk - RND Vx, byte (Set Vx = random byte AND kk.)
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let kk = (op & 0xFF) as u8;

                let number: u8 = self.rng.gen();

                self.v_reg[x] = number & kk;
            }

            // Dxyn - DRW Vx, Vy, nibble (Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.)
            (0xD, _, _, _) => {
                // Get the (x, y) coords for our sprite
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;

                // The last digit determines how many rows high our sprite is
                let num_of_rows_in_sprite = digit4;

                // Keep track if any pixels were flipped
                let mut flipped = false;

                // Iterate over each row of our sprite
                for y_line in 0..num_of_rows_in_sprite {
                    // Determine which memory address our row's data is stored
                    let addr = self.i_reg + y_line;
                    let pixels = self.memory[addr as usize];

                    // Iterate over each column in our row (max rows in screen is 8)
                    for x_line in 0..8 {
                        // Use a mask to fetch current pixel's bit. Only flip if a 1
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            // Sprites should wrap around screen, so apply modulo
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            // Get our pixel's index in the 1D screen array
                            let idx = x + SCREEN_WIDTH * y;

                            // Check if we're about to flip the pixel and set
                            flipped |= self.screen[idx];

                            self.screen[idx] ^= true;
                        }
                    }
                }

                // Populate VF register
                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }

            // Ex9E - SKP Vx (Skip next instruction if key with the value of Vx is pressed.)
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;

                let vx = self.v_reg[x];

                let key = self.keyboard[vx as usize];

                if key {
                    self.pc += 2;
                }
            }

            // ExA1 - SKNP Vx (Skip next instruction if key with the value of Vx is not pressed.)
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;

                let vx = self.v_reg[x];

                let key = self.keyboard[vx as usize];

                if !key {
                    self.pc += 2;
                }
            }

            // Fx07 - LD Vx, DT (Set Vx = delay timer value.)
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;

                self.v_reg[x] = self.delay_timer_reg
            }

            // Fx0A - LD Vx, K (Wait for a key press, store the value of the key in Vx.)
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;

                let mut is_key_pressed = false;

                // check if a key is pressed
                for i in 0..self.keyboard.len() {
                    if self.keyboard[i] {
                        self.v_reg[x] = self.keyboard[i] as u8;

                        is_key_pressed = true;

                        break;
                    }
                }

                // if no key is pressed redo operation
                if !is_key_pressed {
                    self.pc -= 2;
                }
            }

            // Fx15 - LD DT, Vx (Set delay timer = Vx.)
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;

                self.delay_timer_reg = self.v_reg[x];
            }

            // Fx18 - LD ST, Vx (Set sound timer = Vx.)
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;

                self.sound_timer_reg = self.v_reg[x];
            }

            // Fx1E - ADD I, Vx (Set I = I + Vx.)
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;

                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
            }

            // Fx29 - LD F, Vx (Set I = location of sprite for digit Vx.)
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;

                self.i_reg = (self.v_reg[x] as u16) * 5;
            }

            // Fx33 - LD B, Vx (Store BCD representation of Vx in memory locations I, I+1, and I+2.)
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                // Fetch the hundreds digit by dividing by 100 and tossing the decimal
                let hundreds = (vx / 100.0).floor() as u8;

                // Fetch the tens digit by dividing by 10, tossing the ones digit and the decimal
                let tens = ((vx / 10.0) % 10.0).floor() as u8;

                // Fetch the ones digit by tossing the hundreds and the tens
                let ones = (vx % 10.0) as u8;

                self.memory[self.i_reg as usize] = hundreds;
                self.memory[(self.i_reg + 1) as usize] = tens;
                self.memory[(self.i_reg + 2) as usize] = ones;
            }

            // Fx55 - LD [I], Vx (Store registers V0 through Vx in memory starting at location I.)
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;

                for i in 0..=x {
                    self.memory[(self.i_reg as usize) + i] = self.v_reg[i]
                }
            }

            // Fx65 - LD Vx, [I] (Read registers V0 through Vx from memory starting at location I.)
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;

                for i in 0..=x {
                    self.v_reg[i] = self.memory[(self.i_reg as usize) + i];
                }
            }

            (_, _, _, _) => unimplemented!("Unimplemented operation: {:#04x}", op),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer_reg > 0 {
            self.delay_timer_reg -= 1;
        }

        if self.sound_timer_reg > 0 {
            if self.sound_timer_reg == 1 {
                self.play_sound()
            }

            self.sound_timer_reg -= 1;
        }
    }

    // TODO: do not block main thread while playing the sound
    fn play_sound(&self) {
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = File::open(SOUND_FILE).expect("Could not open Audio File");

        // Decode that sound file into a source
        let file = BufReader::new(file);

        let source = Decoder::new(file)
            .expect("Could not decode File")
            .take_duration(Duration::from_secs_f32(0.20))
            .amplify(0.20);

        // Get an output stream handle to the default physical sound device
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Could not access default audio device");

        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.append(source);

        sink.sleep_until_end();
    }
}
