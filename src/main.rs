mod cpu;
mod cartridge;

use cpu::CPU;
use cartridge::Cartridge;

use std::env;
use std::path::PathBuf;
use std::path::Path;

fn main() {
    // let _cpu = CPU::new();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!("No ROM given as argument!");
    }

    // let rom_path = PathBuf::from(&args[1]);
    let filepath = Path::new(&args[1]).to_path_buf();
    println!("{}", filepath.to_str().unwrap());
}

