pub struct Serial {
    data: u8,
    control: u8,
}

impl Serial {
    pub fn new() -> Self {
        Self { data: 0, control: 0 }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.read_data(),
            0xFF02 => self.read_control(),
            _ => panic!("Unknown serial read operation: 0x{:X}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF01 => self.write_data(value),
            0xFF02 => self.write_control(value),
            _ => panic!("Unknown serial write operation: 0x{:X}", addr),
        }
    }

    fn read_data(&self) -> u8 {
        self.data
    }

    fn write_data(&mut self, value: u8) {
        self.data = value;
    }

    fn read_control(&self) -> u8 {
        self.control
    }

    fn write_control(&mut self, value: u8) {
        self.control = value;
        // TODO: Implement
    }
}
