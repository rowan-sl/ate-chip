use owo_colors::OwoColorize;
use sdl2::render::Texture;
use std::time::{Instant, Duration};
use crate::keyboard::ACKey;

const PIXEL: &str = "██";

const SPRITE_CHARS: [[u8; 5]; 0x10] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

const SPRITE_CHARS_ADDR: u16 = 0;

const STACK_SIZE: usize = 0x10;


pub struct ACRenderer {
    pixels: [[bool; 64]; 32],
    //for caching the last render
    last_pixels: Option<[[bool; 64]; 32]>,
    last_render: Option<String>,
}

impl ACRenderer {
    /// Creates a new ACEmulator with all pixels set to black
    pub fn new() -> Self {
        Self {
            pixels: [[false; 64]; 32],
            last_pixels: None,
            last_render: None,
        }
    }

    pub fn xor_pixel(&mut self, mut x: u8, mut y: u8, p: bool) -> bool {
        if x >= 64 {
            x -= 64;
        } //else if x < 0 {
        //    x += 64;
        // }

        if y > 32 {
            y -= 32;
        }// else if y < 0 {
        //     y += 32;
        // }

        self.pixels[31-y as usize][x as usize] ^= p;

        !self.pixels[y as usize][x as usize]//return if the value at xy was erased
    }

    pub fn set_pixel(&mut self, mut x: i8, mut y: i8, v: bool) {
        if x >= 64 {
            x -= 64;
        } else if x < 0 {
            x += 64;
        }

        if y > 32 {
            y -= 32;
        } else if y < 0 {
            y += 32;
        }

        self.pixels[y as usize][x as usize] = v;
    }

    pub fn clear(&mut self) {
        self.pixels = [[false; 64]; 32]
    }

    pub fn render_string(&mut self) -> String {
        // caching yay
        if let Some(last_pixels) = &self.last_pixels {
            if *last_pixels == self.pixels {
                return self.last_render.clone().unwrap();
            }
        }

        let mut rendered = String::new();

        let row_end = "\n";
        let px_off = &PIXEL.bg_rgb::<0, 0, 0>().fg_rgb::<0, 0, 0>().to_string();
        let px_on = &PIXEL.fg_rgb::<105, 237, 44>().bg_rgb::<0, 0, 0>().to_string();

        let mut first_row = true;
        for row in self.pixels.iter().rev() {
            if !first_row {
                rendered += row_end;
            } else {
                first_row = false;
            }
            for px in row {
                rendered += if *px {
                    px_on
                } else {
                    px_off
                };
            }
        }

        //caching yay
        self.last_pixels = Some(self.pixels);
        self.last_render = Some(rendered.clone());

        rendered
    }

    pub fn render_to_tex(&mut self, texture: &mut Texture) {
        texture.with_lock(None, |buffer: &mut [u8], _pitch/* size of a row in bytes */: usize| {
            let mut buf_ptr = 0;
            for row in self.pixels.iter().rev() {
                for px in row {
                    if *px {
                        buffer[buf_ptr] = 255;
                        buffer[buf_ptr+1] = 255;
                        buffer[buf_ptr+2] = 255;
                    } else {
                        buffer[buf_ptr] = 0;
                        buffer[buf_ptr+1] = 0;
                        buffer[buf_ptr+2] = 0;
                    }
                    buf_ptr += 3
                }
            }
        }).expect("Rendered the current frame");
    }
}

pub struct ACEmulator {
    pub renderer: ACRenderer,
    memory: [u8; 4096],
    regs: [u8; 16],
    /// index register?
    i: u16,
    dt: u8,
    st: u8,
    pc: usize,
    stack: [u16; STACK_SIZE],
    stack_ptr: u8,
    /// Enable sound
    tone: bool,
    /// last time that the timer was decremented
    t_last: Instant,
    /// last time that a instruction was run
    i_last: Instant,
    /// paused untill a key is sent
    waiting_for_key: bool,
    waiting_for_key_reg: usize,
}

impl ACEmulator {
    pub fn new() -> Self {
        let mut mem = [0; 4096];
        for (i, sprite) in SPRITE_CHARS.iter().enumerate() {
            let p = SPRITE_CHARS_ADDR as usize + i * sprite.len();
            mem[p..p + sprite.len()].copy_from_slice(sprite)
        }
        Self {
            renderer: ACRenderer::new(),
            memory: mem,
            regs: [0; 16],
            i: 0,
            dt: 0,
            st: 0,
            pc: 0x200 as usize,
            stack: [0; 0x10],
            stack_ptr: 0,
            tone: false,
            t_last: Instant::now(),
            i_last: Instant::now(),
            waiting_for_key: false,
            waiting_for_key_reg: 0,//this will always be set to something before it is needed
        }

    }

    pub fn should_bleep(&self) -> bool {
        self.tone
    }

    /// must be called 60 times per second
    pub fn update(&mut self, keypad: &crate::keyboard::ACKeyboard, new_keypress: Option<ACKey>) {
        log::debug!("updating");
        let now = Instant::now();
        if (now - self.t_last).as_nanos()  > 16_666_666 {//60hz
            log::debug!("updating timer");
            if self.dt != 0 {
                self.dt -= 1;
            }
            self.tone = if self.st != 0 {
                self.st -= 1;
                true
            } else {
                false
            };
            self.t_last = now;
        }
        if (now - self.i_last).as_millis() > 10 && !self.waiting_for_key {
            log::debug!("running instruction");
            if self.pc + 1 == self.memory.len() {
                panic!("Reached the end");
            }
            let instr: u16 = ((self.memory[self.pc] as u16) << 8) | self.memory[self.pc + 1] as u16;
            self.pc += 2;
            self.exec_oper(instr, keypad);
            self.i_last = now;
        } else if self.waiting_for_key {
            if let Some(key) = new_keypress {
                self.regs[self.waiting_for_key_reg] = key.to_hex();
                self.waiting_for_key = false;
            }
        };
    }

    pub fn exec_oper(&mut self, instr: u16, keypad: &crate::keyboard::ACKeyboard) {
        let code = instr & 0xF000;
        let x = ((instr & 0x0F00) >> 8) as usize;
        let y = ((instr & 0x00F0) >> 4) as usize;
        let n = instr & 0x000F;
        let nn = instr & 0x00FF;
        let nnn = instr & 0x0FFF;
        log::debug!("inst: {:4X} at {}, i:{}\nargs:\n{} {} {} {} {}", code, self.pc, self.i, x, y, n, nn, nnn);

        match code {
            0x0000 => {
                match n {
                    // 00E0 - CLS
                    0 => {
                        self.renderer.clear();
                    }
                    // 00EE - RET
                    0x0E => {
                        self.pc = self.stack[self.stack_ptr as usize] as usize;
                        if self.stack_ptr > 0 {
                            self.stack_ptr -= 1;
                        }
                    }
                    //http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#0nnn
                    _ => (),
                }
            }
            0x1000 => {
                //JMP addr
                self.pc = nnn as usize;
            }
            0x2000 => {
                // call at nn
                self.stack_ptr += 1;
                self.stack[self.stack_ptr as usize] = self.pc as u16;
                self.pc = nnn as usize;
            }
            0x3000 => {
                // skip next if Vx = nn
                if self.regs[x] as u16 == nn {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // skip next if Vx != nn
                if self.regs[x] as u16 != nn {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // skip next if Vx == Vy
                if self.regs[x] == self.regs[y] {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // put nn into Vx
                self.regs[x] = nn as u8;
            }
            0x7000 => {
                // add Vx to nn and store in x
                self.regs[x] = (self.regs[x] as u16 + nn) as u8;
            }
            0x8000 => {
                match n {
                    // 8XY0 - LD VX, VY
                    0x00 => self.regs[x] = self.regs[y],
                    // 8XY1 - OR VX, VY
                    0x01 => self.regs[x] = self.regs[x] | self.regs[y],
                    // 8XY2 - AND VX, VY
                    0x02 => self.regs[x] = self.regs[x] & self.regs[y],
                    // 8XY3 - XOR VX, VY
                    0x03 => self.regs[x] = self.regs[x] ^ self.regs[y],
                    // 8XY4 - ADD VX, VY
                    0x04 => {
                        let res = self.regs[x] as usize + self.regs[y] as usize;
                        if res > 255 {
                            // Carry to VF
                            self.regs[0x0F] = 1;
                        } else {
                            self.regs[0x0F] = 0;
                        }
                        self.regs[x] = res as u8;
                    }
                    // 8XY5 - SUB VX, VY
                    0x05 => {
                        self.regs[0x0F] = if self.regs[x] > self.regs[y] {
                            // Carry to VF
                            1
                        } else {
                            0
                        };
                        self.regs[x] =
                            (self.regs[x] as i32 - self.regs[y] as i32) as u8;
                    }
                    // 8XY6 - SHR VX {, VY}
                    0x06 => {
                        self.regs[0x0F] = self.regs[x] & 0x01;
                        self.regs[x] /= 2;
                    }
                    // 8XY7 - SUBN VX, VY
                    0x07 => {
                        self.regs[0x0F] = if self.regs[y] > self.regs[x] {
                            1
                        } else {
                            0
                        };
                        self.regs[x] = self.regs[y] - self.regs[x];
                    }
                    // 8XYE - SHL VX {, VY}
                    0x0E => {
                        self.regs[0x0F] = self.regs[x] & 0x80;
                        self.regs[x] = (self.regs[x] as u16 * 2) as u8;
                    }
                    // Default
                    _ => (),
                }
            }
            0x9000 => {
                // skip next if Vx != Vy
                if self.regs[x] != self.regs[y] {
                    self.pc += 2;
                }
            }
            0xA000 => {
                // set the index to nnn
                self.i = nnn;
            }
            0xB000 => {
                self.pc = nnn as usize + self.regs[0] as usize;
            }
            0xC000 => {
                // generate a random num from 0-255, and store that & nn in reg x
                self.regs[x] = nn as u8 & rand::random::<u8>();
            }
            0xD000 => {
                //& Draw instruction
                self.regs[0x0F] = 0;
                let xpos: usize = self.regs[x] as usize % crate::SCREEN_WIDTH as usize;
                let ypos: usize = self.regs[y] as usize % crate::SCREEN_HEIGHT as usize;
                for row in 0..n {
                    // Fetch bits
                    let bits: u8 = self.memory[(self.i + row) as usize];
                    // Current Y
                    let cy = (ypos + row as usize) % crate::SCREEN_HEIGHT as usize;
                    // Loop over bits
                    for col in 0..8_usize {
                        // Current X
                        let cx = (xpos + col) % crate::SCREEN_WIDTH as usize;
                        let mask: u8 = 0x01 << 7 - col;
                        let color = (bits & mask) >> 7 - col;

                        self.renderer.xor_pixel(cx.try_into().unwrap(), cy.try_into().unwrap(),
                            if color == 1 {
                                true
                            } else if color == 0 {
                                false
                            } else {
                                panic!()
                            }
                        );

                        if cx == crate::SCREEN_WIDTH as usize - 1 {
                            // Reached the right edge
                            // eprintln!("Reached right edge");
                            break;
                        }
                    }
                    if cy == crate::SCREEN_HEIGHT as usize - 1 {
                        // Reached the bottom edge
                        // eprintln!("Reached bottom edge");
                        break;
                    }
                }
            }
            0xE000 => {
                match nn {
                    0x9E => {
                        // skip next if a key on the keyboard with the value Vx is pressed
                        if keypad.is_pressed(&ACKey::from_hex(self.regs[x]).unwrap()) {
                            self.pc += 2;
                        }
                    }
                    0xA1 => {
                        // skip next if a key on the keyboard with the value Vx is NOT pressed
                        if !keypad.is_pressed(&ACKey::from_hex(self.regs[x]).unwrap()) {
                            self.pc += 2;
                        }
                    }
                    _ => (),
                }
            }
            0xF000 => {
                match nn {
                    // FX07 set Vx to delay timer
                    0x07 => self.regs[x] = self.dt,
                    0x0A => {
                        // pause untill a kepress has occured, storing the key in Vx is handled elswhere
                        self.waiting_for_key = true;
                        self.waiting_for_key_reg = x as usize;
                    }
                    // FX15 set delay timer to Vx
                    0x15 => self.dt = self.regs[x],
                    // FX18 set sound timer to Vx
                    0x18 => self.st = self.regs[x],
                    // FX1E set the index register to itself plus Vx
                    0x1E => self.i = self.i + self.regs[x] as u16,
                    // FX29 set I to location of sprite for digit VX
                    0x29 => self.i = self.regs[x] as u16 * 0x05,
                    // FX33 store BCD representation of VX in I, I+1 and I+2
                    0x33 => {
                        let num = self.regs[x];
                        let h = num / 100;
                        let t = (num - h * 100) / 10;
                        let o = num - h * 100 - t * 10;
                        let i = self.i as usize;
                        self.memory[i] = h;
                        self.memory[i + 1] = t;
                        self.memory[i + 2] = o;
                    }
                    // FX55 set memory starting at I to values in V0 to VX
                    0x55 => {
                        let n: usize = x;
                        for reg in 0..n + 1 {
                            self.memory[self.i as usize + reg] = self.regs[reg];
                        }
                    }
                    // FX65 set registers V0 to VX to memory starting at I
                    0x65 => {
                        let n: usize = x;
                        for reg in 0..n + 1 {
                            self.regs[reg] = self.memory[self.i as usize + reg];
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        for (i, v) in rom.iter().enumerate() {
            self.memory[self.pc+i] = *v;
        }
    }
}