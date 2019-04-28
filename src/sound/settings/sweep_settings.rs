pub struct SweepSettings {
    pub period: u8,
    pub frequency_increasing: bool,
    pub shift: u8,
}

impl SweepSettings {
    pub fn new() -> Self {
        Self {
            period: 0,
            frequency_increasing: false,
            shift: 0,
        }
    }

    pub fn write_byte(&mut self, value: u8) {
        self.period = (value >> 4) & 0x07;
        self.frequency_increasing = value & 0x08 == 0;
        self.shift = value & 0x07;
    }

    pub fn tick(&self, current_frequency: u16) -> u16 {
        let freq_mod = current_frequency >> self.shift;
        if self.frequency_increasing {
            if current_frequency + freq_mod > 2048 {
                2048
            } else {
                current_frequency + freq_mod
            }
        } else if current_frequency < freq_mod {
            0
        } else {
            current_frequency - freq_mod
        }
    }
}
