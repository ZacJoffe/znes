use crate::cartridge::Cartridge;
use crate::cartridge::Mapper;
use crate::cartridge::Mirror;

struct MMC1 {
    cart: Cartridge,
    shift_register: u8,
    control: u8,

    prg_ram_bank: [u8; 0x2000],
    prg_ram_enabled: bool,
    prg_mode: u8,
    prg_bank_select: u8,

    chr_ram_bank: [u8; 0x2000],
    chr_low_bank: u8
    chr_high_bank: u8
    chr_mode: u8,
}

impl MMC1 {
    pub fn new(cart: Cartridge) -> MMC1 {
        MMC1 {
            cart: cart,
            shift_register: 0,
            control: 0,

            prg_ram_bank: [0; 0x2000],
            prg_ram_enabled: false,
            prg_mode: 0,
            prg_bank_select: 0,

            chr_ram_bank: [0; 0x2000],
            chr_low_bank: 0
            chr_high_bank: 0
            chr_mode: 0
        }
    }

    fn write_control_register(&mut self, value: u8) {
        // TODO
    }
}

impl Mapper for MMC1 {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x1fff => self.cart.chr[self.bank_select as usize][address],
            0x8000..=0xbfff => self.cart.prg[0][address % 0x4000],
            0xc000..=0xffff => self.cart.prg[self.cart.header.prg_rom_size - 1][address % 0x4000],
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

    fn step(&mut self) {}
}
