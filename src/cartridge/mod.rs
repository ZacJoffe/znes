mod mapper0;

use mapper0::Nrom;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug)]
enum Mirror {
    Horizontal,
    Vertical,
    Single0,
    Single1,
    Four,
}

pub trait Mapper {
    fn read(&self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8);
    fn get_mirror(&self) -> Mirror;
    fn step(&mut self);
}

#[derive(Debug)]
pub struct NesHeader {
    prg_rom_size: usize,
    chr_rom_size: usize,
    mirror: Mirror,
    battery_backed_ram: bool,
    trainer: bool,
    ignore_mirror: bool
}

#[derive(Debug)]
pub struct Cartridge {
    header: NesHeader,
    prg: Vec<Vec<u8>>, // chunks of prg rom (16 KiB chunks)
    chr: Vec<Vec<u8>>, // chunks of chr rom (8 KiB chunks)
    mapper: u8
}

pub fn get_mapper(buffer: Vec<u8>) -> Rc<RefCell<dyn Mapper>> {
    let cart = Cartridge::new(buffer);
    match cart.mapper {
        0 => Rc::new(RefCell::new(Nrom::new(cart))),
        _ => panic!("Unimplemented mapper!")
    }
}

impl Cartridge {
    fn new(buffer: Vec<u8>) -> Cartridge {
        let ines_signature = [0x4e, 0x45, 0x53, 0x1a];

        // https://wiki.nesdev.com/w/index.php/INES
        if buffer[0..4] != ines_signature {
            panic!("Incorrect file signature!");
        }

        let flags6 = buffer[6];
        let flags7 = buffer[7];

        let mapper = (flags7 & 0xf0) | flags6 >> 4;
        let mirror = if flags6 & 0x1 != 0 { Mirror::Vertical } else { Mirror::Horizontal };

        let header = NesHeader {
            prg_rom_size: buffer[4] as usize,
            chr_rom_size: buffer[5] as usize,
            mirror: mirror,
            battery_backed_ram: flags6 & 0x2 != 0,
            trainer: flags6 & 0x4 != 0,
            ignore_mirror: flags6 & 0x8 != 0
        };

        //let prg_ram_size = buffer[8];

        let mut cart = Cartridge {
            header: header,
            prg: Vec::new(),
            chr: Vec::new(),
            mapper: mapper
        };

        let prg_chunk = 1 << 14; // 16 KiB
        let chr_chunk = 1 << 13; // 8 KiB

        let prg_offset = 0x10 + if cart.header.trainer { 0x200 } else { 0 };
        let chr_offset = prg_offset + (cart.header.prg_rom_size * prg_chunk);

        for i in 0..cart.header.prg_rom_size {
            let offset = prg_offset + (i * prg_chunk);
            cart.prg.push(buffer[offset..(offset + prg_chunk)].to_vec());
        }

        for i in 0..cart.header.chr_rom_size {
            let offset = chr_offset + (i * chr_chunk);
            cart.chr.push(buffer[offset..(offset + chr_chunk)].to_vec());
        }

        println!("{:x?}", cart);
        cart
    }
}
