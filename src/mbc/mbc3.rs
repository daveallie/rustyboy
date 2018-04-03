use mbc::{self, MBC};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{UNIX_EPOCH, SystemTime, Duration};

// http://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers#MBC1_.28max_2MByte_ROM_and.2For_32KByte_RAM.29

pub struct MBC3 {
    save_path: String,
    cart_data: Vec<u8>,
    ram: Vec<u8>,
    ram_available: bool,
    ram_and_timer_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    rtc_register: [u8; 5],
    primed_to_latch_rtc: bool,
    rtc_seconds_since_epoch: u64,
    battery: bool,
}

impl MBC3 {
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
            ram_and_timer_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rtc_register: [0_u8; 5],
            primed_to_latch_rtc: false,
            rtc_seconds_since_epoch: 0,
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

    fn latch_rtc(&mut self) {
        let rtc_time = UNIX_EPOCH + Duration::from_secs(self.rtc_seconds_since_epoch);
        let seconds_to_now = SystemTime::now()
            .duration_since(rtc_time)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        if self.rtc_register[4] & 0x40 > 0 {
            // disabled
            return;
        }

        let days = seconds_to_now / 3600 / 24;
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        let seconds = (seconds_to_now % 60) as u8;
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        let minutes = ((seconds_to_now / 60) % 60) as u8;
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        let hours = ((seconds_to_now / 3600) % 24) as u8;
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        let trunc_days = (days & 0xFF) as u8;

        self.rtc_register[0] = seconds;
        self.rtc_register[1] = minutes;
        self.rtc_register[2] = hours;
        self.rtc_register[3] = trunc_days;
        self.rtc_register[4] &= 0xFE;

        if days & 0x0100 > 0 {
            self.rtc_register[4] |= 0x01;
        }
        if days >= 512 {
            self.rtc_register[4] |= 0x80;
            self.reset_rtc();
        }
    }

    fn reset_rtc(&mut self) {
        let seconds_to_now = u64::from(self.rtc_register[0]) + u64::from(self.rtc_register[1]) * 60 +
            u64::from(self.rtc_register[2]) * 3600 +
            u64::from(self.rtc_register[3]) * 3600 * 24 +
            if self.rtc_register[4] & 0x01 > 0 {
                0x0100 * 3600 * 24
            } else {
                0
            };

        self.rtc_seconds_since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() - seconds_to_now)
            .unwrap_or(0);
    }

    fn load_ram(&mut self) {
        let path = Path::new(&self.save_path);
        if !self.battery || !path.exists() {
            return;
        }

        let mut file = File::open(path).expect("Failed to load save data!");
        file.read_exact(&mut self.rtc_register).expect(
            "Failed to read rtc data!",
        );
        if self.ram_available {
            let mut new_ram: Vec<u8> = Vec::with_capacity(self.ram.len());
            file.read_to_end(&mut new_ram).expect("Failed to read ram!");
            new_ram.resize(self.ram.len(), 0);
            self.ram = new_ram;
        }
    }
}

impl Drop for MBC3 {
    fn drop(&mut self) {
        if !self.battery {
            return;
        }

        // Don't bother handling errors here
        if let Ok(mut file) = File::create(&self.save_path) {
            let _ = file.write_all(&self.rtc_register);
            let _ = file.write_all(self.ram.as_slice());
        }
    }
}

impl MBC for MBC3 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x7FFF => self.cart_data[self.adjusted_rom_addr(addr)],
            0xA000...0xBFFF => {
                if !self.ram_and_timer_enabled {
                    panic!("Attempting to read external ram/RTC, which isn't enabled!");
                }

                if self.ram_bank > 0x03 {
                    self.rtc_register[(self.ram_bank - 0x08) as usize]
                } else {
                    if !self.ram_available {
                        panic!("Attempting to read external ram, which isn't enabled!");
                    }
                    self.ram[self.adjusted_ram_addr(addr)]
                }
            }
            _ => unreachable!("Tried to read non-existent mbc address"),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000...0x1FFF => {
                self.ram_and_timer_enabled = value & 0x0F == 0x0A;
            }
            0x2000...0x3FFF => {
                let rom_bank = value & 0x7F;
                self.rom_bank = if rom_bank == 0 {
                    rom_bank + 1
                } else {
                    rom_bank
                };
            }
            0x4000...0x5FFF => {
                match value {
                    0x00...0x03 | 0x08...0x0C => self.ram_bank = value,
                    _ => panic!("Writing unknown ram bank number!"),
                }
            }
            0x6000...0x7FFF => {
                if self.primed_to_latch_rtc && value == 0x01 {
                    self.latch_rtc();
                }

                self.primed_to_latch_rtc = value != 0;
            }
            0xA000...0xBFFF => {
                if !self.ram_and_timer_enabled {
                    panic!("Attempting to write external ram/RTC, which isn't enabled!");
                }

                if self.ram_bank > 0x03 {
                    self.rtc_register[(self.ram_bank - 0x08) as usize] = value;
                    self.reset_rtc();
                } else {
                    if !self.ram_available {
                        panic!("Attempting to write external ram, which isn't enabled!");
                    }
                    let adj_addr = self.adjusted_ram_addr(addr);
                    self.ram[adj_addr] = value;
                }
            }
            _ => unreachable!("Tried to write non-existent mbc address"),
        }
    }
}
