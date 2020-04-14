mod cpu;
mod cartridge;

use cpu::CPU;
use cartridge::Cartridge;

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
}
