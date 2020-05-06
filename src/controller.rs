const A_INDEX: usize = 0;
const B_INDEX: usize = 1;
const SELECT_INDEX: usize = 2;
const UP_INDEX: usize = 4;
const DOWN_INDEX: usize = 5;
const LEFT_INDEX: usize = 6;
const RIGHT_INDEX: usize = 7;


#[derive(Copy, Clone)]
pub struct Controller {
    // a standard NES controller has a total of 8 inputs, with each
    // being conveniently mapped to a bit in byte sized register
    //
    // 0 - A
    // 1 - B
    // 2 - Select
    // 3 - Start
    // 4 - Up
    // 5 - Down
    // 6 - Left
    // 7 - Right
    buttons: u8, // each index represents an input
    index: u8, // index that is being looked at
    pub strobe: u8 // determine when to read/write buttons
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
