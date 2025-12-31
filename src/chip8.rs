use std::fmt;

use std::{fs, io, path::Path};
use log::trace;
use sdl2::audio::{AudioCallback, AudioDevice};
use sdl2::rect::Rect;
use sdl2::render::{Canvas, RenderTarget};
use log::*;

use crate::{BACKGROUND, FOREGROUND};
use crate::timestep::FixedTimestep;

pub struct Chip8 {
    registers: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    pc: u16,
    stack: [u8; 16],
    sp: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub keypad: [u8; 16],
    pub video: [u8; 64 * 32],
    opcode: u16,
}

// Static helpers
static FONTSET_SIZE: usize = 80;
static FONT_HEIGHT: usize = 5;
static FONTSET: [u8; FONTSET_SIZE] =
[
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
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

static START_ADDRESS: usize = 0x200;
static FONTSET_START_ADDRESS: usize = 0x50;

impl Chip8 {
    pub fn new() -> Self {
        let mut memory = [0; 4096];

        for i in 0..FONTSET_SIZE {
            memory[FONTSET_START_ADDRESS + i] = FONTSET[i];
        }

        Chip8 
        {
            registers   : [0; 16],
            memory,
            index       : 0,
            pc          : 0,
            stack       : [0; 16],
            sp          : 0,
            delay_timer : 0,
            sound_timer : 0,
            keypad      : [0; 16],
            video       : [0; 64 * 32],
            opcode      : 0,
        }
    }

    // Update in main loop
    pub fn update<T, U>(&mut self, canvas: &mut Canvas<U>, device: &AudioDevice<T>, scale: f32) 
    where T: AudioCallback, U: RenderTarget {
        if self.sound_timer == 0 {
            device.pause();
        }

        for (i, &element) in self.video.iter().enumerate() {
            let x: i32 = (i % 64) as i32;
            let y: i32 = (i / 64) as i32;
            if element != 0 {
                canvas.set_draw_color(FOREGROUND);
                let _ = canvas.fill_rect(Rect::new(x * crate::WIDTH as i32 / 64, y * crate::HEIGHT as i32 / 32, crate::WIDTH / 64, crate::HEIGHT / 32));
                canvas.set_draw_color(BACKGROUND);
            }
        }

    }

    pub fn update_timers(&mut self) {
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }



    pub fn load_rom<P>(&mut self, path: P) -> Result<(), io::Error> 
    where P: AsRef<Path> {
        let data = fs::read(path)?;
        // Split memory array to use copy method for data
        let (_, data_section) = self.memory.split_at_mut(START_ADDRESS as usize);
        let (program_data, _) = data_section.split_at_mut(data.len());
        program_data.copy_from_slice(&data);
        // Update pc
        self.pc = START_ADDRESS as u16;
        
        Ok(())
    }


    fn draw_sprite(&mut self, x: usize, y: usize, bytes: &[u8]) {
        let height = bytes.len();
        for y_pos in 0..height {
            for x_pos in 0..8 as usize {
                let b: u8 = bytes[y_pos] & (0x80 >> x_pos);
                let x_pos = (x_pos + x) % 64;
                let y_pos = (y_pos + y) % 32;
                self.video[x_pos + y_pos * 64] |= b;
            }
        }

    }

    pub fn font_test(&mut self) {
        let W = 7 as usize;
        for font_index in 0..16 as usize {
            let font_start = FONTSET_START_ADDRESS + font_index * FONT_HEIGHT;
            let bytes : Vec<u8> = self.memory[font_start..font_start + FONT_HEIGHT].to_vec();
            self.draw_sprite(5 + (font_index % W) * W, 1 + (font_index / W) * 8, bytes.as_slice());

        }

    }

    pub fn cycle(&mut self) {
        let pc_index: usize = self.pc as usize;
        trace!("Fetching instruction {:02X}{:02X} from {:04X}", self.memory[pc_index], self.memory[pc_index + 1], self.pc);
        self.opcode = (self.memory[pc_index] as u16) << 8 | (self.memory[pc_index + 1] as u16);
        trace!("{:04X}", self.opcode);
        self.pc += 2;
    }

}

impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "CHIP-8 State:\n")?;
        writeln!(f, "Registers:\n")?;
        for i in 0..16 {
            writeln!(f, "V{0:X} = {1}", i, self.registers[i])?;
        }
        writeln!(f, "Index: {:X}", self.index)?;
        writeln!(f, "Program Counter: {:X}", self.pc)?;
        writeln!(f, "Stack Pointer: {:X}", self.sp)?;
        writeln!(f, "Delay Timer: {}", self.delay_timer)?;
        writeln!(f, "Sound Timer: {}", self.sound_timer)?;
        writeln!(f, "Opcode: {:X}", self.opcode)?;
        write!(f, "")
    }
}