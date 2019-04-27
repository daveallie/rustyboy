mod alu;

#[derive(Copy, Clone)]
pub struct Registers {
    // 8 bit registers
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // Flags register
    // Z - 7th bit - zero flag
    // N - 6th bit - subtract flag
    // H - 5th bit - half-carry flag
    // C - 4th bit - carry flag
    f: u8,

    pub pc: u16, // Program counter
    pub sp: u16, // Stack pointer
}

// http://www.z80.info/z80sflag.htm
pub enum Flags {
    Z = 0b1000_0000,
    N = 0b0100_0000,
    H = 0b0010_0000,
    C = 0b0001_0000,
}

impl Registers {
    pub fn new() -> Self {
        // Set register values as expected after boot sequence
        Self {
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            f: 0xB0, // 10110000 -> ZHC flags
            pc: 0x0100,
            sp: 0xFFFE, // Initialise the stack pointer
        }
    }

    pub fn set_af(&mut self, value: u16) {
        let first_byte = (value >> 8) as u8;
        let second_byte = (value & 0x00FF) as u8;

        self.a = first_byte;
        self.f = second_byte & 0xF0; // Don't care about the last nibble
    }

    pub fn get_af(&self) -> u16 {
        self.get_unioned_address(self.a, self.f)
    }

    pub fn set_bc(&mut self, value: u16) {
        let first_byte = (value >> 8) as u8;
        let second_byte = (value & 0x00FF) as u8;

        self.b = first_byte;
        self.c = second_byte;
    }

    pub fn get_bc(&self) -> u16 {
        self.get_unioned_address(self.b, self.c)
    }

    pub fn set_de(&mut self, value: u16) {
        let first_byte = (value >> 8) as u8;
        let second_byte = (value & 0x00FF) as u8;

        self.d = first_byte;
        self.e = second_byte;
    }

    pub fn get_de(&self) -> u16 {
        self.get_unioned_address(self.d, self.e)
    }

    pub fn set_hl(&mut self, value: u16) {
        let first_byte = (value >> 8) as u8;
        let second_byte = (value & 0x00FF) as u8;

        self.h = first_byte;
        self.l = second_byte;
    }

    pub fn get_hl(&self) -> u16 {
        self.get_unioned_address(self.h, self.l)
    }

    pub fn get_hl_and_inc(&mut self) -> u16 {
        let result = self.get_hl();
        self.set_hl(result.wrapping_add(1));
        result
    }

    pub fn get_hl_and_dec(&mut self) -> u16 {
        let result = self.get_hl();
        self.set_hl(result.wrapping_sub(1));
        result
    }

    pub fn get_flag(&self, flag: Flags) -> bool {
        let flag_byte = flag as u8;
        self.f & flag_byte > 0
    }

    fn get_unioned_address(&self, addr1: u8, addr2: u8) -> u16 {
        u16::from(addr1) << 8 | u16::from(addr2)
    }

    fn set_flag(&mut self, flag: Flags, flagged: bool) {
        let flag_byte = flag as u8;
        if flagged {
            self.f |= flag_byte;
        } else {
            self.f = self.f & !flag_byte & 0xF0;
        }
    }
}
