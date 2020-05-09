use crate::cartridge::Cartridge;
use crate::cartridge::Mapper;
use crate::cartridge::Mirror;

pub struct CNROM {
    cart: Cartridge,
    // https://wiki.nesdev.com/w/index.php/CNROM#Bank_select_.28.248000-.24FFFF.29
    //
    // "Select 8 KB CHR ROM bank for PPU $0000-$1FFF"
    // only the lower 2 bits of the value is used for this
    bank_select: u8
}

impl CNROM {
    pub fn new(cart: Cartridge) -> CNROM {
        CNROM {
            cart: cart,
            bank_select: 0,
        }
    }
}

impl Mapper for CNROM {
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
            0x8000..=0xffff => {
                // write the first 2 bits into the bank select
                self.bank_select = value & 3;
            },
            _ => panic!("Address out of range!")
        }
    }

    fn get_mirror(&self) -> Mirror {
        self.cart.header.mirror
    }

    fn step(&mut self) {}
}
