use cpu::CPU;

pub struct Clock {
    divider: u8,
    divider_increment_timer: u32,
    counter: u8,
    counter_increment_timer: u32,
    modulo: u8,
    control: u8,
    pub interrupt: u8,
}

impl Clock {
    const DIVIDER_INC_SPEED: u32 = 0x4000_u32; // 16_384
    const DIVIDER_TIMER_LIMIT: u32 = CPU::CYCLE_SPEED / Self::DIVIDER_INC_SPEED; // 64

    pub fn new() -> Self {
        Self {
            divider: 0,
            divider_increment_timer: 0,
            counter: 0,
            counter_increment_timer: 0,
            modulo: 0,
            control: 0,
            interrupt: 0,
        }
    }

    pub fn run_cycle(&mut self, cpu_cycles: u8) {
        self.divider_increment_timer += u32::from(cpu_cycles);
        while self.divider_increment_timer >= Self::DIVIDER_TIMER_LIMIT {
            self.divider.wrapping_add(1);
            self.divider_increment_timer -= Self::DIVIDER_TIMER_LIMIT;
        }

        if self.counter_stopped() {
            return;
        }

        self.counter_increment_timer += u32::from(cpu_cycles);
        let counter_timer_limit = CPU::CYCLE_SPEED / self.counter_speed();
        while self.counter_increment_timer >= counter_timer_limit {
            self.counter.wrapping_add(1);
            self.counter_increment_timer -= counter_timer_limit;
            if self.counter == 0 {
                self.interrupt = 0x04;
                self.counter = self.modulo;
            }
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.divider,
            0xFF05 => self.counter,
            0xFF06 => self.modulo,
            0xFF07 => self.control,
            _ => unreachable!("Tried to read non-existent clock address"),
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.divider = 0,
            0xFF05 => self.counter = value,
            0xFF06 => self.modulo = value,
            0xFF07 => self.control = value,
            _ => unreachable!("Tried to write non-existent clock address"),
        }
    }

    fn counter_stopped(&self) -> bool {
        self.control & 0x04 == 0
    }

    fn counter_speed(&self) -> u32 {
        match self.control & 0x03 {
            0x00 => 0x1_000,
            0x01 => 0x40_000,
            0x02 => 0x10_000,
            _ => 0x4_000,
        }
    }
}
