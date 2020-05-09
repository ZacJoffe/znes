use sdl2::keyboard::Scancode;

use crate::cpu::CPU;
use crate::ppu::{PPU, Color};
use crate::cartridge::get_mapper;
use crate::controller;

use std::time::{Instant, Duration};
use std::thread::sleep;
use std::collections::HashSet;

use crate::PIXEL_WIDTH;
use crate::PIXEL_HEIGHT;

// isolate the actual nes console into it's own struct
//
// do not handle anything to do with sdl in here as defeats it the purpose of having a class to
// abstract the elements of the console
pub struct NES {
    // the cpu contains an instance of the ppu, connected by the "main bus" (cpu read and write methods)
    // TODO - APU implementation
    pub cpu: CPU,

    pub screen_buffer: Vec<u8>,

    scaling: u32,
    timer: Instant,
}

impl NES {
    pub fn new(buffer: Vec<u8>, scaling: u32) -> NES {
        let mapper = get_mapper(buffer);
        let ppu = PPU::new(mapper.clone());

        NES {
            cpu: CPU::new(mapper.clone(), ppu),

            screen_buffer: vec![0; (PIXEL_WIDTH * scaling * 3 * PIXEL_HEIGHT * scaling) as usize],

            scaling: scaling,
            timer: Instant::now()
        }
    }

    pub fn step_cpu(&mut self) -> u64 {
        self.cpu.step()
    }

    pub fn step_ppu(&mut self) {
        let pixel = self.cpu.ppu.step();

        if let Some((x, y, color)) = pixel {
            let Color(r, g, b) = color;
            // 3 bytes per pixel, 256 pixels horizontally
            let y_offset = y * (3 * PIXEL_WIDTH * self.scaling * self.scaling) as usize;
            for i in 0..self.scaling {
                let row_offset = y_offset + (3 * PIXEL_WIDTH * self.scaling * i) as usize;
                let x_offset = x * (3 * self.scaling) as usize;
                for j in 0..self.scaling {
                    let col_offset = x_offset + (j * 3) as usize;
                    let offset = row_offset + col_offset;

                    self.screen_buffer[offset] = r;
                    self.screen_buffer[offset + 1] = g;
                    self.screen_buffer[offset + 2] = b;
                }
            }
        }
    }

    // sleep the thread if running to quickly
    pub fn limit_framerate(&mut self) {
        let now = Instant::now();
        if now < self.timer + Duration::from_millis(1000 / 60) {
            sleep(self.timer + Duration::from_millis(1000/60) - now);
        }
        self.timer = Instant::now();
    }

    pub fn poll_inputs(&mut self, scancodes: HashSet<Scancode>) {
        let mut buttons = 0;
        for scancode in scancodes.iter() {
            match scancode {
                // Controls:
                // Z - A
                // X - B
                // Backspace - Select
                // Enter (Return) - Start
                // Up - Up
                // Down - Down
                // Left - Left
                // Right - Right
                Scancode::Z => buttons |= 1 << controller::A_INDEX,
                Scancode::X => buttons |= 1 << controller::B_INDEX,
                Scancode::Backspace => buttons |= 1 << controller::SELECT_INDEX,
                Scancode::Return => buttons |= 1 << controller::START_INDEX,
                Scancode::Up => buttons |= 1 << controller::UP_INDEX,
                Scancode::Down => buttons |= 1 << controller::DOWN_INDEX,
                Scancode::Left => buttons |= 1 << controller::LEFT_INDEX,
                Scancode::Right => buttons |= 1 << controller::RIGHT_INDEX,
                _ => {}
            }
        }

        self.cpu.controllers[0].set_buttons(buttons);
    }
}
