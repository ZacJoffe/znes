pub struct Controller {
    buttons: u8,
    index: u8,
    strobe: u8
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            buttons: 0,
            index: 0,
            strobe: 0
        }
    }

    pub fn read(&mut self) -> u8 {
        let value = if self.index < 8 && (self.buttons & (1 << self.index) != 0) { 0 } else { 1 };
        self.index += 1;

        if self.strobe & 1 != 0 {
            self.index = 0;
        }

        value
    }

    pub fn write(&mut self, value: u8) {
        self.strobe = value;

        if self.strobe & 1 == 1 {
            self.index = 0;
        }
    }
}
