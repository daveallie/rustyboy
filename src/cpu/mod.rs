mod ops;

pub struct CPU {
    reg: ::register::Registers,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            reg: ::register::Registers::new()
        }
    }

    fn get_byte(&mut self) -> u8 {
        0
    }

    fn get_word(&mut self) -> u16 {
        (self.get_byte() as u16) << 8 | (self.get_byte() as u16)
    }
}
