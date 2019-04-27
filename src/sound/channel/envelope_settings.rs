// VVVV APPP - Starting volume, Envelope add mode, period
pub struct EnvelopeSettings {
    pub period: u8,
    pub starting_volume: u8,
    volume_increasing: bool,
}

impl EnvelopeSettings {
    pub fn new() -> Self { Self { period: 1, starting_volume: 0, volume_increasing: false } }

    pub fn write_byte(&mut self, byte: u8) {
        self.starting_volume = byte >> 4;
        self.volume_increasing = byte & 0x08 > 0;
        self.period = byte & 0x07;
    }

    pub fn tick(&mut self, current_volume: u8) -> u8 {
        if self.volume_increasing && current_volume < 15 {
            current_volume + 1
        } else if !self.volume_increasing && current_volume > 0 {
            current_volume - 1
        } else {
            current_volume
        }
    }
}
