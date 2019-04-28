/*
NR30 FF1A E--- ---- DAC power
NR31 FF1B LLLL LLLL Length load (256-L)
NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
NR33 FF1D FFFF FFFF Frequency LSB
NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
*/

pub struct WaveSettings {
    pub sound_length: u16,
    pub length_enabled: bool,
    pub frequency: u16,
    pub volume: f32,
}

impl WaveSettings {
    pub fn new() -> Self {
        Self {
            volume: 0.0,
            sound_length: 0,
            length_enabled: false,
            frequency: 0,
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1B => self.sound_length = 256 - u16::from(value),
            0xFF1C => {
                self.volume = match (value >> 5) & 0x03 {
                    0 => 0.0,
                    1 => 1.0,
                    2 => 0.5,
                    _ => 0.25,
                }
            }
            0xFF1D => self.frequency = (self.frequency & 0x0700) | u16::from(value),
            0xFF1E => {
                self.length_enabled = value & 0x40 > 0;
                self.frequency = u16::from(value & 0x07) << 8 | (self.frequency & 0x00FF);
            }
            _ => unreachable!("Unreachable wave channel sound write operation: 0x{:X}", addr),
        }
    }
}
