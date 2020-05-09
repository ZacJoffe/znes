mod cpu;
mod cartridge;

use cpu::CPU;
use ppu::PPU;
use cartridge::{get_maper};

use std::rc::Rc;
use std::cell::RefCell;

struct Nes {
    pub cpu: CPU,
    pub ppu: PPU,
}

impl Nes {
    pub fn new(buffer: Vec<u8>) -> Nes {
        let mapper = get_mapper(buffer);
        let ppu = PPU::new(mapper.clone());
        let mut cpu = CPU::new(mapper.clone(), ppu);

        Nes {
            cpu: CPU::new(mapper.clone(), ppu),
            ppu: PPU::new(mapper.clone())
        }
    }

    pub fn step() {

    }
}
