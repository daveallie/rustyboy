pub struct Input {
    io_register: u8,
}

impl Input {
    pub fn new() -> Self {
        Self {
            io_register: 0,
        }
    }

    pub fn read(&self) -> u8 {
        self.io_register
    }

    pub fn write(&mut self, value: u8) {
        self.io_register = value;
    }
}
