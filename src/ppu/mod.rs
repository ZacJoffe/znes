mod registers;
mod render;

use crate::cartridge::Mapper;
use crate::cartridge::Mirror;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Copy, Clone, Debug)]
pub struct Color(pub u8, pub u8, pub u8);

pub struct PPU {
    cycle: i32,
    scanline: usize,
    frame: u64,

    // registers
    v: u16,
    t: u16,
    x: u8,
    w: u8,
    f: u8,

    pub end_of_frame: bool, // signal the end of a frame that's ready for drawing

    nametable_data: [[u8; 0x400]; 2],
    palette_data: [u8; 0x20],

    oam_data: [u8; 0x100],
    secondary_oam: [u8; 0x20],

    mapper: Rc<RefCell<dyn Mapper>>,

    // background variables
    nametable_byte: u8,
    attribute_table_byte: u8,
    low_tile_byte: u8,
    high_tile_byte: u8,
    tile_data: u64,

    // sprite variables
    sprite_count: usize,
    sprite_attribute_latches: [u8; 8],
    sprite_positions: [u8; 8],
    sprite_indexes: [u8; 8],
    sprite_pattern_shift_regs: [(u8, u8); 8],

    // background shift registers
    //
    // "These contain the pattern table data for two tiles. Every 8 cycles, the data for
    // the next tile is loaded into the upper 8 bits of this shift register. Meanwhile,
    // the pixel to render is fetched from one of the lower 8 bits."
    //
    // https://wiki.nesdev.com/w/index.php/PPU_rendering#Preface
    pattern_shift_reg_low: u16,
    pattern_shift_reg_high: u16,

    // "These contain the palette attributes for the lower 8 pixels of the 16-bit shift
    // register. These registers are fed by a latch which contains the palette
    // attribute for the next tile."
    palette_shift_reg_low: u8,
    palette_shift_reg_high: u8,
    palette_latch: u8,

    oam_address: u8,

    pub data_buffer: u8, // "Least significant bits previously written into a PPU register"

    // flags
    // NMI flags
    nmi_previous: bool,
    nmi_output: bool,
    nmi_delay: u8,

    // trigger an NMI
    // check this flag in the CPU step function
    pub trigger_nmi: bool,

    // $2000 PPUCTRL
    flag_nametable: u8,
    increment: bool, // true => add 32, false => add 1
    flag_sprite_table: bool, // false => 0x0000, true => 0x1000
    flag_background_table: bool, // false => 0x0000, true => 0x1000
    flag_sprite_size: bool, // false => 8x8 pixels, true => 8x16 pixels
    flag_master_slave: bool,

    // $2001 PPUMASK
    grayscale: bool,
    show_left_background: bool,
    show_left_spries: bool,
    show_background: bool,
    show_sprites: bool,
    red_tint: bool,
    blue_tint: bool,
    green_tint: bool,

    // $2002 STATUS
    sprite_zero_hit: bool,
    sprite_overflow: bool,

    // $2007 PPUDATA
    read_buffer_data: u8,

    in_vblank: bool,

    // rgb color data
    palette_table: [Color; 0x40]
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

            end_of_frame: false,

            nametable_data: [[0; 0x400]; 2],
            palette_data: [0; 0x20],

            oam_data: [0; 0x100],
            secondary_oam: [0; 0x20],

            mapper: mapper,

            nametable_byte: 0,
            attribute_table_byte: 0,
            low_tile_byte: 0,
            high_tile_byte: 0,
            tile_data: 0,

            sprite_count: 0,
            sprite_attribute_latches: [0; 8],
            sprite_positions: [0; 8],
            sprite_indexes: [0; 8],
            sprite_pattern_shift_regs: [(0, 0); 8],

            pattern_shift_reg_low: 0,
            pattern_shift_reg_high: 0,

            palette_shift_reg_low: 0,
            palette_shift_reg_high: 0,
            palette_latch: 0,

            oam_address: 0,

            data_buffer: 0,

            nmi_previous: false,
            nmi_output: false,
            nmi_delay: 0,

            trigger_nmi: false,

            flag_nametable: 0,
            increment: false, // true => add 32, false => add 1
            flag_sprite_table: false,
            flag_background_table: false,
            flag_sprite_size: false,
            flag_master_slave: false,

            grayscale: false,
            show_left_background: false,
            show_left_spries: false,
            show_background: false,
            show_sprites: false,
            red_tint: false,
            blue_tint: false,
            green_tint: false,

            sprite_zero_hit: false,
            sprite_overflow: false,

            read_buffer_data: 0,

            in_vblank: false,

            // hardcoded https://wiki.nesdev.com/w/index.php/PPU_palettes#2C02
            palette_table: [
                Color(84, 84, 84), Color(0, 30, 116), Color(8, 16, 144), Color(48, 0, 136), Color(68, 0, 100), Color(92, 0, 48), Color(84, 4, 0), Color(60, 24, 0), Color(32, 42, 0), Color(8, 58, 0), Color(0, 64, 0), Color(0, 60, 0), Color(0, 50, 60), Color(0, 0, 0), Color(0, 0, 0), Color(0, 0, 0),
                Color(152, 150, 152), Color(8, 76, 196), Color(48, 50, 236), Color(92, 30, 228), Color(136, 20, 176), Color(160, 20, 100), Color(152, 34, 32), Color(120, 60, 0), Color(84, 90, 0), Color(40, 114, 0), Color(8, 124, 0), Color(0, 118, 40), Color(0, 102, 120), Color(0, 0, 0), Color(0, 0, 0), Color(0, 0, 0),
                Color(236, 238, 236), Color(76, 154, 236), Color(120, 124, 236), Color(176, 98, 236), Color(228, 84, 236), Color(236, 88, 180), Color(236, 106, 100), Color(212, 136, 32), Color(160, 170, 0), Color(116, 196, 0), Color(76, 208, 32), Color(56, 204, 108), Color(56, 180, 204), Color(60,  60,  60), Color(0, 0, 0), Color(0, 0, 0),
                Color(236, 238, 236), Color(168, 204, 236), Color(188, 188, 236), Color(212, 178, 236), Color(236, 174, 236), Color(236, 174, 212), Color(236, 180, 176), Color(228, 196, 144), Color(204, 210, 120), Color(180, 222, 120), Color(168, 226, 144), Color(152, 226, 180), Color(160, 214, 228), Color(160, 162, 160), Color(0, 0, 0), Color(0, 0, 0)
            ]
        }
    }

    pub fn clock(&mut self) {
        if self.cycle == 339 && self.scanline == 261 && self.frame % 2 == 1 {
            self.cycle = 0;
            self.scanline = 0;
            self.frame = self.frame.wrapping_add(1);
        } else if self.cycle == 340 && self.scanline == 261 {
            self.cycle = 0;
            self.scanline = 0;
            self.frame = self.frame.wrapping_add(1);
        } else if self.cycle == 340 {
            self.cycle = 0;
            self.scanline += 1;
        } else {
            self.cycle += 1;
        }
    }

    pub fn step(&mut self) -> Option<(usize, usize, Color)> {
        // println!("CYCLE: {} SCANLINE: {} FRAME: {}", self.cycle, self.scanline, self.frame);

        // handle nmi delays
        if self.nmi_delay > 0 {
            self.nmi_delay -= 1;
            if self.nmi_delay == 0 && self.nmi_output & self.in_vblank {
                self.trigger_nmi = true; // this will be handled the next time the CPU steps
            }
        }

        let rendering_enabled = self.show_background || self.show_sprites;
        let mut pixel: Option<(usize, usize, Color)> = None;

        if rendering_enabled {
            // visible scanlines
            if self.scanline < 240 || self.scanline == 261 {
                match self.cycle {
                    0 => (),
                    1..=256 => {
                        if self.scanline != 261 {
                            pixel = Some(self.render_pixel());
                        }
                        match self.cycle % 8 {
                            0 => self.increment_coarse_x(),
                            1 => {
                                self.load_shift_registers();
                                self.fetch_nametable_byte();
                            },
                            3 => self.fetch_attribute_table_byte(),
                            5 => self.fetch_low_tile_byte(),
                            7 => self.fetch_high_tile_byte(),
                            _ => (),
                        }

                        self.update_shift_registers();
                    },
                    257 => {
                        // copy x
                        // horizontal(v) = horizontal(t)
                        self.v = (self.v & 0xfbe0) | (self.t & 0x041f);
                    },
                    321..=336 => {
                        match self.cycle % 8 {
                            0 => self.increment_coarse_x(),
                            1 => {
                                self.load_shift_registers();
                                self.fetch_nametable_byte();
                            },
                            3 => self.fetch_attribute_table_byte(),
                            5 => self.fetch_low_tile_byte(),
                            7 => self.fetch_high_tile_byte(),
                            _ => (),
                        }

                        self.update_shift_registers();
                    },
                    cycle if cycle > 340 => panic!("found cycle > 340"),
                    _ => ()
                }
            }

            // sprite rendering
            if self.scanline < 240 {
                match self.cycle {
                    1 => self.secondary_oam = [0xff; 0x20],
                    257 => {
                        self.evaluate_sprites();
                        self.fetch_sprites();
                    }
                    _ => ()
                }
            }


            if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 {
                // vertical(v) = vertical(t)
                self.v = (self.v & 0x841f) | (self.t & 0x7be0);
            }

            if (self.scanline < 240 || self.scanline == 261) && self.cycle == 256 {
                self.increment_y();
            }
        }

        // vblank logic
        if self.scanline == 241 && self.cycle == 1 {
            self.in_vblank = true;
            self.nmi_change();
        }

        if self.scanline == 261 && self.cycle == 1 {
            self.in_vblank = false;
            self.nmi_change();

            self.sprite_zero_hit = false;
            self.sprite_overflow = false;
        }

        // update end of frame signal
        self.end_of_frame = self.cycle == 256 && self.scanline == 240;

        // advance cycle, scanline, and frame counters
        self.clock();

        pixel
    }

    // PPU's bus read
    fn read(&mut self, address: usize) -> u8 {
        match address {
            0x0000..=0x1fff => self.mapper.borrow().read(address),
            0x2000..=0x3eff => {
                let address = address & 0x0fff;
                match self.mapper.borrow().get_mirror() {
                    Mirror::Horizontal => {
                        // this could be cleaner, but this is more explicit
                        let offset = address & 0x03ff;
                        match address {
                            0x0000..=0x03ff => self.nametable_data[0][offset],
                            0x0400..=0x07ff => self.nametable_data[0][offset],
                            0x0800..=0x0bff => self.nametable_data[1][offset],
                            0x0c00..=0x0fff => self.nametable_data[1][offset],
                            _ => panic!("Bad nametable read at address 0x{:x}", address)
                        }
                    },
                    Mirror::Vertical => {
                        let offset = address & 0x03ff;
                        match address {
                            0x0000..=0x03ff => self.nametable_data[0][offset],
                            0x0400..=0x07ff => self.nametable_data[1][offset],
                            0x0800..=0x0bff => self.nametable_data[0][offset],
                            0x0c00..=0x0fff => self.nametable_data[1][offset],
                            _ => panic!("Bad nametable read at address 0x{:x}", address)
                        }
                    },
                    Mirror::Single0 => self.nametable_data[0][address & 0x03ff],
                    Mirror::Single1 => self.nametable_data[1][address & 0x03ff],
                    _ => {
                        // TODO - implement other mirror reads
                        0
                    }
                }
            },
            0x3f00..=0x3fff => self.palette_data[address & 0x001f],
            _ => 0
        }
    }

    // PPU's bus write
    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x0000..=0x1fff => self.mapper.borrow_mut().write(address, value),
            0x2000..=0x3eff => {
                let address = address & 0x0fff;
                match self.mapper.borrow().get_mirror() {
                    Mirror::Horizontal => {
                        // this could be cleaner, but this is more explicit
                        let offset = address & 0x03ff;
                        match address {
                            0x0000..=0x03ff => self.nametable_data[0][offset] = value,
                            0x0400..=0x07ff => self.nametable_data[0][offset] = value,
                            0x0800..=0x0bff => self.nametable_data[1][offset] = value,
                            0x0c00..=0x0fff => self.nametable_data[1][offset] = value,
                            _ => panic!("Bad nametable write at 0x{:x}", address)
                        }
                    },
                    Mirror::Vertical => {
                        let offset = address & 0x03ff;
                        match address {
                            0x0000..=0x03ff => self.nametable_data[0][offset] = value,
                            0x0400..=0x07ff => self.nametable_data[1][offset] = value,
                            0x0800..=0x0bff => self.nametable_data[0][offset] = value,
                            0x0c00..=0x0fff => self.nametable_data[1][offset] = value,
                            _ => panic!("Bad nametable write at 0x{:x}", address)
                        }
                    },
                    Mirror::Single0 => self.nametable_data[0][address & 0x03ff] = value,
                    Mirror::Single1 => self.nametable_data[1][address & 0x03ff] = value,
                    _ => {
                        // TODO - implement other mirror writes
                    }
                }
            },
            0x3f00..=0x3fff => {
                // "Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C"
                // writing to both addresses will create cleaner code for the PPU read operation
                // https://wiki.nesdev.com/w/index.php/PPU_palettes#Memory_Map

                let address = address & 0x001f;

                if address == 0x10 {
                    self.palette_data[0x0] = value;
                } else if address == 0x14 {
                    self.palette_data[0x04] = value;
                } else if address == 0x18 {
                    self.palette_data[0x08] = value;
                } else if address == 0x1c {
                    self.palette_data[0x0c] = value;
                }
                self.palette_data[address] = value;
            },
            _ => ()
        }
    }

    pub fn nmi_change(&mut self) {
        let nmi = self.nmi_output && self.in_vblank;
        if nmi && !self.nmi_previous {
            self.nmi_delay = 1;
        }
        self.nmi_previous = nmi;
    }
}
