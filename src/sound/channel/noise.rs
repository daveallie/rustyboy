use sound::settings::noise_settings::NoiseSettings;
use sound::Sound;

pub struct Noise {
    enabled: bool,
    volume: u8,
    length: u8,
    volume_tick_counter: u8,
    sound_tick_countdown: u16,
    lfsr_reg: u16,
    settings: NoiseSettings,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            enabled: false,
            volume: 0,
            length: 0,
            volume_tick_counter: 0,
            sound_tick_countdown: 0,
            lfsr_reg: 1,
            settings: NoiseSettings::new(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF20...0xFF23 => self.settings.write_byte(addr, value),
            _ => unreachable!("Unreachable noise channel sound write operation: 0x{:X}", addr),
        }

        if addr == 0xFF23 && value & 0x80 > 0 {
            self.enabled = true;
            self.length = self.settings.sound_length;
            self.volume = self.settings.envelope.starting_volume;
        }
    }

    // called at 256Hz (4096 CPU cycles)
    // To get 44100 samples, this function should return about 173 samples
    // Each of those samples ~24 cycles apart
    pub fn generate_sound(&mut self) -> [f32; Sound::SAMPLES_PER_CALL as usize] {
        let mut sound = [0_f32; Sound::SAMPLES_PER_CALL as usize];
        if !self.enabled || self.volume == 0 {
            return sound;
        }

        // every 23 cycles, add item to array
        // every period, adjust square_wave_step
        // run for total of Sound::CYCLES_PER_TICK cycles
        for index in 0..Sound::SAMPLES_PER_CALL {
            sound[index as usize] = f32::from(self.volume) * f32::from(1 - (self.lfsr_reg & 0x01));
            self.decrement_sound_tick_countdown(Sound::CYCLES_PER_SOUND);
        }

        self.decrement_sound_tick_countdown(Sound::ADDITIONAL_CYCLES_PER_TICK);

        sound
    }

    fn decrement_sound_tick_countdown(&mut self, amount: u16) {
        let mut amount = amount;
        while self.sound_tick_countdown < amount {
            amount -= self.sound_tick_countdown;
            self.sound_tick_countdown = self.settings.frequency;
            let mut new_lfsr = self.lfsr_reg >> 1;
            if self.lfsr_reg & 0x01 | new_lfsr & 0x01 > 0 {
                new_lfsr |= 0x80;
                if self.settings.lfsr_7_bit_mode {
                    new_lfsr |= 0x20;
                }
            }
            self.lfsr_reg = new_lfsr;
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

    pub fn tick_volume_envelope(&mut self) {
        if self.settings.envelope.period == 0 {
            return;
        }

        self.volume_tick_counter += 1;
        if self.volume_tick_counter < self.settings.envelope.period {
            return;
        }

        self.volume_tick_counter = 0;
        self.volume = self.settings.envelope.tick(self.volume);
    }
}
