extern crate sdl2;
extern crate clap;
extern crate cpuprofiler;

mod cpu;
mod cartridge;
mod ppu;
mod controller;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;

use clap::{Arg, App};

use cpuprofiler::PROFILER;

use cpu::CPU;
use ppu::{PPU, Color};
use cartridge::{Cartridge, get_mapper};

use std::env;
use std::path::PathBuf;
use std::path::Path;
use std::fs;
use std::time::{Instant, Duration};
use std::thread::sleep;
use std::collections::HashSet;

const PIXEL_WIDTH: u32 = 256;
const PIXEL_HEIGHT: u32 = 240;

fn main() {
    let matches = App::new("znes")
        .arg(
            Arg::with_name("file") // positional argument
                .about("The .nes file to be ran by the emulator")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name("scale") // scaling factor
                .short('s')
                .takes_value(true)
                .about("Resolution scaling factor, defaults to 3"),
        )
        .arg(
            Arg::with_name("debug") // debug flag
                .short('d')
                .multiple(false)
                .about("Turn debugging information on"),
        )
        .get_matches();

    let file = matches.value_of("file").unwrap();
    let buffer = fs::read(file);
    let buffer = match buffer {
        Ok(b) => b,
        Err(_) => panic!("Cannot load rom! {}", file)
    };

    let scaling = matches.value_of_t("scale").unwrap_or(3);

    let debug_mode = match matches.occurrences_of("debug") {
        1 => true,
        _ => false
    };

    // println!("{:x?}", buffer);

    // initialize sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("znes", PIXEL_WIDTH * scaling, PIXEL_HEIGHT * scaling).position_centered().build().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_streaming(sdl2::pixels::PixelFormatEnum::RGB24, PIXEL_WIDTH * scaling, PIXEL_HEIGHT * scaling).unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mapper = get_mapper(buffer);
    let ppu = PPU::new(mapper.clone());
    let mut cpu = CPU::new(mapper.clone(), ppu);

    let mut screen_buffer = vec![0; (PIXEL_WIDTH * scaling * 3 * PIXEL_HEIGHT * scaling) as usize];

    let mut timer = Instant::now();

    if debug_mode {
        PROFILER.lock().unwrap().start("./znes.profile").unwrap();
    }

    'running: loop {
        let cpu_cycles = cpu.step();
        let ppu_cycles = cpu_cycles * 3;

        for _ in 0..ppu_cycles {
            let pixel = cpu.ppu.step();

            if let Some((x, y, color)) = pixel {
                let Color(r, g, b) = color;
                // 3 bytes per pixel, 256 pixels horizontally
                let y_offset = y * (3 * PIXEL_WIDTH * scaling * scaling) as usize;
                for i in 0..scaling {
                    let row_offset = y_offset + (3 * PIXEL_WIDTH * scaling * i) as usize;
                    let x_offset = x * (3 * scaling) as usize;
                    for j in 0..scaling {
                        let col_offset = x_offset + (j * 3) as usize;
                        let offset = row_offset + col_offset;

                        screen_buffer[offset] = r;
                        screen_buffer[offset + 1] = g;
                        screen_buffer[offset + 2] = b;
                    }
                }
            }

            if cpu.ppu.end_of_frame {
                // println!("{:?}", screen_buffer);
                texture.update(None, &screen_buffer, (PIXEL_WIDTH * 3 * scaling) as usize).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();

                let now = Instant::now();
                if now < timer + Duration::from_millis(1000 / 60) {
                    sleep(timer + Duration::from_millis(1000/60) - now);
                }
                timer = Instant::now();
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

    if debug_mode {
        PROFILER.lock().unwrap().stop().unwrap();
    }
}
