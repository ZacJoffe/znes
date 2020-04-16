use crate::cartridge::Mapper;

use std::rc::Rc;
use std::cell::RefCell;

pub struct PPU {
    cycle: i32,
    scanline: i32,
    frame: u64,

    // registers
    v: u16,
    t: u16,
    x: u8,
    w: u8,
    f: u8,

    nametable_data: [u8; 0x800],
    palette_data: [u8; 0x20],
    oam_data: [u8; 0x100],

    mapper: Rc<RefCell<dyn Mapper>>,

    nametable_byte: u8,
    attribute_table_byte: u8,
    low_tile_byte: u8,
    high_tile_byte: u8,
    tile_data: u64,
}

impl PPU {
    pub fn new(mapper: Rc<RefCell<dyn Mapper>>) -> PPU {
        PPU {
            cycle: 0,
            scanline: 0,
            frame: 0,

            v: 0,
            t: 0,
            x: 0,
            w: 0,
            f: 0,

            mapper: mapper,

            nametable_data: [0; 0x800],
            palette_data: [0; 0x20],
            oam_data: [0; 0x100],

            nametable_byte: 0,
            attribute_table_byte: 0,
            low_tile_byte: 0,
            high_tile_byte: 0,
            tile_data: 0,
        }
    }

    fn read_status(&self) -> u8 {
        // TODO
        0
    }

    fn read_oam_data(&self) -> u8 {
        // TODO
        0
    }

    fn read_data(&self) -> u8 {
        // TODO
        0
    }

    pub fn read_register(&self, address: usize) -> u8 {
        match address {
            0x2002 => self.read_status(),
            0x2004 => self.read_oam_data(),
            0x2007 => self.read_data(),
            _ => 0
        }
    }

    pub fn write_register(&mut self, address: usize, value: u8) {
        match address {
            0x2000 => {},
            0x2001 => {},
            0x2003 => {},
            0x2004 => {},
            0x2005 => {},
            0x2006 => {},
            0x2007 => {},
            0x4014 => {},
            _ => panic!("Invalid PPU register write! 0x{:x}", address)
        }
    }
}
