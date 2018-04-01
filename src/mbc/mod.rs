mod rom;
mod mbc1;
mod mbc2;
mod mbc3;

use std::fs::File;
use std::io::Read;
use mbc::rom::ROM;
use mbc::mbc1::MBC1;
use mbc::mbc2::MBC2;
use mbc::mbc3::MBC3;
use std::path::Path;

// http://gbdev.gg8.se/wiki/articles/The_Cartridge_Header#0147_-_Cartridge_Type
/*
 00h  ROM ONLY                 19h  MBC5
 01h  MBC1                     1Ah  MBC5+RAM
 02h  MBC1+RAM                 1Bh  MBC5+RAM+BATTERY
 03h  MBC1+RAM+BATTERY         1Ch  MBC5+RUMBLE
 05h  MBC2                     1Dh  MBC5+RUMBLE+RAM
 06h  MBC2+BATTERY             1Eh  MBC5+RUMBLE+RAM+BATTERY
 08h  ROM+RAM                  20h  MBC6
 09h  ROM+RAM+BATTERY          22h  MBC7+SENSOR+RUMBLE+RAM+BATTERY
 0Bh  MMM01
 0Ch  MMM01+RAM
 0Dh  MMM01+RAM+BATTERY
 0Fh  MBC3+TIMER+BATTERY
 10h  MBC3+TIMER+RAM+BATTERY   FCh  POCKET CAMERA
 11h  MBC3                     FDh  BANDAI TAMA5
 12h  MBC3+RAM                 FEh  HuC3
 13h  MBC3+RAM+BATTERY         FFh  HuC1+RAM+BATTERY
*/

pub fn new(cart_path: &str) -> Box<MBC> {
    let mut cart_data: Vec<u8> = Vec::new();
    load_cart(cart_path, &mut cart_data);
    let cartridge_type = cart_data[0x147];

    let ram_size: usize = match cart_data[0x149] {
        0x00 => 0,
        0x01 => 0x800, // 2_048
        0x02 => 0x2000, // 8_192
        0x03 => 0x8000, // 32_768
        0x04 => 0x2_0000, // 131_072
        0x05 => 0x1_0000, // 65_536
        _ => unreachable!("Unknown cart ram size!"),
    };

    match cartridge_type {
        0x00 => Box::new(ROM::new(cart_path, cart_data)),
        0x01 => Box::new(MBC1::without_ram(cart_path, cart_data)),
        0x02 => Box::new(MBC1::with_ram(cart_path, cart_data, ram_size)),
        0x03 => Box::new(MBC1::with_ram_and_battery(cart_path, cart_data, ram_size)),
        0x05 => Box::new(MBC2::without_battery(cart_path, cart_data)),
        0x06 => Box::new(MBC2::with_battery(cart_path, cart_data)),
        0x11 => Box::new(MBC3::without_ram(cart_path, cart_data)),
        0x12 => Box::new(MBC3::with_ram(cart_path, cart_data, ram_size)),
        0x13 => Box::new(MBC3::with_ram_and_battery(cart_path, cart_data, ram_size)),
        _ => panic!("Unknown cartridge type: 0x{:X}", cartridge_type),
    }
}

pub fn build_save_path(cart_path: &str) -> String {
    String::from(Path::new(cart_path).with_extension("gbsave-rustyboy").to_string_lossy())
}

fn load_cart(cart_path: &str, buffer: &mut Vec<u8>) {
    match File::open(cart_path).and_then(|mut file| file.read_to_end(buffer)) {
        Ok(_) => println!("ROM loaded from {}", &cart_path),
        Err(e) => panic!("Failed to read file from {}: {}", cart_path, e),
    };
}

pub trait MBC : Send {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, value: u8);
}
