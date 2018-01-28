impl ::cpu::CPU {
    pub fn step(&mut self) -> u32 {
        let code = self.get_byte();
        let read_regs = self.reg;

        println!("instr: 0x{:X} -- opcode: 0x{:X}", self.reg.pc, code);

        // http://clrhome.org/table/
        match code {
            0x00 => { // nop
                1
            }
            0x01 => { // load word into bc
                let value = self.get_word();
                self.reg.set_bc(value);
                3
            }
//            0x02 => { // write a into location pointed by bc
//                // TODO: Implement
//            }
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
            0x2C => { // inc l
                self.reg.l = self.reg.alu_inc(read_regs.l);
                1
            }
            0xC3 => { // jump to location point by word
                self.reg.pc = self.get_word();
                3
            }
            _ => {
                panic!("unknown op code 0x{:X}", code)
            }
        }
    }
}
