struct PPU {
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

    nametable_byte: u8,
    attribute_table_byte: u8,
    low_tile_byte: u8,
    high_tile_byte: u8,
    tile_data: u64,
}
