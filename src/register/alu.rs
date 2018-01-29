use register::{Flags, Registers};

impl Registers {
    pub fn alu_inc(&mut self, input: u8) -> u8 {
        let result = input.wrapping_add(1);
        // https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
        self.set_flag(Flags::H, (input & 0x0F) + 1 == 0x10);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::N, false);
        result
    }

    pub fn alu_dec(&mut self, input: u8) -> u8 {
        let result = input.wrapping_sub(1);
        self.set_flag(Flags::H, (input & 0x0F) == 0);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::N, true);
        result
    }

    pub fn alu_sub(&mut self, input: u8, usec: bool) {
        let carry: u8 =
            if usec && self.get_flag(Flags::C) {
                1
            } else {
                0
            };

        let a = self.a;
        let result = a.wrapping_sub(input).wrapping_sub(carry);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::H, (a & 0x0F) < (input & 0x0F) + carry);
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::C, (a as u16) < (input as u16) + (carry as u16));
        self.a = result;
    }

    pub fn alu_xor(&mut self, input: u8) {
        let result = self.a ^ input;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        self.a = result;
    }

    pub fn alu_cp(&mut self, input: u8) {
        let temp_a = self.a;
        self.alu_sub(input, false);
        self.a = temp_a;
    }
}
