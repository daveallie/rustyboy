pub struct Noise {
    sound_length: u8,
    envelope: u8,
    polynomial_counter: u8,
    counter_consecutive: u8,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            sound_length: 0,
            envelope: 0,
            polynomial_counter: 0,
            counter_consecutive: 0,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF20 => self.sound_length,
            0xFF21 => self.envelope,
            0xFF22 => self.polynomial_counter,
            0xFF23 => self.counter_consecutive & 0x40,
            _ => {
                unreachable!(
                    "Unreachable noise channel sound read operation: 0x{:X}",
                    addr
                )
            }
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF20 => self.sound_length = value & 0x3F,
            0xFF21 => self.envelope = value,
            0xFF22 => self.polynomial_counter = value,
            0xFF23 => self.counter_consecutive = value & 0xC0,
            _ => {
                unreachable!(
                    "Unreachable noise channel sound write operation: 0x{:X}",
                    addr
                )
            }
        }
    }
}
