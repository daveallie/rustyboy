mod ops;

const CLOCK_SPEED: f64 = 4194304f64;

pub struct CPU {
    reg: ::register::Registers,
    mmu: ::mmu::MMU,
}

impl CPU {
    pub fn new(cart_path: &str) -> CPU {
        CPU {
            reg: ::register::Registers::new(),
            mmu: ::mmu::MMU::new(cart_path),
        }
    }

    fn get_byte(&mut self) -> u8 {
        let byte = self.mmu.read_byte(self.reg.pc);
        self.reg.pc += 1;
        byte
    }

    fn get_word(&mut self) -> u16 {
        let word = self.mmu.read_word(self.reg.pc);
        self.reg.pc += 2;
        word
    }
}
