use crate::cartridge::Cartridge;
use crate::cartridge::Mapper;
use crate::cartridge::Mirror;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub struct MMC1 {
    cart: Cartridge,
    step: u8,
    shift_register: u8,
    control: u8,

    prg_ram_bank: [u8; 0x2000],
    prg_ram_enabled: bool,
    prg_mode: u8,
    prg_bank_select: u8,

    chr_ram_bank: [u8; 0x2000],
    chr_low_bank: u8,
    chr_high_bank: u8,
    chr_mode: bool,
}

impl MMC1 {
    pub fn new(cart: Cartridge) -> MMC1 {
        let mut mmc1 = MMC1 {
            cart: cart,
            step: 0,
            shift_register: 0,
            control: 0,

            prg_ram_bank: [0; 0x2000],
            prg_ram_enabled: false,
            prg_mode: 0,
            prg_bank_select: 0,

            chr_ram_bank: [0; 0x2000],
            chr_low_bank: 0,
            chr_high_bank: 0,
            chr_mode: false
        };
        mmc1.load_battery();

        mmc1
    }

    fn write_control_register(&mut self, value: u8) {
        self.control = value;

        self.cart.header.mirror = match value & 3 {
            0 => Mirror::Single0,
            1 => Mirror::Single1,
            2 => Mirror::Vertical,
            3 => Mirror::Horizontal,
            _ => panic!("Bad mirror value!")
        };

        self.prg_mode = (value >> 2) & 3;
        self.chr_mode = value & 0x10 != 0;
    }
}

impl Mapper for MMC1 {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x1fff => {
                if self.cart.header.chr_rom_size == 0 {
                    self.chr_ram_bank[address]
                } else {
                    if self.chr_mode {
                        let bank = match address {
                            0x0000..=0x0fff => self.chr_low_bank,
                            0x1000..=0x1fff => self.chr_high_bank,
                            _ => panic!("Address out of range! 0x{:X}", address)
                        };

                        let chunk_half = if bank % 2 == 0 { 0 } else { 0x1000 };

                        self.cart.chr[(bank / 2) as usize][chunk_half + (address % 0x1000)]
                    } else {
                        self.cart.chr[self.chr_low_bank as usize][address]
                    }
                }
            }
            0x6000..=0x7fff => self.prg_ram_bank[address & 0x2000],
            0x8000..=0xbfff => {
                match self.prg_mode {
                    0 | 1 => self.cart.prg[(self.prg_bank_select & 0xfe) as usize][address % 0x4000],
                    2 => self.cart.prg[0][address % 0x4000],
                    3 => self.cart.prg[self.prg_bank_select as usize][address % 0x4000],
                    _ => panic!("Bad prg mode!")
                }
            },
            0xc000..=0xffff => {
                match self.prg_mode {
                    0 | 1 => self.cart.prg[((self.prg_bank_select & 0xfe) + 1) as usize][address % 0x4000],
                    2 => self.cart.prg[self.prg_bank_select as usize][address % 0x4000],
                    3 => self.cart.prg[self.cart.header.prg_rom_size - 1][address % 0x4000],
                    _ => panic!("Bad prg mode!")
                }
            },
            _ => panic!("Address out of range! 0x{:X}", address)
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x0000..=0x1fff => {
                if self.cart.header.chr_rom_size == 0 {
                    self.chr_ram_bank[address] = value;
                }
            },
            0x6000..=0x7fff => self.prg_ram_bank[address % 0x2000] = value,
            0x8000..=0xffff => {
                // write serial port
                if value & 0x80 == 1 {
                    self.shift_register = 0;
                    self.step = 0;
                    self.write_control_register(self.control | 0x0c);
                } else {
                    self.shift_register >>= 1;
                    self.shift_register |= (value << 7) & 0x80;

                    if self.step == 4 {
                        self.shift_register >>= 3;

                        match address {
                            0x8000..=0x9fff => self.write_control_register(self.shift_register),
                            0xa000..=0xbfff => {
                                // write chr low bank
                                if self.chr_mode {
                                    self.chr_low_bank = self.shift_register;
                                } else {
                                    let v = self.shift_register & 0xfe;
                                    self.chr_low_bank = v;
                                    self.chr_high_bank = v + 1;
                                }
                            },
                            0xc000..=0xdfff => {
                                // write chr high bank
                                if self.chr_mode {
                                    self.chr_high_bank = self.shift_register;
                                }
                            },
                            0xe000..=0xffff => {
                                // write prg bank
                                self.prg_bank_select = self.shift_register & 0x0f;
                                self.prg_ram_enabled = self.shift_register & 0x10 != 0;
                            },
                            _ => ()
                        }

                        self.step = 0;
                        self.shift_register = 0;
                    } else {
                        self.step += 1;
                    }
                }
            },
            _ => panic!("Address out of range!")
        }
    }

    fn get_mirror(&self) -> Mirror {
        self.cart.header.mirror
    }

    fn load_battery(&mut self) {
        if self.cart.header.battery_backed_ram {
            let path = Path::new(&self.cart.header.file_path).parent().unwrap();
            let mut save = path.join(Path::new(&self.cart.header.file_path).parent().unwrap());
            save.set_extension("sav");

            if Path::new(&save).exists() {
                let mut file = File::open(save.clone()).expect("Failed to open .sav file");
                let mut battery_ram_buffer = vec![];

                file.read_to_end(&mut battery_ram_buffer).expect("Failed to read .sav file");
                println!("loading battery-backed RAM from file: {:?}", save);

                // copy vector data into array
                self.prg_ram_bank.copy_from_slice(&battery_ram_buffer[..]);
            }
        }
    }

    fn save_battery(&self) {
        if self.cart.header.battery_backed_ram {
            let path = Path::new(&self.cart.header.file_path).parent().unwrap();
            let mut save = path.join(Path::new(&self.cart.header.file_path).parent().unwrap());
            save.set_extension("sav");

            let mut file = File::create(&save).expect("Failed to create .sav file");
            file.write_all(&self.prg_ram_bank).expect("Failed to write to .sav file");
        }
    }

    fn step(&mut self) {}
}
