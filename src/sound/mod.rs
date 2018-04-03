mod channel;

use sound::channel::noise::Noise;

pub struct Sound {
    noise: Noise,
}

impl Sound {
    pub fn new() -> Self {
        Self {
            noise: Noise::new(),
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10...0xFF1F | 0xFF24...0xFF26 => 0, // TODO: Implement
            0xFF20...0xFF23 => self.noise.read_byte(addr),
            _ => unreachable!("Unreachable sound read operation: 0x{:X}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10...0xFF1F | 0xFF24...0xFF26 => (), // TODO: Implement
            0xFF20...0xFF23 => self.noise.write_byte(addr, value),
            _ => unreachable!("Unreachable sound read operation: 0x{:X}", addr),
        }
    }
}
