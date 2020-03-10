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
