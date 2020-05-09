use crate::cartridge::Cartridge;

struct MMC1 {
    cart: Cartridge,
    shift_register: u8,
    control: u8,

    prg_mode: u8,
    prg_bank: u8,

    chr_mode: u8,
    chr_low_bank: u8
    chr_high_bank: u8
}

impl MMC1 {
    pub fn new(cart: Cartridge) -> MMC1 {
        MMC1 {
            cart: cart,
            shift_register: 0,
            control: 0,
        }
    }
}
