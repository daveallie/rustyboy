mod rom;

use std::fs::File;
use std::io::Read;
use mbc::rom::ROM;

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

    match cartridge_type {
        0x00 => Box::new(ROM { cart_data, ram_enabled: false }),
        _ => panic!("Unknown cartridge type: 0x{:X}", cartridge_type),
    }
}

fn load_cart(cart_path: &str, buffer: &mut Vec<u8>) {
    let mut file = match File::open(cart_path) {
        Ok(f) => f,
        Err(e) => panic!("Failed to open file from {}: {}", cart_path, e),
    };

    match file.read_to_end(buffer) {
        Ok(_) => println!("ROM loaded from {}", &cart_path),
        Err(e) => panic!("Failed to read file from {}: {}", cart_path, e),
    }
}

pub trait MBC : Send {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, value: u8);
}
