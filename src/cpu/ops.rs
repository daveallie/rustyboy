use cpu::CPU;
use register::Flags;

impl CPU {
    pub fn step(&mut self) -> u32 {
        let read_regs = self.reg;
        let code = self.get_byte();

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
                self.mmu.write_byte(self.reg.get_bc(), read_regs.a);
                2
            }
            0x03 => { // inc bc
                let value = self.reg.get_bc().wrapping_add(1);
                self.reg.set_bc(value);
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
                let value = self.reg.get_bc().wrapping_sub(1);
                self.reg.set_bc(value);
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
            0x20 => { // JR * if Z is reset
                if self.reg.get_flag(Flags::Z) {
                    self.reg.pc += 1;
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
            0x2A => { // load value at hl address into a. inc hl
                self.mmu.write_byte(self.reg.get_hl_and_inc(), read_regs.a);
                2
            }
            0x2C => { // inc l
                self.reg.l = self.reg.alu_inc(read_regs.l);
                1
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
            0x36 => { // load byte into location pointed by hl
                let value = self.get_byte();
                self.mmu.write_byte(self.reg.get_hl(), value);
                3
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
                // self.reg.a = self.reg.a;
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
            0xC3 => { // jump to location point by word
                self.reg.pc = self.get_word();
                3
            }
            0xC9 => { // load word off stack and move to that address
                let addr = self.mmu.read_word(read_regs.sp);
                self.reg.sp = read_regs.sp + 2;
                self.reg.pc = addr;
                2
            }
            0xCC => { // call next word conditional on Z flag
                let new_pc = read_regs.pc + 2;

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
                self.push_stack(read_regs.pc + 2);
                self.reg.pc = self.get_word();
                6
            }
            0xE0 => { // store a into (0xFF00 | next byte)
                let addr = 0xFF00 | self.get_byte() as u16;
                self.mmu.write_byte(addr, read_regs.a);
                3
            }
            0xE2 => { // store a into (0xFF00 | C)
                let addr = 0xFF00 | self.reg.c as u16;
                self.mmu.write_byte(addr, read_regs.a);
                2
            }
            0xF0 => {
                let addr = 0xFF00 | self.get_byte() as u16;
                self.reg.a = self.mmu.read_byte(addr);
                3
            }
            0xF3 => {
                self.disable_interrupt = 2;
                1
            }
            0xF9 => { // Load hl into stack pointer
                self.reg.sp = self.reg.get_hl();
                2
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
