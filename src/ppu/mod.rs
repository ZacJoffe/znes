use crate::cartridge::Mapper;

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
    pub fn new(mapper: RC<RefCell<dyn Mapper>>) -> PPU {
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
}
