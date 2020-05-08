extern crate sdl2;

mod cpu;
mod cartridge;
mod ppu;
mod controller;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;

use cpu::CPU;
use ppu::{PPU, Color};
use cartridge::{Cartridge, get_mapper};

use std::env;
use std::path::PathBuf;
use std::path::Path;
use std::fs;
use std::time::Instant;
use std::collections::HashSet;

fn main() {
    // let _cpu = CPU::new();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("No ROM given as argument!");
    }

    let buffer = fs::read(&args[1]);
    let buffer = match buffer {
        Ok(b) => b,
        Err(_) => panic!("Cannot load rom! {}", &args[1])
    };

    // println!("{:x?}", buffer);

    // initialize sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // TODO - make resolutions const with scaling
    let window = video_subsystem.window("znes", 256, 240).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, 256, 240).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mapper = get_mapper(buffer);
    let ppu = PPU::new(mapper.clone());
    let mut cpu = CPU::new(mapper.clone(), ppu);

    let mut screen_buffer = vec![0; 256 * 3 * 240];

    let timer = Instant::now();

    'running: loop {
        let cpu_cycles = cpu.step();
        let ppu_cycles = cpu_cycles * 3;

        for _ in 0..ppu_cycles {
            let pixel = cpu.ppu.step();
            /*
            match pixel {
                Some(p) => println!("{:?}", p),
                None => ()
            }
            */

            if let Some((x, y, color)) = pixel {
                let Color(r, g, b) = color;
                // 3 bytes per pixel, 256 pixels horizontally
                let y_offset = y * 3 * 256;
                let x_offset = x * 3;
                let offset = y_offset + x_offset;

                // let row_offset = y_offset;
                // let col_offset = x_offset;
                // let offset = row_offset + col_offset;

                // println!("{}", offset);
                screen_buffer[offset] = r;
                screen_buffer[offset + 1] = g;
                screen_buffer[offset + 2] = b;
            }

            if cpu.ppu.end_of_frame {
                // println!("{:?}", screen_buffer);
                // panic!("MOSS");
                texture.update(None, &screen_buffer, 256 * 3).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        // handle inputs if the strobe is high
        if cpu.controllers[0].strobe & 1 != 0 {
            // get a set of all pressed keys at any given time
            //
            // technically all keys can be pressed at once, even though on the physical hardware
            // pressing two opposite directions on the d-pad isn't possible
            let scancodes: HashSet<Scancode> = event_pump.keyboard_state().pressed_scancodes().collect();
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

            cpu.controllers[0].set_buttons(buttons);
        }
    }
}
