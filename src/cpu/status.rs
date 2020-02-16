use std::convert::From;

#[derive(Copy, Clone)]
pub struct Status {
    pub negative: bool,
    pub overflow: bool,
    pub decimal: bool,
    pub interrupt: bool,
    pub zero: bool,
    pub carry: bool
}

impl Status {
    pub fn new() -> Status {
        Status {
            negative: false,
            overflow: false,
            decimal: false,
            interrupt: false,
            zero: false,
            carry: false
        }
    }

    pub fn set_negative(&mut self, num: u8) {
        self.negative = (num & 0x80) == 0x80;
    }

    pub fn set_zero(&mut self, num: u8) {
        self.zero = num == 0;
    }
}

impl From<u8> for Status {
    fn from(byte: u8) -> Self {
        let negative = ((byte >> 7) & 0x1) != 0;
        let overflow = ((byte >> 6) & 0x1) != 0;
        let decimal = ((byte >> 3) & 0x1) != 0;
        let interrupt = ((byte >> 2) & 0x1) != 0;
        let zero = ((byte >> 1) & 0x1) != 0;
        let carry = (byte & 0x1) != 0;

        Status {
            negative,
            overflow,
            decimal,
            interrupt,
            zero,
            carry
        }
    }
}

impl From<Status> for u8 {
    fn from(status: Status) -> u8 {
        let negative = if status.negative { 1 } else { 0 };
        let overflow = if status.overflow { 1 } else { 0 };
        let decimal = if status.decimal { 1 } else { 0 };
        let interrupt = if status.interrupt { 1 } else { 0 };
        let zero = if status.zero { 1 } else { 0 };
        let carry = if status.carry { 1 } else { 0 };

        (negative << 7) | (overflow << 6) | (decimal << 3) | (interrupt << 2) | (zero << 1) | carry
    }
}
