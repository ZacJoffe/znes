use crate::cartridge::Cartridge;
use crate::cartridge::Mapper;
use crate::cartridge::Mirror;

pub struct UxROM {
    cart: Cartridge,
    chr_ram: [u8; 0x2000],
    bank_select: u8
}

impl UxROM {
    pub fn new(cart: Cartridge) -> UxROM {
        cart: cart,
        chr_ram: [0; 0x2000],
        bank_select: 0
    }
}

impl Mapper for UxROM {
   fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x1fff => {
                if self.cart.header.chr_rom_size > 0 {
                    self.cart.chr[0][address]
                } else {
                    self.chr_ram[address]
                }
            },
            0x8000..=0xbfff => self.cart.prg[self.bank_select][address % 0x4000],
            0xc000..=0xffff => self.cart.prg[self.cart.header.prg_rom_size - 1][address % 0x4000],
            _ => panic!("Address out of range! 0x{:X}", address)
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x0000..=0x1fff => {
                if self.cart.header.chr_rom_size == 0 {
                    self.chr_ram[address] = value;
                }
            }
            0x8000..=0xffff => self.bank_select = value,
            _ => panic!("Address out of range!")
        }
    }

    fn get_mirror(&self) -> Mirror {
        self.cart.header.mirror
    }

    fn step(&mut self) {}
}
