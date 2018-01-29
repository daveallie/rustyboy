use cpu::CPU;
use register::Flags;

impl CPU {
    pub fn step(&mut self) -> u32 {
        let read_regs = self.reg;
        let code = self.get_byte();

        println!("instr: 0x{:X} -- opcode: 0x{:X}", read_regs.pc, code);

        // http://clrhome.org/table/
        // http://gbdev.gg8.se/wiki/articles/CPU_Comparision_with_Z80
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
                1
            }
            0x03 => { // inc bc
                let value = self.reg.get_bc().wrapping_add(1);
                self.reg.set_bc(value);
                1
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
            0x2C => { // inc l
                self.reg.l = self.reg.alu_inc(read_regs.l);
                1
            }
            0x32 => { // write a into location pointed by hl (and dec hl)
                self.mmu.write_byte(self.reg.get_hl_and_dec(), read_regs.a);
                2
            }
            0x36 => { // load byte into location pointed by hl
                let value = self.get_byte();
                self.mmu.write_byte(self.reg.get_hl(), value);
                2
            }
            0x3E => { // load byte into a
                self.reg.a = self.get_byte();
                2
            }
            0xAF => {
                self.reg.alu_xor(read_regs.a);
                1
            }
            0xC3 => { // jump to location point by word
                self.reg.pc = self.get_word();
                3
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
            0xE0 => { // store a into (0xFF00 | next byte)
                let addr = 0xFF00 | self.get_byte() as u16;
                self.mmu.write_byte(addr, read_regs.a);
                3
            }
            0xF3 => {
                self.disable_interrupt = 2;
                1
            }
            0xF9 => { // Load hl into stack pointer
                self.reg.sp = self.reg.get_hl();
                1
            }
            _ => {
                panic!("unknown op code 0x{:X}", code)
            }
        }
    }
}
