use crate::cartridge::Mapper;
use crate::cartridge::Mirror;

use std::rc::Rc;
use std::cell::RefCell;


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

    data_buffer: u8, // "Least significant bits previously written into a PPU register"

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
    flag_sprite_table: bool,
    flag_background_table: bool, // false => 0x0000, true => 0x1000
    flag_sprite_size: bool,
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
    palette_table: [(u8, u8, u8); 0x40]
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
                (84, 84, 84), (0, 30, 116), (8, 16, 144), (48, 0, 136), (68, 0, 100), (92, 0, 48), (84, 4, 0), (60, 24, 0), (32, 42, 0), (8, 58, 0), (0, 64, 0), (0, 60, 0), (0, 50, 60), (0, 0, 0), (0, 0, 0), (0, 0, 0),
                (152, 150, 152), (8, 76, 196), (48, 50, 236), (92, 30, 228), (136, 20, 176), (160, 20, 100), (152, 34, 32), (120, 60, 0), (84, 90, 0), (40, 114, 0), (8, 124, 0), (0, 118, 40), (0, 102, 120), (0, 0, 0), (0, 0, 0), (0, 0, 0),
                (236, 238, 236), (76, 154, 236), (120, 124, 236), (176, 98, 236), (228, 84, 236), (236, 88, 180), (236, 106, 100), (212, 136, 32), (160, 170, 0), (116, 196, 0), (76, 208, 32), (56, 204, 108), (56, 180, 204), (60,  60,  60), (0, 0, 0), (0, 0, 0),
                (236, 238, 236), (168, 204, 236), (188, 188, 236), (212, 178, 236), (236, 174, 236), (236, 174, 212), (236, 180, 176), (228, 196, 144), (204, 210, 120), (180, 222, 120), (168, 226, 144), (152, 226, 180), (160, 214, 228), (160, 162, 160), (0, 0, 0), (0, 0, 0)
            ]
        }
    }

    pub fn clock(&mut self) {
        if self.nmi_delay > 0 {
            self.nmi_delay -= 1;
            if self.nmi_delay == 0 && self.nmi_output & self.in_vblank {
                self.trigger_nmi = true; // this will be handled the next time the CPU steps
            }
        }

        let rendering_enabled = self.show_background || self.show_sprites;

        if rendering_enabled && self.cycle == 339 && self.scanline == 261 && self.frame % 2 == 0 {
            self.cycle = 0;
            self.scanline = 0;
            self.frame = self.frame.wrapping_add(1);
            return
        }

        self.cycle += 1;

        if self.cycle >= 341 {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline >= 261 {
                self.scanline = 0;
                self.frame = self.frame.wrapping_add(1);
            }
        }
    }

    pub fn step(&mut self) {
        // advance cycle, scanline, and frame counters
        self.clock();

        let rendering_enabled = self.show_background || self.show_sprites;
        let mut pixel: Option<(usize, usize, (u8, u8, u8))> = None;

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
                            0 => self.inc_coarse_x(),
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
                            0 => self.inc_coarse_x(),
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

            // TODO - sprite rendering
            if self.scanline < 240 {
                match self.cycle {
                    1 => self.secondary_oam = [0xff; 0x20],
                    257 => {
                        self.evaluate_sprites();
                        self.fetch_spries();
                    }
                    _ => ()
                }
            }


            if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 {
                // vertical(v) = vertical(t)
                self.v = (self.v & 0x841f) | (self.t & 0x7be0);
            }

            if (self.scanline < 240 || self.scanline == 261) && self.cycle == 256 {
                self.inc_y();
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
        }
    }

    fn evaluate_sprites(&mut self) {
        let sprite_size = if self.flag_sprite_size { 16 } else { 8 };
        let mut sprite_count = 0;

        for i in 0..64 {
            let y = self.oam_data[i * 4] as usize;
            let row = self.scanline - y;
            if row >= 0 && row < sprite_size {
                for j in 0..4 {
                    self.secondary_oam[sprite_count * 4 + j] = self.oam_data[i * 4 + j];
                }
                self.sprite_indexes[sprite_count] = i as u8;
                sprite_count += 1;
            }

            if sprite_count > 8 {
                sprite_count = 8;
                self.sprite_overflow = true;
                break;
            }
        }

        self.sprite_count = sprite_count;
    }

    fn fetch_spries(&mut self) {
        for i in 0..self.sprite_count {
            let y = self.secondary_oam[4 * i] as usize;
            let sprite_tile_index = self.secondary_oam[4 * i + 1] as usize;
            let sprite_attributes = self.secondary_oam[4 * i + 2];
            let x = self.secondary_oam[4 * i + 3];

            let flipped_vertically = sprite_attributes & (1 << 7) != 0;
            let flipped_horizontally = sprite_attributes & (1 << 6) != 0;

            // TODO - make scanline usize
            let row = self.scanline as usize - y;

            let mut address: usize = 0;
            let sprite_size: usize = if self.flag_sprite_size { 16 } else { 0 };
            if sprite_size == 8 {
                address += if self.flag_sprite_table { 0 } else { 0x1000 };
                address += sprite_tile_index * 16;

                address += if !flipped_vertically {
                    row
                } else {
                    sprite_size - 1 - row
                };
            } else {
                address += if sprite_tile_index & 1 == 0 { 0x0 } else { 0x1000 };
                address += (sprite_tile_index & 0xfffe) << 4;

                let fine_y = if !flipped_vertically {
                    row
                } else {
                    sprite_size - 1 - row
                };

                address += fine_y;

                if fine_y > 7 {
                    address += 8;
                }
            }

            let low_pattern_byte = self.read(address);
            let high_pattern_byte = self.read(address + 8);

            let mut shift_registers: (u8, u8) = (0, 0);

            // fill out sprite shift register by looping through each bit
            //
            // if flipped horizontally, the bits are mirrored by the nibble
            // e.g. 0b1001_0110 => 0b0110_1001
            for j in 0..8 {
                let mut low_bits = low_pattern_byte & (1 << j);
                let mut high_bits = high_pattern_byte & (1 << j);

                if flipped_horizontally {
                    // mirror the bits by the nibble
                    // e.g. 0b0001_0000 => 0b0000_1000
                    let low_bits = (low_bits >> j) << (7 - j);
                    let high_bits = (high_bits >> j) << (7 - j);
                }

                shift_registers.0 |= low_bits;
                shift_registers.1 |= high_bits;
            }

            self.sprite_pattern_shift_regs[i] = shift_registers;
            self.sprite_attribute_latches[i] = sprite_attributes;
            self.sprite_positions[i] = x;
        }
    }

    fn render_pixel(&mut self) -> (usize, usize, (u8, u8, u8)) {
        let x = (self.cycle - 1) as usize; // TODO - check this value
        let y = self.scanline as usize;

        let mut background_pixel: u8 = if self.show_background {
            // combine values from the shift register to get background pixel values
            let shift = 15 - self.x;
            let bit_lo = ((self.pattern_shift_reg_low & (1 << shift)) >> shift) as u8;
            let bit_hi = ((self.pattern_shift_reg_high & (1 << shift)) >> shift) as u8;
            (bit_hi << 1) | bit_lo
        } else {
            0
        };

        // TODO - implement sprites
        let mut sprite_pixel = 0;
        let current_sprite = 0;

        let shift = 7 - self.x;
        let palette_bit_lo = ((self.palette_shift_reg_low & (1 << shift)) >> shift) as u8;
        let palette_bit_hi = ((self.palette_shift_reg_high & (1 << shift)) >> shift) as u8;
        let palette_offset = (palette_bit_hi << 1) | palette_bit_lo;

        if x < 8 {
            if !self.show_left_background {
                background_pixel = 0;
            }

            if !self.show_left_spries {
                sprite_pixel = 0;
            }
        }

        // TODO - implement sprites
        let mut palette_address = 0;

        let pixel = self.palette_data[palette_address as usize];

        (x, y, self.palette_table[pixel as usize])
    }

    fn inc_coarse_x(&mut self) {
        // https://wiki.nesdev.com/w/index.php/PPU_scrolling#Coarse_X_increment
        let coarse_x = self.v & 0x001f;

        // handle wrap around if 5 bit course x is at its maximum value
        if coarse_x == 0x1f {
            self.v &= 0xffe0;
            self.v ^= 0x0400;
        } else {
            self.v += 1;
        }
    }

    fn inc_y(&mut self) {
        let fine_y = (self.v & 0x7000) >> 12;

        let coarse_y_mask = 0x03e0;
        let mut coarse_y = (self.v & coarse_y_mask) >> 5;

        if fine_y < 7 {
            // increment fine_y
            self.v += 0x1000
        } else {
            // reset fine_y to 0
            self.v &= 0x8fff;

            if coarse_y == 29 {
                // wrap coarse_y back to 0
                coarse_y = 0;

                // toggle vertical nametable
                self.v ^= 0x0800;
            } else if coarse_y == 31 {
                // coarse_y = 0 without changing the nametable
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }
        }

        // store our new coarse_y back into v
        self.v = (self.v & !coarse_y_mask) | (coarse_y << 5);
    }

    fn fetch_nametable_byte(&mut self) {
        // we can get the nametable byte by concatenating the the course y and course x bits.
        // these are conveniently stored in bits 9-5 and 4-0 respectively in the v register,
        // so we can simply mask off the unneeded bits and or it with the offset to get the address
        let address = 0x2000 | (self.v & 0x0fff) as usize;
        self.nametable_byte = self.read(address);
    }

    fn fetch_attribute_table_byte(&mut self) {
        let address = (0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07)) as usize;
        let shift = ((self.v >> 4) & 4) | (self.v & 2);
        self.attribute_table_byte = ((self.read(address) >> shift) & 3) << 2;
    }

    fn fetch_low_tile_byte(&mut self) {
        let fine_y = (self.v >> 12) & 7;
        let table_base = 0x1000 * (self.flag_background_table as u16);
        let tile = (self.nametable_byte << 4) as u16;

        let address = (table_base + tile + fine_y) as usize;
        self.low_tile_byte = self.read(address);
    }

    fn fetch_high_tile_byte(&mut self) {
        let fine_y = (self.v >> 12) & 7;
        let table_base = 0x1000 * (self.flag_background_table as u16);
        let tile = (self.nametable_byte << 4) as u16;

        let address = (table_base + tile + fine_y) as usize;
        self.high_tile_byte = self.read(address + 8);
    }

    fn update_shift_registers(&mut self) {
        self.pattern_shift_reg_low <<= 1;
        self.pattern_shift_reg_high <<= 1;

        self.palette_shift_reg_low <<= 1;
        self.palette_shift_reg_high <<= 1;

        // set bits 0 and 1 of the palette latch to bit 0 of the low/high
        // palette registers respectively
        let latch_bit0 = self.palette_latch & 0b01;
        let latch_bit1 = (self.palette_latch & 0b10) >> 1;
        self.palette_shift_reg_low |= latch_bit0;
        self.palette_shift_reg_high |= latch_bit1;
    }

    fn load_shift_registers(&mut self) {
        self.pattern_shift_reg_low |= self.low_tile_byte as u16;
        self.pattern_shift_reg_high |= self.high_tile_byte as u16;
        self.palette_latch = self.attribute_table_byte;
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
                        match address {
                            0x0000..=0x03ff => self.nametable_data[0][address & 0x03ff],
                            0x0400..=0x07ff => self.nametable_data[0][address & 0x03ff],
                            0x0800..=0x0bff => self.nametable_data[1][address & 0x03ff],
                            0x0c00..=0x0fff => self.nametable_data[1][address & 0x03ff],
                            _ => panic!("Bad nametable read at address 0x{:x}", address)
                        }
                    },
                    Mirror::Vertical => {
                         match address {
                            0x0000..=0x03ff => self.nametable_data[0][address & 0x03ff],
                            0x0400..=0x07ff => self.nametable_data[1][address & 0x03ff],
                            0x0800..=0x0bff => self.nametable_data[0][address & 0x03ff],
                            0x0c00..=0x0fff => self.nametable_data[1][address & 0x03ff],
                            _ => panic!("Bad nametable read at address 0x{:x}", address)
                        }
                    },
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
                        match address {
                            0x0000..=0x03ff => self.nametable_data[0][address & 0x03ff] = value,
                            0x0400..=0x07ff => self.nametable_data[0][address & 0x03ff] = value,
                            0x0800..=0x0bff => self.nametable_data[1][address & 0x03ff] = value,
                            0x0c00..=0x0fff => self.nametable_data[1][address & 0x03ff] = value,
                            _ => panic!("Bad nametable write at 0x{:x}", address)
                        }
                    },
                    Mirror::Vertical => {
                         match address {
                            0x0000..=0x03ff => self.nametable_data[0][address & 0x03ff] = value,
                            0x0400..=0x07ff => self.nametable_data[1][address & 0x03ff] = value,
                            0x0800..=0x0bff => self.nametable_data[0][address & 0x03ff] = value,
                            0x0c00..=0x0fff => self.nametable_data[1][address & 0x03ff] = value,
                            _ => panic!("Bad nametable write at 0x{:x}", address)
                        }
                    },
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



    // CPU READS
    // $2002 PPUSTATUS read
    pub fn read_status(&mut self) -> u8 {
        let mut result: u8 = self.data_buffer & 0x1f;

        if self.sprite_overflow { result |= 1 << 5; }
        if self.sprite_zero_hit { result |= 1 << 6; }
        if self.in_vblank { result |= 1 << 7; }

        self.w = 0;
        self.in_vblank = false;
        self.nmi_change();

        result
    }

    // $2004 OAMDATA read
    pub fn read_oam_data(&mut self) -> u8 {
        self.oam_data[self.oam_address as usize]
    }

    // $2007 PPUDATA read
    pub fn read_data(&mut self) -> u8 {
        let mut result = self.read(self.v as usize);

        if self.v % 0x4000 < 0x3f00 {
            let buffered_data = self.read_buffer_data;
            self.read_buffer_data = result;
            result = buffered_data;
        } else {
            // palette address space
            self.read_buffer_data = self.read(self.v as usize - 0x1000);
        }

        // increment address based on horizontal or vertical mirror
        self.v += if self.increment { 32 } else { 1 };

        result
    }


    // CPU WRITES
    // $2000 PPUCTRL write
    pub fn write_control(&mut self, value: u8) {
        self.flag_nametable = value & 3;
        self.increment = value & (1 << 2) != 0;
        self.flag_sprite_table = value & (1 << 3) != 0;
        self.flag_background_table = value & (1 << 4) != 0;
        self.flag_sprite_size = value & (1 << 5) != 0;
        self.flag_master_slave = value & (1 << 6) != 0;
        self.nmi_output = value & (1 << 7) != 0;
        self.nmi_change();

        self.t = (self.t & 0xf3ff) | ((value as u16 & 3) << 10)
    }

    // $2001 PPUMASK write
    pub fn write_mask(&mut self, value: u8) {
        self.grayscale = value & 1 != 0;
        self.show_left_background = value & (1 << 1) != 0;
        self.show_left_spries = value & (1 << 2) != 0;
        self.show_background = value & (1 << 3) != 0;
        self.show_sprites = value & (1 << 4) != 0;
        self.red_tint = value & (1 << 5) != 0;
        self.blue_tint = value & (1 << 6) != 0;
        self.green_tint = value & (1 << 7) != 0;
    }

    // $2003 OAMADDR write
    pub fn write_oam_address(&mut self, value: u8) {
        self.oam_address = value;
    }

    // $2004 OAMDATA write
    pub fn write_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_address as usize] = value;
        self.oam_address += 1;
    }

    // $2005 PPUSCROLL write
    pub fn write_scroll(&mut self, value: u8) {
        if self.w == 0 {
            self.t = (self.t & 0xffe0) | (value as u16 >> 3);
            self.x = value & 7;
            self.w = 1;
        } else {
            self.t = (self.t & 0x8fff) | ((value as u16 & 7) << 12);
            self.t = (self.t & 0xfc1f) | ((value as u16 & 0xf8) << 2);
            self.w = 0;
        }
    }

    // $2006 PPUADDR write
    pub fn write_address(&mut self, value: u8) {
        if self.w == 0 {
            self.t = (self.t & 0x80ff) | ((value as u16 & 0x3f) << 8);
            self.w = 1;
        } else {
            self.t = (self.t & 0xff00) | (value as u16);
            self.v = self.t;
            self.w = 0;
        }
    }

    // $2007 PPUDATA write
    pub fn write_data(&mut self, value: u8) {
        self.write(self.v as usize, value);

        // increment address based on horizontal or vertical mirror
        self.v += if self.increment { 32 } else { 1 };
    }

    // $4014 OAMDMA write
    pub fn write_oam_dma(&mut self, data: [u8; 256]) {
        self.oam_data = data;
    }
}
