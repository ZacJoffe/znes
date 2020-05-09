use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;

use sdl2::render::Canvas;
use sdl2::render::Texture;

use crate::cpu::CPU;
use crate::ppu::{PPU, Color};
use crate::cartridge::{Cartridge, get_mapper};

use std::rc::Rc;
use std::cell::RefCell;

use std::time::{Instant, Duration};
use std::thread::sleep;
use std::collections::HashSet;

use crate::PIXEL_WIDTH;
use crate::PIXEL_HEIGHT;

struct Nes {
    pub cpu: CPU,
    pub ppu: PPU,

    pub screen_buffer: Vec<u8>,

    scaling: u32,
    timer: Instant,
}

impl Nes {
    pub fn new(buffer: Vec<u8>, scaling: u32) -> Nes {
        let mapper = get_mapper(buffer);
        let ppu = PPU::new(mapper.clone());
        let cpu = CPU::new(mapper.clone(), ppu);


        Nes {
            cpu: cpu,
            ppu: ppu,

            screen_buffer: vec![0; (PIXEL_WIDTH * scaling * 3 * PIXEL_HEIGHT * scaling) as usize],

            scaling: scaling,
            timer: Instant::now()
        }
    }

    pub fn step(&mut self) {
        let cpu_cycles = self.cpu.step();
        let ppu_cycles = cpu_cycles * 3;

        for _ in 0..ppu_cycles {
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
    }

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

            }
        }

        self.cpu.controllers[0].set_buttons(buttons);
    }
}
