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
        self.set_flag(Flags::H, input.trailing_zeros() >= 4);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::N, true);
        result
    }

    pub fn alu_add(&mut self, input: u8) {
        let a = self.a;
        let result = a.wrapping_add(input);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::H, (a & 0x0F) + (input & 0x0F) > 0x0F);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::C, 0xFF - a < input);
        self.a = result;
    }

    pub fn alu_sub(&mut self, input: u8) {
        let a = self.a;
        let result = a.wrapping_sub(input);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::H, (a & 0x0F) < (input & 0x0F));
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::C, a < input);
        self.a = result;
    }

    pub fn alu_add_16_bit(&mut self, input: u16) {
        let hl = self.get_hl();
        let result = hl.wrapping_add(input);
        self.set_flag(Flags::H, (hl & 0x07FF) + (input & 0x07FF) > 0x07FF);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::C, 0xFFFF - hl < input);
        self.set_hl(result);
    }

    pub fn alu_or(&mut self, input: u8) {
        let result = self.a | input;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
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

    pub fn alu_and(&mut self, input: u8) {
        let result = self.a & input;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, false);
        self.set_flag(Flags::H, true);
        self.set_flag(Flags::N, false);
        self.a = result;
    }

    pub fn alu_cp(&mut self, input: u8) {
        let temp_a = self.a;
        self.alu_sub(input);
        self.a = temp_a;
    }

    pub fn alu_daa(&mut self) {
        panic!("WTF IS THIS DAA SHIT")
    }

    pub fn alu_cpl(&mut self) {
        self.a = !self.a;
        self.set_flag(Flags::C, true);
        self.set_flag(Flags::H, true);
    }

    pub fn alu_nible_swap(&mut self, input: u8) -> u8 {
        let result = ((input & 0x0F) << 4) | ((input & 0xF0) >> 4);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }
}
