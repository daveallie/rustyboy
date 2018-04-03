use mbc::MBC;

pub struct ROM {
    cart_data: Vec<u8>,
}

impl ROM {
    pub fn new(_cart_path: &str, cart_data: Vec<u8>) -> Self {
        Self { cart_data }
    }
}

impl MBC for ROM {
    fn read_byte(&self, addr: u16) -> u8 {
        if addr >= 0x8000 {
            panic!("Attempting to read for external ram, which doesn't exist!");
        }

        self.cart_data[addr as usize]
    }

    fn write_byte(&mut self, _addr: u16, _value: u8) {}
}
