pub struct Controller {
    buttons: u8, // each index represents an input
    index: u8, // index that is being looked at
    strobe: u8 // button write
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
        // we read the bit at the index and return its value
        let value = if self.index < 8 && (self.buttons & (1 << self.index) != 0) { 1 } else { 0 };
        self.index += 1;

        // when the first bit in the strobe is high, reading will return the current state of the A button
        if self.strobe & 1 != 0 {
            self.index = 0;
        }

        value
    }

    pub fn write(&mut self, value: u8) {
        // write the value to the strobe
        self.strobe = value;

        if self.strobe & 1 == 1 {
            self.index = 0;
        }
    }
}
