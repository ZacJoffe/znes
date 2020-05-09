use crate::cartridge::Cartridge;
use crate::cartridge::Mapper;
use crate::cartridge::Mirror;

pub struct CNROM {
    cart: Cartridge,
    bank_select: u8
}

impl CNROM {
    pub fn new(cart: Cartridge) -> CNROM {
        CNROM {
            cart: cart,
            bank_select = 0;
        }
    }
}

impl Mapper for CNROM {
    fn read(&self, address: usize) -> u8 {
        match address {
            0x0000..=0x1fff => {
            },
            0x8000..=0xbfff => {
            },
            0xc000..=0xffff => {
            },
            _ => panic!("Address out of range! 0x{:X}", address)
        }
    }

    fn write(&mut self, address: usize, value: u8) {
        match address {
            0x0000..=0x1fff => {
            },
            0x8000..=0xffff => {},
            _ => panic!("Address out of range!")
        }
    }

    fn get_mirror(&self) -> Mirror {
        self.cart.header.mirror
    }

    fn step(&mut self) {}
}
