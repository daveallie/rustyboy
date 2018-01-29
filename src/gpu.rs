pub struct GPU {
    OAM: [u8; 160], // Sprite attribute table
    stat: u8,
    scy: u8,
    scx: u8,
}

impl GPU {
    pub fn new() -> GPU {
        GPU {
            OAM: [0u8; 160],
            stat: 0,
            scy: 0,
            scx: 0,
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        self.OAM[(addr & 0xFF) as usize]
    }

    pub fn read_control(&self, addr: u16) -> u8 {
        match addr {
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            _ => panic!("Unknown GPU control read operation: 0x{:X}", addr),
        }
    }

    pub fn write_control(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF41 => self.stat = value,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            _ => panic!("Unknown GPU control write operation: 0x{:X}", addr),
        }
    }
}
