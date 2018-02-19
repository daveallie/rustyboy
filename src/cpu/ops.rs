use cpu::CPU;
use register::Flags;

impl CPU {
    pub fn call_reg_op(&mut self) -> u8 {
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
            0x09 => { // add bc to hl, store in hl
                self.reg.alu_add_16_bit(read_regs.get_bc());
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
            0x12 => { // write a into location pointed by de
                self.mmu.write_byte(read_regs.get_de(), read_regs.a);
                2
            }
            0x13 => { // inc de
                self.reg.set_de(read_regs.get_de().wrapping_add(1));
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
            0x16 => { // load byte into d
                self.reg.d = self.get_byte();
                2
            }
            0x19 => { // add de to hl, store in hl
                self.reg.alu_add_16_bit(read_regs.get_de());
                2
            }
            0x1C => { // inc e
                self.reg.e = self.reg.alu_inc(read_regs.e);
                1
            }
            0x1D => { // dec e
                self.reg.e = self.reg.alu_dec(read_regs.e);
                1
            }
            0x1E => { // load byte into e
                self.reg.e = self.get_byte();
                2
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
            0x23 => { // inc hl
                self.reg.set_hl(read_regs.get_hl().wrapping_add(1));
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
            0x26 => { // load byte into h
                self.reg.h = self.get_byte();
                2
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
            0x29 => { // add hl to hl, store in hl
                self.reg.alu_add_16_bit(read_regs.get_hl());
                2
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
            0x2E => { // load byte into l
                self.reg.l = self.get_byte();
                2
            }
            0x2F => { // compliment a
                self.reg.alu_cpl();
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
            0x33 => { // inc hl
                self.reg.sp = read_regs.sp.wrapping_add(1);
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
            0x39 => { // add sp to hl, store in sp
                self.reg.alu_add_16_bit(read_regs.sp);
                2
            }
            0x3C => { // inc a
                self.reg.a = self.reg.alu_inc(read_regs.a);
                1
            }
            0x3D => { // dec a
                self.reg.a = self.reg.alu_dec(read_regs.a);
                1
            }
            0x40 => { // load b into b
                self.reg.b = read_regs.b;
                1
            }
            0x41 => { // load c into b
                self.reg.b = read_regs.c;
                1
            }
            0x42 => { // load d into b
                self.reg.b = read_regs.d;
                1
            }
            0x43 => { // load e into b
                self.reg.b = read_regs.e;
                1
            }
            0x44 => { // load h into b
                self.reg.b = read_regs.h;
                1
            }
            0x45 => { // load l into b
                self.reg.b = read_regs.l;
                1
            }
            0x46 => { // load byte pointed to by hl into b
                let value = self.mmu.read_byte(read_regs.get_hl());
                self.reg.b = value;
                2
            }
            0x47 => { // load a into b
                self.reg.b = read_regs.a;
                1
            }
            0x48 => { // load b into c
                self.reg.c = read_regs.b;
                1
            }
            0x49 => { // load c into c
                self.reg.c = read_regs.c;
                1
            }
            0x4A => { // load d into c
                self.reg.c = read_regs.d;
                1
            }
            0x4B => { // load e into c
                self.reg.c = read_regs.e;
                1
            }
            0x4C => { // load h into c
                self.reg.c = read_regs.h;
                1
            }
            0x4D => { // load l into c
                self.reg.c = read_regs.l;
                1
            }
            0x4E => { // load byte pointed to by hl into c
                let value = self.mmu.read_byte(read_regs.get_hl());
                self.reg.c = value;
                2
            }
            0x4F => { // load a into c
                self.reg.c = read_regs.a;
                1
            }
            0x50 => { // load b into d
                self.reg.d = read_regs.b;
                1
            }
            0x51 => { // load c into d
                self.reg.d = read_regs.c;
                1
            }
            0x52 => { // load d into d
                self.reg.d = read_regs.d;
                1
            }
            0x53 => { // load e into d
                self.reg.d = read_regs.e;
                1
            }
            0x54 => { // load h into d
                self.reg.d = read_regs.h;
                1
            }
            0x55 => { // load l into d
                self.reg.d = read_regs.l;
                1
            }
            0x56 => { // load byte pointed to by hl into d
                let value = self.mmu.read_byte(read_regs.get_hl());
                self.reg.d = value;
                2
            }
            0x57 => { // load a into d
                self.reg.d = read_regs.a;
                1
            }
            0x58 => { // load b into e
                self.reg.e = read_regs.b;
                1
            }
            0x59 => { // load c into e
                self.reg.e = read_regs.c;
                1
            }
            0x5A => { // load d into e
                self.reg.e = read_regs.d;
                1
            }
            0x5B => { // load e into e
                self.reg.e = read_regs.e;
                1
            }
            0x5C => { // load h into e
                self.reg.e = read_regs.h;
                1
            }
            0x5D => { // load l into e
                self.reg.e = read_regs.l;
                1
            }
            0x5E => { // load byte pointed to by hl into e
                let value = self.mmu.read_byte(read_regs.get_hl());
                self.reg.e = value;
                2
            }
            0x5F => { // load a into e
                self.reg.e = read_regs.a;
                1
            }
            0x60 => { // load b into h
                self.reg.h = read_regs.b;
                1
            }
            0x61 => { // load c into h
                self.reg.h = read_regs.c;
                1
            }
            0x62 => { // load d into h
                self.reg.h = read_regs.d;
                1
            }
            0x63 => { // load e into h
                self.reg.h = read_regs.e;
                1
            }
            0x64 => { // load h into h
                self.reg.h = read_regs.h;
                1
            }
            0x65 => { // load l into h
                self.reg.h = read_regs.l;
                1
            }
            0x66 => { // load byte pointed to by hl into h
                let value = self.mmu.read_byte(read_regs.get_hl());
                self.reg.h = value;
                2
            }
            0x67 => { // load a into h
                self.reg.h = read_regs.a;
                1
            }
            0x68 => { // load b into l
                self.reg.l = read_regs.b;
                1
            }
            0x69 => { // load c into l
                self.reg.l = read_regs.c;
                1
            }
            0x6A => { // load d into l
                self.reg.l = read_regs.d;
                1
            }
            0x6B => { // load e into l
                self.reg.l = read_regs.e;
                1
            }
            0x6C => { // load h into l
                self.reg.l = read_regs.h;
                1
            }
            0x6D => { // load l into l
                self.reg.l = read_regs.l;
                1
            }
            0x6E => { // load byte pointed to by hl into l
                let value = self.mmu.read_byte(read_regs.get_hl());
                self.reg.l = value;
                2
            }
            0x6F => { // load a into l
                self.reg.l = read_regs.a;
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
            0x80 => { // add a and b, store in a
                self.reg.alu_add(read_regs.b);
                1
            }
            0x81 => { // add a and c, store in a
                self.reg.alu_add(read_regs.c);
                1
            }
            0x82 => { // add a and d, store in a
                self.reg.alu_add(read_regs.d);
                1
            }
            0x83 => { // add a and e, store in a
                self.reg.alu_add(read_regs.e);
                1
            }
            0x84 => { // add a and h, store in a
                self.reg.alu_add(read_regs.h);
                1
            }
            0x85 => { // add a and l, store in a
                self.reg.alu_add(read_regs.l);
                1
            }
            0x86 => { // add a and byte pointed to by hl, store in a
                let value = self.mmu.read_byte(read_regs.get_hl());
                self.reg.alu_add(value);
                2
            }
            0x87 => { // add a and a, store in a
                self.reg.alu_add(read_regs.a);
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
            0xC7 => { // push pc to stack and jump to 0x00
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = 0;
                8
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
            0xCB => { // run a cb command
                self.call_cb_op()
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
            0xCF => { // push pc to stack and jump to 0x08
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = 0x08;
                8
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
            0xD7 => { // push pc to stack and jump to 0x10
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = 0x10;
                8
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
            0xDF => { // push pc to stack and jump to 0x18
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = 0x18;
                8
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
            0xE6 => { // and a and next byte, store in a
                let value = self.get_byte();
                self.reg.alu_and(value);
                2
            }
            0xE7 => { // push pc to stack and jump to 0x20
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = 0x20;
                8
            }
            0xE5 => { // jump to address in hl
                let hl = read_regs.get_hl();
                self.reg.pc = hl;
                1
            }
            0xEF => { // push pc to stack and jump to 0x28
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = 0x28;
                8
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
            0xF7 => { // push pc to stack and jump to 0x30
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = 0x30;
                8
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
            0xFF => { // push pc to stack and jump to 0x38
                let old_pc = self.reg.pc;
                self.push_stack(old_pc);
                self.reg.pc = 0x38;
                8
            }
            _ => {
                panic!("unknown op code 0x{:X} at 0x{:X}", code, read_regs.pc)
            }
        }
    }

    fn call_cb_op(&mut self) -> u8 {
        let read_regs = self.reg;
        let code = self.get_byte();

        match code {
            0x30 => { // swap nibles of b https://www.geeksforgeeks.org/swap-two-nibbles-byte/
                self.reg.b = self.reg.alu_nible_swap(read_regs.b);
                2
            }
            0x31 => { // swap nibles of c https://www.geeksforgeeks.org/swap-two-nibbles-byte/
                self.reg.c = self.reg.alu_nible_swap(read_regs.c);
                2
            }
            0x32 => { // swap nibles of d https://www.geeksforgeeks.org/swap-two-nibbles-byte/
                self.reg.d = self.reg.alu_nible_swap(read_regs.d);
                2
            }
            0x33 => { // swap nibles of e https://www.geeksforgeeks.org/swap-two-nibbles-byte/
                self.reg.e = self.reg.alu_nible_swap(read_regs.e);
                2
            }
            0x34 => { // swap nibles of h https://www.geeksforgeeks.org/swap-two-nibbles-byte/
                self.reg.h = self.reg.alu_nible_swap(read_regs.h);
                2
            }
            0x35 => { // swap nibles of l https://www.geeksforgeeks.org/swap-two-nibbles-byte/
                self.reg.l = self.reg.alu_nible_swap(read_regs.l);
                2
            }
            0x36 => { // swap nibles of byte at hl https://www.geeksforgeeks.org/swap-two-nibbles-byte/
                let addr = read_regs.get_hl();
                let value = self.mmu.read_byte(addr);
                self.mmu.write_byte(addr, self.reg.alu_nible_swap(value));
                4
            }
            0x37 => { // swap nibles of a https://www.geeksforgeeks.org/swap-two-nibbles-byte/
                self.reg.a = self.reg.alu_nible_swap(read_regs.a);
                2
            }
            _ => {
                panic!("unknown CB op code 0x{:X} at 0x{:X}", code, read_regs.pc)
            }
        }
    }
}
