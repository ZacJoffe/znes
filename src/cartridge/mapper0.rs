use crate::cartridge::Cartridge;

struct Nrom {
    cart: Cartridge,
    chr_ram: [u8; 0x2000]
}

impl Nrom {
    pub fn new(cart: Cartridge) -> Nrom {
        Nrom {
            cart: cart,
            chr_ram: [0; 0x2000]
        }
    }
}

impl Mapper for Nrom {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x1fff => {
                if self.cart.header.chr_rom_size > 0 {
                    self.cart.chr[address]
                } else {
                    self.chr_ram[address]
                }
            },
            0x8000..=0xbfff => {
                self.cart.prg[address % 0x4000]
            },
            0xc000..=0xffff => {
                self.cart.prg[self.cart.header.prg_rom_size - 1][address % 0x4000]
            },
            _ => panic!("Address out of range!")
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x0000..=0x1fff => {
                if self.cart.header.chr_rom_size == 0 {
                    self.chr_ram[address] = value;
                }
            },
            0x8000.=0xffff => {},
            _ => panic!("Address out of range!")
        }
    }

    fn step(&mut self) {}
}
