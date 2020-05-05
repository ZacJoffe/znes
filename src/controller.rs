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

    pub fn read() -> u8 {
        0
    }

    pub fn write(value: u8) {
    }
}
