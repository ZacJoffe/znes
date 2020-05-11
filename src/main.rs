extern crate sdl2;
extern crate clap;
extern crate cpuprofiler;

mod cpu;
mod cartridge;
mod ppu;
mod controller;
mod nes;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use clap::{Arg, App};

use cpuprofiler::PROFILER;

use nes::NES;

pub const PIXEL_WIDTH: u32 = 256;
pub const PIXEL_HEIGHT: u32 = 240;

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

    let scaling = matches.value_of_t("scale").unwrap_or(3);

    let debug_mode = match matches.occurrences_of("debug") {
        1 => true,
        _ => false
    };

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

    let mut nes = NES::new(String::from(file), scaling);

    if debug_mode {
        PROFILER.lock().unwrap().start("./znes.profile").unwrap();
    }

    'running: loop {
        let cpu_cycles = nes.step_cpu();
        let ppu_cycles = cpu_cycles * 3;

        for _ in 0..ppu_cycles {
            nes.step_ppu();

            if nes.cpu.ppu.end_of_frame {
                // println!("{:?}", screen_buffer);
                texture.update(None, &nes.screen_buffer, (PIXEL_WIDTH * 3 * scaling) as usize).unwrap();
                canvas.copy(&texture, None, None).unwrap();
                canvas.present();

                nes.limit_framerate();
            }
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }

        // handle inputs if the strobe is high
        if nes.cpu.controllers[0].strobe & 1 != 0 {
            // get a set of all pressed keys at any given time
            //
            // technically all keys can be pressed at once, even though on the physical hardware
            // pressing two opposite directions on the d-pad isn't possible
            nes.poll_inputs(event_pump.keyboard_state().pressed_scancodes().collect());
        }
    }

    if debug_mode {
        PROFILER.lock().unwrap().stop().unwrap();
    }

    nes.save_battery();
}
