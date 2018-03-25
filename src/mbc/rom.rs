use mbc::MBC;

pub struct ROM {
    pub cart_data: Vec<u8>,
    pub ram_enabled: bool,
}

impl MBC for ROM {
    fn read_byte(&self, addr: u16) -> u8 {
        self.cart_data[addr as usize]
    }

    fn write_byte(&mut self, _addr: u16, _value: u8) {
        panic!("ROM MBC Type is read only!");
    }
}
