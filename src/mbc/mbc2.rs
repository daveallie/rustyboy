use mbc::MBC;

// http://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers#MBC2_.28max_256KByte_ROM_and_512x4_bits_RAM.29

pub struct MBC2 {
    cart_data: Vec<u8>,
    ram: [u8; 512],
    ram_enabled: bool,
    rom_bank: u8,
}

impl MBC2 {
    pub fn new(cart_data: Vec<u8>) -> Self {
        Self {
            cart_data,
            ram: [0_u8; 512],
            ram_enabled: false,
            rom_bank: 1,
        }
    }

    fn adjusted_rom_addr(&self, addr: u16) -> usize {
        if addr < 0x4000 {
            addr as usize
        } else {
            (addr + 0x4000 * u16::from(self.rom_bank - 1)) as usize
        }
    }
}

impl MBC for MBC2 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x7FFF => self.cart_data[self.adjusted_rom_addr(addr)],
            0xA000...0xA1FF => {
                if !self.ram_enabled {
                    panic!("Attempting to read external ram, which isn't enabled!");
                }
                self.ram[(addr & 0x1FF) as usize]
            },
            _ => unreachable!("Tried to read non-existent mbc address"),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000...0x1FFF => {
                if addr & 0x0100 > 1 {
                    self.ram_enabled = value & 0x0F == 0x0A;
                }
            },
            0x2000...0x3FFF => {
                if addr & 0x0100 > 1 {
                    let rom_bank = value & 0x0F;
                    self.rom_bank = if rom_bank == 0 {
                        rom_bank + 1
                    } else {
                        rom_bank
                    };
                }
            },
            0xA000...0xA1FF => {
                if !self.ram_enabled {
                    panic!("Attempting to write external ram, which isn't enabled!");
                }
                self.ram[(addr & 0x1FF) as usize] = value & 0x0F;
            },
            _ => unreachable!("Tried to write non-existent mbc address"),
        }
    }
}
