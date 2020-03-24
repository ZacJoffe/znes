mod cpu;
mod cartridge;

use cpu::CPU;
use cartridge::{get_maper};

use std::rc::Rc;
use std::cell::RefCell;

struct Nes {
    cpu: CPU,
    mapper: Rc<RefCell<dyn Mapper>>,
    ram: Vec<u8>
}

impl Nes {
    pub fn new(file_path: String) -> Nes {
        Nes {
            cpu: CPU::new(),
            mapper: get_mapper(file_path),
            ram: Vec![0; 2048]
        }
    }
}
