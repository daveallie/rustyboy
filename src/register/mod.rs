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

enum Flags {
    Z = 0b10000000,
    N = 0b01000000,
    H = 0b00100000,
    C = 0b00010000,
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

    pub fn get_bc(&mut self) -> u16 {
        (self.b as u16) << 8 | (self.c as u16)
    }

    pub fn get_flag(&mut self, flag: Flags) -> bool {
        let flag_byte = flag as u8;
        self.f & flag_byte > 0
    }

    fn set_flag(&mut self, flag: Flags, flagged: bool) {
        let flag_byte = flag as u8;
        match flagged {
            true => self.f |= flag_byte,
            false => self.f = self.f & !flag_byte & 0xF0,
        }
    }
}
