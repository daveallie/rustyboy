impl ::cpu::CPU {
    fn call(&mut self) -> u32 {
        let code = self.get_byte();

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
                let value = self.reg.get_bc() + 1;
                self.reg.set_bc(value);
                1
            }
//            0x04 => { // inc b
//                // TODO: Implement
//            }
//            0x05 => { // dec b
//                // TODO: Implement
//            }
            0x06 => { // load byte into b
                self.reg.b = self.get_byte();
                2
            }
            _ => {
                panic!("unknown op code {:?}", code)
            }
        }
    }
}
