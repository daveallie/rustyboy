use sound::settings::envelope_settings::EnvelopeSettings;

/*
NR41 FF20 --LL LLLL Length load (64-L)
NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
NR44 FF23 TL-- ---- Trigger, Length enable
*/

pub struct NoiseSettings {
    pub sound_length: u8,
    pub length_enabled: bool,
    pub frequency: u16,
    pub envelope: EnvelopeSettings,
    pub lfsr_7_bit_mode: bool,
}

impl NoiseSettings {
    pub fn new() -> Self {
        Self {
            sound_length: 0,
            length_enabled: false,
            frequency: 2048,
            envelope: EnvelopeSettings::new(),
            lfsr_7_bit_mode: false,
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF20 => self.sound_length = 64 - (value & 0x3F),
            0xFF21 => self.envelope.write_byte(value),
            0xFF22 => {
                self.lfsr_7_bit_mode = value & 0x08 > 0;
                let clock_shift = u16::from(value) >> 4;
                let divisor = match u16::from(value & 0x07) {
                    0 => 8_u16,
                    divisor_code => 16 * (divisor_code + 1),
                };
                self.frequency = divisor << clock_shift;
            }
            0xFF23 => self.length_enabled = value & 0x40 > 0,
            _ => unreachable!("Unreachable square channel sound write operation: 0x{:X}", addr),
        }
    }
}
