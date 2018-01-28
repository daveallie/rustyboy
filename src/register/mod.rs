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
    S  = 0b10000000,
    Z  = 0b01000000,
    F5 = 0b00100000,
    H  = 0b00010000,
    F3 = 0b00001000,
    PV = 0b00000100,
    N  = 0b00000010,
    C  = 0b00000001,
}

impl Registers {
    pub fn new() -> Registers {
        // Set register values as expected after boot sequence
        Registers {
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

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;     // first byte
        self.c = (value & 0x00FF) as u8; // second byte
    }

    pub fn get_bc(&self) -> u16 {
        self.get_unioned_address(self.b, self.c)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;     // first byte
        self.l = (value & 0x00FF) as u8; // second byte
    }

    pub fn get_hl(&self) -> u16 {
        self.get_unioned_address(self.h, self.l)
    }

    pub fn get_flag(&mut self, flag: Flags) -> bool {
        let flag_byte = flag as u8;
        self.f & flag_byte > 0
    }

    fn get_unioned_address(&self, addr1: u8, addr2: u8) -> u16 {
        (addr1 as u16) << 8 | (addr2 as u16)
    }

    fn set_flag(&mut self, flag: Flags, flagged: bool) {
        let flag_byte = flag as u8;
        match flagged {
            true => self.f |= flag_byte,
            false => self.f = self.f & !flag_byte & 0xF0,
        }
    }
}
