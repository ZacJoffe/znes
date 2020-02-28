struct Cartridge {
    prg: Vec<u8>,
    chr: Vec<u8>,
    sram: Vec<u8>,

    mapper: u8,
    mirror: u8,
    battery: u8
}
