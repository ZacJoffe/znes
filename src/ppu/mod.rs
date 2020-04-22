use crate::cartridge::Mapper;
use crate::cartridge::Mirror;

use std::rc::Rc;
use std::cell::RefCell;



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

    nametable_data: [[u8; 0x400]; 2],
    palette_data: [u8; 0x20],
    oam_data: [u8; 0x100],

    mapper: Rc<RefCell<dyn Mapper>>,

    nametable_byte: u8,
    attribute_table_byte: u8,
    low_tile_byte: u8,
    high_tile_byte: u8,
    tile_data: u64,

    oam_address: u8,

    data_buffer: u8, // "Least significant bits previously written into a PPU register"

    // flags
    // NMI flags
    nmi_previous: bool,
    nmi_output: bool,
    nmi_delay: u8,

    // $2000 PPUCTRL
    increment: bool, // true => add 32, false => add 1

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

            mapper: mapper,

            nametable_data: [[0; 0x400]; 2],
            palette_data: [0; 0x20],
            oam_data: [0; 0x100],

            nametable_byte: 0,
            attribute_table_byte: 0,
            low_tile_byte: 0,
            high_tile_byte: 0,
            tile_data: 0,

            data_buffer: 0,

            oam_address: 0,

            nmi_previous: false,
            nmi_output: false,
            nmi_delay: 0,

            increment: false,

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
    fn read_status(&mut self) -> u8 {
        let mut result: u8 = self.data_buffer & 0x1f;

        if self.sprite_overflow { result |= 1 << 5; }
        if self.sprite_zero_hit { result |= 1 << 6; }
        if self.in_vblank { result |= 1 << 7; }

        self.w = 0;
        self.in_vblank = false;
        self.nmi_change();

        result
    }

    fn read_oam_data(&mut self) -> u8 {
        self.oam_data[self.oam_address as usize]
    }

    fn read_data(&mut self) -> u8 {
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
    fn write_oam_address(&mut self, value: u8) {
        self.oam_address = value;
    }

    fn write_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_address as usize] = value;
        self.oam_address += 1;
    }

    fn write_data(&mut self, value: u8) {
        self.write(self.v as usize, value);

        // increment address based on horizontal or vertical mirror
        self.v += if self.increment { 32 } else { 1 };
    }

    pub fn read_register(&mut self, address: usize) -> u8 {
        match address {
            0x2002 => self.read_status(),
            0x2004 => self.read_oam_data(),
            0x2007 => self.read_data(),
            _ => 0
        }
    }

    pub fn write_register(&mut self, address: usize, value: u8) {
        match address {
            0x2000 => {},
            0x2001 => {},
            0x2003 => self.write_oam_address(value),
            0x2004 => self.write_oam_data(value),
            0x2005 => {},
            0x2006 => {},
            0x2007 => self.write_data(value),
            0x4014 => {},
            _ => panic!("Invalid PPU register write! 0x{:x}", address)
        }
    }
}
