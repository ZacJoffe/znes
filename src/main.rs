extern crate sdl2;

mod cpu;
mod cartridge;
mod ppu;
mod controller;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use cpu::CPU;
use ppu::PPU;
use cartridge::{Cartridge, get_mapper};

use std::env;
use std::path::PathBuf;
use std::path::Path;
use std::fs;

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

    println!("{:x?}", buffer);

    // initialize sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

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

    'running: loop {
        let cpu_cycles = cpu.step();
        let ppu_cycles = cpu_cycles * 3;

        for _ in 0..ppu_cycles {
            cpu.ppu.step();
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                _ => {}
            }
        }
    }
}
