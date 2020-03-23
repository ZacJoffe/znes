use std::fs::File;
use std::path::Path;

enum Mirror {
    Horizontal,
    Vertical,
}

pub trait Mapper {
    fn read(&self, address: usize) -> u8;
    fn write(&mut self, address: usize, value: u8);
    fn step(&mut self);
}

pub struct NesHeader {
    prg_rom_size: usize,
    chr_rom_size: usize,
    mirror: Mirror,
    battery_backed_ram: bool,
    trainer: bool,
    ignore_mirror: bool
}

pub struct Cartridge {
    header: NesHeader,
    prg: Vec<u8>,
    chr: Vec<u8>,
    mapper: u8
}

let ines_signature = [0x4e, 0x45, 0x53, 0x1a];

impl Cartridge {
    pub fn new(file_path: String) -> Cartridge {
        let mut f = File::open(Path::new(&file_path)).expect("Could not open rom!");
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer).unwrap();

        // https://wiki.nesdev.com/w/index.php/INES
        if buffer[0..4] != ines_signature {
            panic!("Incorrect file signature!");
        }

        let flags6 = buffer[6];
        let flags7 = buffer[7];

        let mapper = (flags7 && 0xf0) | flags6 >> 4;
        let mirror = if flags6 & 0x1 != 0 { Mirror::Vertical } else { Mirror::Horizontal };

        let header = Header {
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

        let prg_chunk = 1 << 14;
        let chr_chunk = 1 << 13;

        let prg_offset = 0x10 + if cart.header.trainer { 0x200 } else { 0 };
        let chr_offset = prg_offset + (cart.header.prg_rom_size + prg_chunk);

        for i in 0..cart.header.prg_rom_size {
            let offset = prg_offset + (i * prg_chunk);
            cart.prg.push(buffer[offset..(offset + prg_chunk)].to_vec());
        }

        for i 0..cart.header.chr_rom_size {
            let offset = chr_offset + (i * chr_chunk);
            cart.chr.push(buffer[offset..(offset + chr_chunk)].to_vec());
        }

        cart
    }
}
