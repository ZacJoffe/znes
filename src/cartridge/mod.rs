struct Cartridge {
    prg: Vec<u8>,
    chr: Vec<u8>,

    mapper: u8,
    mirror: u8,
    battery: u8
}

impl Cartridge {
    pub fn new(prg: Vec<u8>, chr: Vec<u8>, mapper: u8, mirror: u8, battery: u8) -> Cartridge {
        Cartridge {
            prg: prg,
            chr: chr,
            mapper: mapper,
            mirror: mirror,
            battery: battery
        }
    }
}

// https://wiki.nesdev.com/w/index.php/INES
struct NesFileHeader {
    signature: u32,
}
