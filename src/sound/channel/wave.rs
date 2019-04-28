use sound::settings::wave_settings::WaveSettings;
use sound::Sound;

/*
NR30 FF1A E--- ---- DAC power
NR31 FF1B LLLL LLLL Length load (256-L)
NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
NR33 FF1D FFFF FFFF Frequency LSB
NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
*/

pub struct Wave {
    enabled: bool,
    length: u16,
    sound_tick_countdown: u16,
    wave_step: u8,
    settings: WaveSettings,
    wave_ram: [u8; 32],
}

impl Wave {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length: 0,
            sound_tick_countdown: 0,
            wave_step: 0,
            settings: WaveSettings::new(),
            wave_ram: [0_u8; 32],
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1A => (), // TODO
            0xFF1B...0xFF1E => self.settings.write_byte(addr, value),
            _ => unreachable!("Unreachable wave channel sound write operation: 0x{:X}", addr),
        }

        if (addr == 0xFF14 || addr == 0xFF19) && value & 0x80 > 0 {
            self.enabled = true;
            self.length = self.settings.sound_length;
        }
    }

    pub fn read_wave_ram(&self, addr: u16) -> u8 {
        self.wave_ram[2 * (addr - 0xFF30) as usize] << 4 | self.wave_ram[1 + 2 * (addr - 0xFF30) as usize]
    }

    pub fn write_wave_ram(&mut self, addr: u16, value: u8) {
        self.wave_ram[2 * (addr - 0xFF30) as usize] = value >> 4;
        self.wave_ram[1 + 2 * (addr - 0xFF30) as usize] = value & 0x0F;
    }

    // called at 256Hz (4096 CPU cycles)
    // To get 44100 samples, this function should return about 173 samples
    // Each of those samples ~24 cycles apart
    pub fn generate_sound(&mut self) -> [f32; Sound::SAMPLES_PER_CALL as usize] {
        let mut sound = [0_f32; Sound::SAMPLES_PER_CALL as usize];
        if !self.enabled || self.settings.volume == 0.0 {
            return sound;
        }

        // every 23 cycles, add item to array
        // every period, adjust square_wave_step
        // run for total of Sound::CYCLES_PER_TICK cycles
        for index in 0..Sound::SAMPLES_PER_CALL {
            sound[index as usize] = self.settings.volume * f32::from(self.wave_ram[self.wave_step as usize]);
            self.decrement_sound_tick_countdown(Sound::CYCLES_PER_SOUND);
        }

        self.decrement_sound_tick_countdown(Sound::ADDITIONAL_CYCLES_PER_TICK);

        sound
    }

    fn decrement_sound_tick_countdown(&mut self, amount: u16) {
        let mut amount = amount;
        while self.sound_tick_countdown < amount {
            amount -= self.sound_tick_countdown;
            self.sound_tick_countdown = (2048 - self.settings.frequency) / 2;
            self.wave_step = (self.wave_step + 1) % 32;
        }

        self.sound_tick_countdown -= amount;
    }

    pub fn decrement_length(&mut self) {
        if self.settings.length_enabled && self.length > 0 {
            self.length -= 1;
            if self.length == 0 {
                self.enabled = false;
            }
        }
    }
}
