use crate::ppu::PPU;
use crate::ppu::Color;

impl PPU {
    pub fn evaluate_sprites(&mut self) {
        let sprite_size = if self.flag_sprite_size { 16 } else { 8 };
        let mut sprite_count = 0;

        for i in 0..64 {
            let y = self.oam_data[i * 4] as usize;

            if self.scanline >= y && self.scanline - y < sprite_size {
                for j in 0..4 {
                    self.secondary_oam[sprite_count * 4 + j] = self.oam_data[i * 4 + j];
                }
                self.sprite_indexes[sprite_count] = i as u8;
                sprite_count += 1;
            }

            if sprite_count == 8 {
                // self.sprite_overflow = true;
                break;
            }
        }

        self.sprite_count = sprite_count;
    }

    pub fn fetch_sprites(&mut self) {
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
            let sprite_size: usize = if self.flag_sprite_size { 16 } else { 8 };
            if sprite_size == 8 {
                address += if self.flag_sprite_table { 0x1000 } else { 0 };
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
                    low_bits = (low_bits >> j) << (7 - j);
                    high_bits = (high_bits >> j) << (7 - j);
                }

                shift_registers.0 |= low_bits;
                shift_registers.1 |= high_bits;
            }

            self.sprite_pattern_shift_regs[i] = shift_registers;
            self.sprite_attribute_latches[i] = sprite_attributes;
            self.sprite_positions[i] = x;
        }
    }

    pub fn render_pixel(&mut self) -> (usize, usize, Color) {
        let x = (self.cycle - 1) as usize; // TODO - check this value
        let y = self.scanline;

        let mut background_pixel: u8 = if self.show_background {
            // combine values from the shift register to get background pixel values
            let shift = 15 - self.x;
            let bit_lo = ((self.pattern_shift_reg_low & (1 << shift)) >> shift) as u8;
            let bit_hi = ((self.pattern_shift_reg_high & (1 << shift)) >> shift) as u8;
            (bit_hi << 1) | bit_lo
        } else {
            0
        };

        let mut current_sprite = 0;
        let mut sprite_pixel = if self.show_sprites {
            let mut bit_lo = 0;
            let mut bit_hi = 0;

            for i in 0..self.sprite_count {
                if self.sprite_positions[i] == 0 {
                    current_sprite = i;
                    bit_lo = (self.sprite_pattern_shift_regs[i].0 & (1 << 7)) >> 7;
                    bit_hi = (self.sprite_pattern_shift_regs[i].1 & (1 << 7)) >> 7;
                    if bit_lo != 0 || bit_hi != 0 {
                        break;
                    }
                }
            }

            for i in 0..self.sprite_count {
                if self.sprite_positions[i] == 0 {
                    self.sprite_pattern_shift_regs[i].0 <<= 1;
                    self.sprite_pattern_shift_regs[i].1 <<= 1;
                }
            }

            for i in 0..self.sprite_count {
                if self.sprite_positions[i] > 0 {
                    self.sprite_positions[i] -= 1;
                }
            }

            (bit_hi << 1) | bit_lo
        } else {
            0
        };

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

        let mut palette_address = 0;

        if background_pixel == 0 && sprite_pixel != 0 {
            palette_address += 0x10;
            palette_address += (self.sprite_attribute_latches[current_sprite] & 3) << 2;
            palette_address += sprite_pixel;
        } else if background_pixel != 0 && sprite_pixel == 0 {
            palette_address += palette_offset << 2;
            palette_address += background_pixel;
        } else if background_pixel != 0 && sprite_pixel != 0 {
            if self.sprite_indexes[current_sprite] == 0 {
                self.sprite_zero_hit = true;
            }

            if self.sprite_attribute_latches[current_sprite] & (1 << 5) == 0 {
                palette_address += 0x10;
                palette_address += (self.sprite_attribute_latches[current_sprite] & 3) << 2;
                palette_address += sprite_pixel;
            } else {
                palette_address += palette_offset << 2;
                palette_address += background_pixel;
            }
        }

        // println!("Palette: 0x{:X}  BG: {:b}  Sprite: {:b}", palette_address, background_pixel, sprite_pixel);

        let pixel = self.palette_data[palette_address as usize];

        (x, y, self.palette_table[pixel as usize])
    }

    pub fn increment_coarse_x(&mut self) {
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

    pub fn increment_y(&mut self) {
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

    pub fn fetch_nametable_byte(&mut self) {
        // we can get the nametable byte by concatenating the the course y and course x bits.
        // these are conveniently stored in bits 9-5 and 4-0 respectively in the v register,
        // so we can simply mask off the unneeded bits and or it with the offset to get the address
        let address = 0x2000 | (self.v & 0x0fff) as usize;
        self.nametable_byte = self.read(address);
    }

    pub fn fetch_attribute_table_byte(&mut self) {
        let address = (0x23C0 | (self.v & 0x0C00) | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07)) as usize;
        let byte = self.read(address as usize);

        let coarse_x =  self.v & 0x1f;
        let coarse_y = (self.v & 0x3e0) >> 5;
        let left_or_right = (coarse_x / 2) % 2; // 0 => left, 1 => right
        let top_or_bottom = (coarse_y / 2) % 2; // 0 => top, 1 => bottom

        self.attribute_table_byte = match (top_or_bottom, left_or_right) {
            (0,0) => (byte >> 0) & 0b11,
            (0,1) => (byte >> 2) & 0b11,
            (1,0) => (byte >> 4) & 0b11,
            (1,1) => (byte >> 6) & 0b11,
            _ => panic!("should not get here"),
        };
    }

    pub fn fetch_low_tile_byte(&mut self) {
        let table_base = if self.flag_background_table { 0x1000 } else { 0 };
        let fine_y = (self.v >> 12) & 7;
        let tile = ((self.nametable_byte as u16) << 4) as u16;

        let address = (table_base + tile + fine_y) as usize;

        self.low_tile_byte = self.read(address);
    }

    pub fn fetch_high_tile_byte(&mut self) {
        let table_base = if self.flag_background_table { 0x1000 } else { 0 };
        let fine_y = (self.v >> 12) & 7;
        let tile = ((self.nametable_byte as u16) << 4) as u16;

        let address = (table_base + tile + fine_y) as usize;

        self.high_tile_byte = self.read(address + 8);
    }

    pub fn update_shift_registers(&mut self) {
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

    pub fn load_shift_registers(&mut self) {
        if self.cycle % 8 == 1 {
            self.pattern_shift_reg_low |= self.low_tile_byte as u16;
            self.pattern_shift_reg_high |= self.high_tile_byte as u16;
            self.palette_latch = self.attribute_table_byte;
        }

        // println!("SR_LO {:X} SR_HI {:X} LATCH {:X}", self.pattern_shift_reg_low, self.pattern_shift_reg_high, self.palette_latch);
    }
}
