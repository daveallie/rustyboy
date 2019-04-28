use sound::settings::envelope_settings::EnvelopeSettings;
use sound::settings::sweep_settings::SweepSettings;

pub struct SquareSettings {
    pub duty: u8,
    pub sound_length: u8,
    pub length_enabled: bool,
    pub frequency: u16,
    pub envelope: EnvelopeSettings,
    pub sweep: SweepSettings,
}

impl SquareSettings {
    pub fn new() -> Self {
        Self {
            duty: 0,
            sound_length: 0,
            length_enabled: false,
            frequency: 0,
            envelope: EnvelopeSettings::new(),
            sweep: SweepSettings::new(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 => self.sweep.write_byte(value),
            0xFF11 | 0xFF16 => {
                self.duty = value >> 6;
                self.sound_length = 64 - (value & 0x3F);
            }
            0xFF12 | 0xFF17 => self.envelope.write_byte(value),
            0xFF13 | 0xFF18 => self.frequency = (self.frequency & 0x0700) | u16::from(value),
            0xFF14 | 0xFF19 => {
                self.length_enabled = value & 0x40 > 0;
                self.frequency = u16::from(value & 0x07) << 8 | (self.frequency & 0x00FF);
            }
            _ => unreachable!("Unreachable square channel sound write operation: 0x{:X}", addr),
        }
    }
}
