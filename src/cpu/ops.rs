use cpu::CPU;
use register::Flags;

impl CPU {
    pub fn step(&mut self) -> u8 {
        let read_regs = self.reg;
        let code = self.get_byte();

        #[cfg(feature = "debugger")]
        println!("instr: 0x{:X} -- opcode: 0x{:X}", read_regs.pc, code);

        match code {
            0x00 => { // nop
                1
            }
            0x01 => { // load word into bc
                let value = self.get_word();
                self.reg.set_bc(value);
                3
            }
            0x02 => { // write a into location pointed by bc
                self.mmu.write_byte(read_regs.get_bc(), read_regs.a);
                2
            }
            0x03 => { // inc bc
                self.reg.set_bc(read_regs.get_bc().wrapping_add(1));
                2
            }
            0x04 => { // inc b
                self.reg.b = self.reg.alu_inc(read_regs.b);
                1
            }
            0x05 => { // dec b
                self.reg.b = self.reg.alu_dec(read_regs.b);
                1
            }
            0x06 => { // load byte into b
                self.reg.b = self.get_byte();
                2
            }
            0x0B => { // dec bc
                self.reg.set_bc(read_regs.get_bc().wrapping_sub(1));
                2
            }
            0x0C => { // inc c
                self.reg.c = self.reg.alu_inc(read_regs.c);
                1
            }
            0x0D => { // dec c
                self.reg.c = self.reg.alu_dec(read_regs.c);
                1
            }
            0x0E => { // load byte into c
                self.reg.c = self.get_byte();
                2
            }
            0x14 => { // inc d
                self.reg.d = self.reg.alu_inc(read_regs.d);
                1
            }
            0x15 => { // dec d
                self.reg.d = self.reg.alu_dec(read_regs.d);
                1
            }
            0x1C => { // inc e
                self.reg.e = self.reg.alu_inc(read_regs.e);
                1
            }
            0x1D => { // dec e
                self.reg.e = self.reg.alu_dec(read_regs.e);
                1
            }
            0x20 => { // JR * if Z is reset
                if read_regs.get_flag(Flags::Z) {
                    self.reg.pc += 1; // Skip jump destination byte
                    2
                } else {
                    self.jr();
                    3
                }
            }
            0x21 => { // load word into hl
                let value = self.get_word();
                self.reg.set_hl(value);
                3
            }
            0x22 => { // write a into location pointed by hl (and inc hl)
                self.mmu.write_byte(self.reg.get_hl_and_inc(), read_regs.a);
                2
            }
            0x24 => { // inc h
                self.reg.h = self.reg.alu_inc(read_regs.h);
                1
            }
            0x25 => { // dec h
                self.reg.h = self.reg.alu_dec(read_regs.h);
                1
            }
            0x27 => { // DAA
                self.reg.alu_daa();
                1
            }
            0x28 => { // JR * if Z is set
                if read_regs.get_flag(Flags::Z) {
                    self.jr();
                    3
                } else {
                    self.reg.pc += 1; // Skip jump destination byte
                    2
                }
            }
            0x2A => { // load value at hl address into a. inc hl
                self.reg.a = self.mmu.read_byte(self.reg.get_hl_and_inc());
                2
            }
            0x2C => { // inc l
                self.reg.l = self.reg.alu_inc(read_regs.l);
                1
            }
            0x2D => { // dec l
                self.reg.l = self.reg.alu_dec(read_regs.l);
                1
            }
            0x30 => { // JR * if C is reset
                if read_regs.get_flag(Flags::C) {
                    self.reg.pc += 1; // Skip jump destination byte
                    2
                } else {
                    self.jr();
                    3
                }
            }
            0x31 => { // load word in sp
                let value = self.get_word();
                self.reg.sp = value;
                3
            }
            0x32 => { // write a into location pointed by hl (and dec hl)
                self.mmu.write_byte(self.reg.get_hl_and_dec(), read_regs.a);
                2
            }
            0x34 => { // inc byte pointed to by hl
                let addr = read_regs.get_hl();
                let value = self.reg.alu_inc(self.mmu.read_byte(addr));
                self.mmu.write_byte(addr, value);
                3
            }
            0x35 => { // dec byte pointed to by hl
                let addr = read_regs.get_hl();
                let value = self.reg.alu_dec(self.mmu.read_byte(addr));
                self.mmu.write_byte(addr, value);
                3
            }
            0x36 => { // load byte into location pointed by hl
                let value = self.get_byte();
                self.mmu.write_byte(read_regs.get_hl(), value);
                3
            }
            0x38 => { // JR * if C is set
                if read_regs.get_flag(Flags::C) {
                    self.jr();
                    3
                } else {
                    self.reg.pc += 1; // Skip jump destination byte
                    2
                }
            }
            0x3C => { // inc a
                self.reg.a = self.reg.alu_inc(read_regs.a);
                1
            }
            0x3D => { // dec a
                self.reg.a = self.reg.alu_dec(read_regs.a);
                1
            }
            0x76 => { // halt
                self.halting = true;
                1
            }
            0x77 => { // load a into location pointed by hl
                self.mmu.write_byte(read_regs.get_hl(), read_regs.a);
                2
            }
            0x78 => { // load b into a
                self.reg.a = read_regs.b;
                1
            }
            0x79 => { // load c into a
                self.reg.a = read_regs.c;
                1
            }
            0x7A => { // load d into a
                self.reg.a = read_regs.d;
                1
            }
            0x7B => { // load e into a
                self.reg.a = read_regs.e;
                1
            }
            0x7C => { // load h into a
                self.reg.a = read_regs.h;
                1
            }
            0x7D => { // load l into a
                self.reg.a = read_regs.l;
                1
            }
            0x7F => { // load a into a
                self.reg.a = self.reg.a;
                1
            }
            0xEA => { // load a into into location pointed by next word
                let addr = self.get_word();
                self.mmu.write_byte(addr, read_regs.a);
                4
            }
            0x3E => { // load byte into a
                self.reg.a = self.get_byte();
                2
            }
            0xA0 => { // and a and b, store in a
                self.reg.alu_and(read_regs.b);
                1
            }
            0xA1 => { // and a and c, store in a
                self.reg.alu_and(read_regs.c);
                1
            }
            0xA2 => { // and a and d, store in a
                self.reg.alu_and(read_regs.d);
                1
            }
            0xA3 => { // and a and e, store in a
                self.reg.alu_and(read_regs.e);
                1
            }
            0xA4 => { // and a and h, store in a
                self.reg.alu_and(read_regs.h);
                1
            }
            0xA5 => { // and a and l, store in a
                self.reg.alu_and(read_regs.l);
                1
            }
            0xA6 => { // and a and b, store in a
                self.reg.alu_and(self.mmu.read_byte(read_regs.get_hl()));
                2
            }
            0xA7 => { // and a and a, store in a
                self.reg.alu_and(read_regs.a);
                1
            }
            0xA8 => { // xor a and b, store in a
                self.reg.alu_xor(read_regs.b);
                1
            }
            0xA9 => { // xor a and c, store in a
                self.reg.alu_xor(read_regs.c);
                1
            }
            0xAA => { // xor a and d, store in a
                self.reg.alu_xor(read_regs.d);
                1
            }
            0xAB => { // xor a and e, store in a
                self.reg.alu_xor(read_regs.e);
                1
            }
            0xAC => { // xor a and h, store in a
                self.reg.alu_xor(read_regs.h);
                1
            }
            0xAD => { // xor a and l, store in a
                self.reg.alu_xor(read_regs.l);
                1
            }
            0xAF => { // xor a and a, store in a
                self.reg.alu_xor(read_regs.a);
                1
            }
            0xB0 => { // or a and b, store in a
                self.reg.alu_or(read_regs.b);
                1
            }
            0xB1 => { // or a and c, store in a
                self.reg.alu_or(read_regs.c);
                1
            }
            0xB2 => { // or a and d, store in a
                self.reg.alu_or(read_regs.d);
                1
            }
            0xB3 => { // or a and e, store in a
                self.reg.alu_or(read_regs.e);
                1
            }
            0xB4 => { // or a and h, store in a
                self.reg.alu_or(read_regs.h);
                1
            }
            0xB5 => { // or a and l, store in a
                self.reg.alu_or(read_regs.l);
                1
            }
            0xB7 => { // or a and a, store in a
                self.reg.alu_or(read_regs.a);
                1
            }
            0xC0 => { // load word off stack and move to that address if Z is reset
                if read_regs.get_flag(Flags::Z) {
                    2
                } else {
                    self.reg.pc = self.pop_stack();
                    5
                }
            }
            0xC1 => { // pop stack into bc
                let value = self.pop_stack();
                self.reg.set_bc(value);
                3
            }
            0xC3 => { // jump to location point by word
                self.reg.pc = self.get_word();
                3
            }
            0xC5 => { // push bc onto stack
                self.push_stack(read_regs.get_bc());
                4
            }
            0xC8 => { // load word off stack and move to that address if Z is set
                if read_regs.get_flag(Flags::Z) {
                    self.reg.pc = self.pop_stack();
                    5
                } else {
                    2
                }
            }
            0xC9 => { // load word off stack and move to that address
                self.reg.pc = self.pop_stack();
                4
            }
            0xCC => { // call next word conditional on Z flag
                let new_pc = self.reg.pc + 2;

                if self.reg.get_flag(Flags::Z) {
                    self.push_stack(new_pc);
                    self.reg.pc = self.get_word();
                    6
                } else {
                    self.reg.pc = new_pc;
                    3
                }
            }
            0xCD => { // call next word
                let new_pc = self.reg.pc + 2;

                self.push_stack(new_pc);
                self.reg.pc = self.get_word();
                6
            }
            0xD0 => { // load word off stack and move to that address if C is reset
                if read_regs.get_flag(Flags::C) {
                    2
                } else {
                    self.reg.pc = self.pop_stack();
                    5
                }
            }
            0xD1 => { // pop stack into de
                let value = self.pop_stack();
                self.reg.set_de(value);
                3
            }
            0xD5 => { // push de onto stack
                self.push_stack(read_regs.get_de());
                4
            }
            0xD8 => { // load word off stack and move to that address if C is set
                if read_regs.get_flag(Flags::C) {
                    self.reg.pc = self.pop_stack();
                    5
                } else {
                    2
                }
            }
            0xD9 => { // load word off stack and move to that address and enable interrupts
                self.reg.pc = self.pop_stack();
                self.enable_interrupt_after = 1;
                4
            }
            0xE0 => { // store a into (0xFF00 | next byte)
                let addr = 0xFF00 | u16::from(self.get_byte());
                self.mmu.write_byte(addr, read_regs.a);
                3
            }
            0xE1 => { // pop stack into hl
                let value = self.pop_stack();
                self.reg.set_hl(value);
                3
            }
            0xE2 => { // store a into (0xFF00 | C)
                let addr = 0xFF00 | u16::from(read_regs.c);
                self.mmu.write_byte(addr, read_regs.a);
                2
            }
            0xE5 => { // push hl onto stack
                self.push_stack(read_regs.get_hl());
                4
            }
            0xF0 => {
                let addr = 0xFF00 | u16::from(self.get_byte());
                self.reg.a = self.mmu.read_byte(addr);
                3
            }
            0xF1 => { // pop stack into hl
                let value = self.pop_stack();
                self.reg.set_af(value);
                3
            }
            0xF3 => { // disable interrupts after following cpu instruction
                self.disable_interrupt_after = 2;
                1
            }
            0xF5 => { // push af onto stack
                self.push_stack(read_regs.get_af());
                4
            }
            0xF9 => { // Load hl into stack pointer
                self.reg.sp = read_regs.get_hl();
                2
            }
            0xFA => { // load byte pointed to by next work into a
                let addr = self.get_word();
                self.reg.a = self.mmu.read_byte(addr);
                4
            }
            0xFB => { // enable interrupts after following cpu instruction
                self.enable_interrupt_after = 2;
                1
            }
            0xFE => {
                let input = self.get_byte();
                self.reg.alu_cp(input);
                2
            }
            _ => {
                panic!("unknown op code 0x{:X}", code)
            }
        }
    }
}
