use crate::ppu::PPU;

// https://wiki.nesdev.com/w/index.php/PPU_registers
//
// implementation of the PPU registers
// these are memory mapped to the CPU through the NES's address space
// through $2000-$3FFF (and $4014)
//
// all these methods are public so they can be called via the CPU's "bus" (r/w methods)
impl PPU {
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

        if (self.show_background || self.show_sprites) && (self.scanline < 240 || self.scanline == 261) {
            self.increment_coarse_x();
            self.increment_y();
        } else {
            // increment address based on horizontal or vertical mirror
            self.v += if self.increment { 32 } else { 1 };
        }


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

        if (self.show_background || self.show_sprites) && (self.scanline < 240 || self.scanline == 261) {
            self.increment_coarse_x();
            self.increment_y();
        } else {
            // increment address based on horizontal or vertical mirror
            self.v += if self.increment { 32 } else { 1 };
        }
    }

    // $4014 OAMDMA write
    pub fn write_oam_dma(&mut self, data: [u8; 256]) {
        self.oam_data = data;
    }
}
