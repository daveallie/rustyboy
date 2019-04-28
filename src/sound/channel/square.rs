use sound::settings::square_settings::SquareSettings;
use sound::Sound;

/*
       Square 1
NR10 FF10 -PPP NSSS Sweep period, negate, shift
NR11 FF11 DDLL LLLL Duty, Length load (64-L)
NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
NR13 FF13 FFFF FFFF Frequency LSB
NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB

       Square 2
     FF15 ---- ---- Not used
NR21 FF16 DDLL LLLL Duty, Length load (64-L)
NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
NR23 FF18 FFFF FFFF Frequency LSB
NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB
*/

pub struct Square {
    enabled: bool,
    sweep_enabled: bool,
    volume: u8,
    frequency: u16,
    length: u8,
    square_wave_step: u8,
    volume_tick_counter: u8,
    sweep_tick_counter: u8,
    sound_tick_countdown: u16,
    settings: SquareSettings,
}

impl Square {
    const DUTY_LAYOUTS: [[u8; 8]; 4] = [
        [0, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1, 1, 1],
        [0, 1, 1, 1, 1, 1, 1, 0],
    ];

    pub fn new(sweep_enabled: bool) -> Self {
        Self {
            enabled: false,
            sweep_enabled,
            volume: 0,
            frequency: 0,
            length: 0,
            square_wave_step: 0,
            volume_tick_counter: 0,
            sweep_tick_counter: 0,
            sound_tick_countdown: 0,
            settings: SquareSettings::new(),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10...0xFF14 | 0xFF16...0xFF19 => self.settings.write_byte(addr, value),
            _ => unreachable!("Unreachable noise channel sound write operation: 0x{:X}", addr),
        }

        if (addr == 0xFF14 || addr == 0xFF19) && value & 0x80 > 0 {
            self.enabled = true;
            self.length = self.settings.sound_length;
            self.frequency = self.settings.frequency;
            self.volume = self.settings.envelope.starting_volume;
            if self.sweep_enabled && self.settings.sweep.shift > 0 && self.settings.sweep.period > 0 {
                self.sweep_tick_counter = 0;
                self.tick_sweep();
            }
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
            sound[index as usize] = f32::from(self.volume)
                * f32::from(Self::DUTY_LAYOUTS[self.settings.duty as usize][self.square_wave_step as usize]);
            self.decrement_sound_tick_countdown(Sound::CYCLES_PER_SOUND);
        }

        self.decrement_sound_tick_countdown(Sound::ADDITIONAL_CYCLES_PER_TICK);

        sound
    }

    fn decrement_sound_tick_countdown(&mut self, amount: u16) {
        let mut amount = amount;
        while self.sound_tick_countdown < amount {
            amount -= self.sound_tick_countdown;
            self.sound_tick_countdown = 2048 - self.frequency;
            self.square_wave_step = (self.square_wave_step + 1) % 8;
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

    pub fn tick_sweep(&mut self) {
        if !self.sweep_enabled || self.settings.sweep.period == 0 && self.settings.sweep.shift == 0 {
            return;
        }

        self.sweep_tick_counter += 1;
        if self.sweep_tick_counter < self.settings.sweep.period {
            return;
        }

        self.sweep_tick_counter = 0;
        self.frequency = self.settings.sweep.tick(self.frequency);
        if self.frequency == 2048 {
            self.enabled = false;
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
