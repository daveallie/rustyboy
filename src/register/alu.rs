use register::{Flags, Registers};

impl Registers {
    // --------------- 8-BIT ALU ---------------

    // Add n to A
    pub fn alu_add(&mut self, input: u8) {
        let a = self.a;
        let result = a.wrapping_add(input);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::H, (a & 0x0F) + (input & 0x0F) > 0x0F);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::C, 0xFF - a < input);
        self.a = result;
    }

    // Add n + Carry flag to A
    pub fn alu_adc(&mut self, input: u8) {
        let a = self.a;
        let carry_bit = if self.get_flag(Flags::C) {
            0x01
        } else {
            0x00
        };

        let result = a.wrapping_add(input).wrapping_add(carry_bit);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::H, (a & 0x0F) + (input & 0x0F) + carry_bit > 0x0F);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::C, u16::from(0xFF - a) < (u16::from(input) + u16::from(carry_bit)));
        self.a = result;
    }

    // Subtract n from A
    pub fn alu_sub(&mut self, input: u8) {
        let a = self.a;
        let result = a.wrapping_sub(input);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::H, (a & 0x0F) < (input & 0x0F));
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::C, a < input);
        self.a = result;
    }

    // Subtract n + Carry flag from A
    pub fn alu_sbc(&mut self, input: u8) {
        let a = self.a;
        let carry_bit = if self.get_flag(Flags::C) {
            0x01
        } else {
            0x00
        };

        let result = a.wrapping_sub(input).wrapping_sub(carry_bit);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::H, (a & 0x0F) < ((input & 0x0F) + carry_bit));
        self.set_flag(Flags::N, true);
        self.set_flag(Flags::C, u16::from(a) < (u16::from(input) + u16::from(carry_bit)));
        self.a = result;
    }

    // Logically AND n with A, result in A
    pub fn alu_and(&mut self, input: u8) {
        let result = self.a & input;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, false);
        self.set_flag(Flags::H, true);
        self.set_flag(Flags::N, false);
        self.a = result;
    }

    // Logical OR n with register A, result in A
    pub fn alu_or(&mut self, input: u8) {
        let result = self.a | input;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        self.a = result;
    }

    // Logical exclusive OR n with register A, result in A
    pub fn alu_xor(&mut self, input: u8) {
        let result = self.a ^ input;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        self.a = result;
    }

    // Compare A with n. This is basically an A - n subtraction instruction but the results are thrown away
    pub fn alu_cp(&mut self, input: u8) {
        let old_a = self.a;
        self.alu_sub(input);
        self.a = old_a;
    }

    // Increment register n
    pub fn alu_inc(&mut self, input: u8) -> u8 {
        let result = input.wrapping_add(1);
        // https://robdor.com/2016/08/10/gameboy-emulator-half-carry-flag/
        self.set_flag(Flags::H, (input & 0x0F) + 1 == 0x10);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::N, false);
        result
    }

    // Decrement register n
    pub fn alu_dec(&mut self, input: u8) -> u8 {
        let result = input.wrapping_sub(1);
        self.set_flag(Flags::H, input.trailing_zeros() >= 4);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::N, true);
        result
    }

    // --------------- 16-BIT ARITHMETIC ---------------

    // Add n to HL
    pub fn alu_add_hl(&mut self, input: u16) {
        let hl = self.get_hl();
        let result = hl.wrapping_add(input);
        self.set_flag(Flags::H, (hl & 0x07FF) + (input & 0x07FF) > 0x07FF);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::C, 0xFFFF - hl < input);
        self.set_hl(result);
    }

    // Add 16 bit number and 8 bit number
    pub fn alu_add_16_and_8(&mut self, input16: u16, input8: i8) -> u16 {
        let input16_adj = i32::from(input16);
        let input8_adj = i32::from(input8);
        let result = input16_adj.wrapping_add(input8_adj);

        self.set_flag(Flags::N, false);
        self.set_flag(Flags::Z, false);
        self.set_flag(Flags::H, (input16_adj ^ input8_adj ^ result) & 0x10 != 0);
        self.set_flag(Flags::C, (input16_adj ^ input8_adj ^ result) & 0x100 != 0);
        #[cfg_attr(feature="clippy", allow(cast_sign_loss, cast_possible_truncation))]
        let casted_result = result as u16;
        casted_result
    }

    // --------------- MISCELLANEOUS ---------------

    pub fn alu_nible_swap(&mut self, input: u8) -> u8 {
        // Swap upper & lower nibles of n
        let result = ((input & 0x0F) << 4) | ((input & 0xF0) >> 4);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }

    // Decimal adjust register A.
    //This instruction adjusts register A so that the correct representation of Binary Coded Decimal (BCD) is obtained
    // https://ehaskins.com/2018-01-30%20Z80%20DAA/
    pub fn alu_daa(&mut self) {
        let a = self.a;
        let mut correction = 0;

        if self.get_flag(Flags::H) || (!self.get_flag(Flags::N) && (a & 0x0F) > 9) {
            correction |= 0x06;
        }

        if self.get_flag(Flags::C) || (!self.get_flag(Flags::N) && a > 0x99) {
            correction |= 0x60;
            self.set_flag(Flags::C, true);
        } else {
            self.set_flag(Flags::C, false);
        }

        let result = if self.get_flag(Flags::N) {
            a.wrapping_sub(correction)
        } else {
            a.wrapping_add(correction)
        };

        self.a = result;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::H, false);
    }

    // Complement A register. (Flip all bits.)
    pub fn alu_cpl(&mut self) {
        self.a = !self.a;
        self.set_flag(Flags::C, true);
        self.set_flag(Flags::H, true);
    }

    // Complement carry flag.
    // If C flag is set, then reset it. If C flag is reset, then set it.
    pub fn alu_ccf(&mut self) {
        let c_was = self.get_flag(Flags::C);
        self.set_flag(Flags::C, !c_was);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, false);
    }

    // Set Carry flag
    pub fn alu_scf(&mut self) {
        self.set_flag(Flags::C, true);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, false);
    }

    // --------------- ROTATING AND SHIFTING ---------------

    // Rotate n left. Old bit 7 to Carry flag.
    pub fn alu_rlc(&mut self, input: u8) -> u8 {
        let msb_was_set = input & 0x80 > 0;
        let result = if msb_was_set {
            input << 1 | 0x01
        } else {
            input << 1
        };
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, msb_was_set);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }

    // Rotate n left through Carry flag.
    pub fn alu_rl(&mut self, input: u8) -> u8 {
        let msb_was_set = input & 0x80 > 0;
        let result = if self.get_flag(Flags::C) {
            input << 1 | 0x01
        } else {
            input << 1
        };
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, msb_was_set);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }

    // Rotate n right. Old bit 0 to Carry flag.
    pub fn alu_rrc(&mut self, input: u8) -> u8 {
        let lsb_was_set = input & 0x01 > 0;
        let result = if lsb_was_set {
            input >> 1 | 0x80
        } else {
            input >> 1
        };

        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, lsb_was_set);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }

    // Rotate n right through Carry flag.
    pub fn alu_rr(&mut self, input: u8) -> u8 {
        let lsb_was_set = input & 0x01 > 0;
        let result = if self.get_flag(Flags::C) {
            input >> 1 | 0x80
        } else {
            input >> 1
        };
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, lsb_was_set);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }

    // Shift n left into Carry. LSB of n set to 0.
    pub fn alu_sla(&mut self, input: u8) -> u8 {
        let result = input << 1;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, input & 0x80 > 0);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }

    // Shift n right into Carry. MSB doesn't change.
    pub fn alu_sra(&mut self, input: u8) -> u8 {
        let result = (input >> 1) | (input & 0x80);
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, input & 0x01 > 0);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }

    // Shift n right into Carry. MSB set to 0.
    pub fn alu_srl(&mut self, input: u8) -> u8 {
        let result = input >> 1;
        self.set_flag(Flags::Z, result == 0);
        self.set_flag(Flags::C, input & 0x01 > 0);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::N, false);
        result
    }

    // --------------- BIT OPCODES ---------------

    // Test bit b in register r
    pub fn alu_bit_test(&mut self, input: u8, bit: u8) {
        self.set_flag(Flags::Z, input & (1 << bit) == 0);
        self.set_flag(Flags::H, true);
        self.set_flag(Flags::N, false);
    }
}
