use mbc::MBC;
use std::time::{UNIX_EPOCH, SystemTime, Duration};

// http://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers#MBC1_.28max_2MByte_ROM_and.2For_32KByte_RAM.29

pub struct MBC3 {
    cart_data: Vec<u8>,
    ram: Vec<u8>,
    ram_available: bool,
    ram_and_timer_enabled: bool,
    rom_bank: u8,
    ram_bank: u8,
    rtc_register: [u8; 5],
    primed_to_latch_rtc: bool,
    rtc_seconds_since_epoch: u64,
}

impl MBC3 {
    pub fn new(cart_data: Vec<u8>, ram_available: bool, ram_size: usize) -> Self {
        let ram = if ram_available {
            vec![0; ram_size]
        } else {
            vec![]
        };

        Self {
            cart_data,
            ram,
            ram_available,
            ram_and_timer_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            rtc_register: [0; 5],
            primed_to_latch_rtc: false,
            rtc_seconds_since_epoch: 0,
        }
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
        let seconds_to_now = SystemTime::now().duration_since(rtc_time)
            .map(|duration| duration.as_secs())
            .unwrap_or(0);

        if self.rtc_register[4] & 0x40 > 0 {
            // disabled
            return
        }

        let days = seconds_to_now / 3600 / 24;
        self.rtc_register[0] = (seconds_to_now % 60) as u8;
        self.rtc_register[1] = ((seconds_to_now / 60) % 60) as u8;
        self.rtc_register[2] = ((seconds_to_now / 3600) % 24) as u8;
        self.rtc_register[3] = (days & 0xFF) as u8;
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
        let seconds_to_now = u64::from(self.rtc_register[0])
            + u64::from(self.rtc_register[1]) * 60
            + u64::from(self.rtc_register[2]) * 3600
            + u64::from(self.rtc_register[3]) * 3600 * 24
            + if self.rtc_register[4] & 0x01 > 0 { 0x0100 * 3600 * 24 } else { 0 };

        self.rtc_seconds_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() - seconds_to_now)
            .unwrap_or(0);
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
            },
            _ => unreachable!("Tried to read non-existent mbc address"),
        }
    }

    fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000...0x1FFF => {
                self.ram_and_timer_enabled = value & 0x0F == 0x0A;
            },
            0x2000...0x3FFF => {
                let rom_bank = value & 0x7F;
                self.rom_bank = if rom_bank == 0 {
                    rom_bank + 1
                } else {
                    rom_bank
                };
            },
            0x4000...0x5FFF => {
                match value {
                    0x00...0x03 | 0x08...0x0C => self.ram_bank = value,
                    _ => panic!("Writing unknown ram bank number!"),
                }
            },
            0x6000...0x7FFF => {
                if self.primed_to_latch_rtc && value == 0x01 {
                    self.latch_rtc();
                }

                self.primed_to_latch_rtc = value != 0;
            },
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
            },
            _ => unreachable!("Tried to write non-existent mbc address"),
        }
    }
}
