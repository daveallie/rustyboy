mod channel;
mod player;
mod settings;

use cpu::CPU;
use sound::channel::noise::Noise;
use sound::channel::square::Square;
use sound::player::Player;

pub struct Sound {
    reg_values: [u8; 0x17], // store reg values here as shadow register is used in channels
    cycle_counter: u32,
    tick_counter: u8,
    square1: Square,
    square2: Square,
    noise: Noise,
    player: Player,
}

impl Sound {
    pub const CYCLES_PER_TICK: u32 = CPU::CYCLE_SPEED / 256; // 4096
    pub const CYCLES_PER_SOUND: u16 = 23;
    pub const SAMPLES_PER_CALL: u16 = 173;
    pub const ADDITIONAL_CYCLES_PER_TICK: u16 =
        (Self::CYCLES_PER_TICK - (Self::SAMPLES_PER_CALL * Self::CYCLES_PER_SOUND) as u32) as u16;

    pub fn new() -> Self {
        Self {
            reg_values: [0; 0x17],
            cycle_counter: 0,
            tick_counter: 0,
            square1: Square::new(true),
            square2: Square::new(false),
            noise: Noise::new(),
            player: Player::new(),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.reg_values[(addr - 0xFF10) as usize]
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.reg_values[(addr - 0xFF10) as usize] = value;
        match addr {
            0xFF10...0xFF14 => self.square1.write_byte(addr, value),
            0xFF15...0xFF19 => self.square2.write_byte(addr, value),
            0xFF1A...0xFF1E => (), // wave
            0xFF20...0xFF23 => self.noise.write_byte(addr, value),
            0xFF24...0xFF26 => (), // control/status
            _ => unreachable!("Unreachable sound read operation: 0x{:X}", addr),
        }
    }

    pub fn run_cycle(&mut self, cycles: u8) {
        self.cycle_counter += u32::from(cycles);
        if self.cycle_counter < Self::CYCLES_PER_TICK {
            return;
        }

        self.cycle_counter -= Self::CYCLES_PER_TICK;
        self.tick_counter += 1;

        let square1_sound = self.square1.generate_sound();
        let square2_sound = self.square2.generate_sound();
        let noise_sound = self.noise.generate_sound();
        let mut output = [0_f32; Sound::SAMPLES_PER_CALL as usize];
        for i in 0..(Sound::SAMPLES_PER_CALL as usize) {
            output[i] = (square1_sound[i] + square2_sound[i] + noise_sound[i]) / 15.0 / 4.0;
        }

        self.square1.decrement_length();
        self.square2.decrement_length();
        self.noise.decrement_length();

        if self.tick_counter % 2 == 0 {
            self.square1.tick_sweep();
            self.square2.tick_sweep();
        }

        if self.tick_counter % 4 == 0 {
            self.square1.tick_volume_envelope();
            self.square2.tick_volume_envelope();
            self.noise.tick_volume_envelope();
            self.tick_counter = 0;
        }

        self.player.play(&output, &output)
    }
}
