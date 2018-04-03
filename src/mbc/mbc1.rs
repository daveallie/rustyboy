use mbc::{self, MBC};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

// http://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers#MBC1_.28max_2MByte_ROM_and.2For_32KByte_RAM.29

pub struct MBC1 {
    save_path: String,
    cart_data: Vec<u8>,
    ram: Vec<u8>,
    ram_available: bool,
    ram_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    rom_banking_mode: bool,
    battery: bool,
}

impl MBC1 {
    pub fn new(cart_path: &str, cart_data: Vec<u8>, ram_available: bool, ram_size: usize, battery: bool) -> Self {
        let ram = if ram_available {
            vec![0; ram_size]
        } else {
            vec![]
        };

        let mut res = Self {
            save_path: mbc::build_save_path(cart_path),
            cart_data,
            ram,
            ram_available,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rom_banking_mode: true,
            battery,
        };

        res.load_ram();
        res
    }

    pub fn without_ram(cart_path: &str, cart_data: Vec<u8>) -> Self {
        Self::new(cart_path, cart_data, false, 0, false)
    }

    pub fn with_ram(cart_path: &str, cart_data: Vec<u8>, ram_size: usize) -> Self {
        Self::new(cart_path, cart_data, true, ram_size, false)
    }

    pub fn with_ram_and_battery(cart_path: &str, cart_data: Vec<u8>, ram_size: usize) -> Self {
        Self::new(cart_path, cart_data, true, ram_size, true)
    }

    fn adjusted_rom_addr(&self, addr: u16) -> usize {
        if addr < 0x4000 {
            addr as usize
        } else {
            addr as usize + (self.rom_bank as usize - 1) * 0x4000
        }
    }

    fn adjusted_ram_addr(&self, addr: u16) -> usize {
        (addr as usize & 0x1FFF) + (self.ram_bank as usize * 0x1FFF)
    }

    fn load_ram(&mut self) {
        let path = Path::new(&self.save_path);
        if !self.battery || !self.ram_available || !path.exists() {
            return;
        }

        let mut file = File::open(path).expect("Failed to load save data!");
        let mut new_ram: Vec<u8> = Vec::with_capacity(self.ram.len());
        file.read_to_end(&mut new_ram).expect("Failed to read ram!");
        new_ram.resize(self.ram.len(), 0);
        self.ram = new_ram;
    }
}

impl Drop for MBC1 {
    fn drop(&mut self) {
        if !self.battery || !self.ram_available {
            return;
        }

        // Don't bother handling errors here
        let _ = File::create(&self.save_path).and_then(|mut file| file.write_all(self.ram.as_slice()));
    }
}

impl MBC for MBC1 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x7FFF => self.cart_data[self.adjusted_rom_addr(addr)],
            0xA000...0xBFFF => {
                if !self.ram_enabled {
                    panic!("Attempting to read external ram, which isn't enabled!");
                }
                self.ram[self.adjusted_ram_addr(addr)]
            }
            _ => unreachable!("Tried to read non-existent mbc address"),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000...0x1FFF => {
                if !self.ram_available {
                    panic!("Attempting to set external ram enabled, when not available!");
                }
                self.ram_enabled = value & 0x0F == 0x0A;
            }
            0x2000...0x3FFF => {
                let rom_bank = value & 0x1F;
                self.rom_bank = match rom_bank {
                    0x00 | 0x20 | 0x40 | 0x60 => rom_bank + 1,
                    _ => rom_bank,
                };
            }
            0x4000...0x5FFF => {
                let bits = value & 0x03;
                if self.rom_banking_mode {
                    self.rom_bank = (self.rom_bank & 0x1F) | (bits << 5);
                } else {
                    self.ram_bank = bits;
                }
            }
            0x6000...0x7FFF => self.rom_banking_mode = value == 0,
            0xA000...0xBFFF => {
                if !self.ram_enabled {
                    panic!("Attempting to write external ram, which isn't enabled!");
                }
                let adj_addr = self.adjusted_ram_addr(addr);
                self.ram[adj_addr] = value;
            }
            _ => unreachable!("Tried to write non-existent mbc address"),
        }
    }
}
