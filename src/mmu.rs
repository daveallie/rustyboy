use std::fs::File;
use std::io::Read;

// Gameboy only needs 0x2000 working RAM
// In the future if CGB support is needed,
// this should be expanded to 0x8000 to support
// the switchable memory modules.
const WRAM_SIZE: usize = 0x2000;
const ZRAM_SIZE: usize = 0x80;

pub struct MMU {
    rom: Vec<u8>,
    wram: [u8; WRAM_SIZE], // Working RAM
    zram: [u8; ZRAM_SIZE], // Zero page RAM
}

impl MMU {
    pub fn new(cart_path: &str) -> MMU {
        let mut cart_data: Vec<u8> = Vec::new();
        MMU::load_cart(&cart_path, &mut cart_data);

        MMU {
            rom: cart_data,
            wram: [0u8; WRAM_SIZE],
            zram: [0u8; ZRAM_SIZE],
        }
    }

    pub fn read_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000...0x7FFF => self.rom[addr as usize], // ROM
            0x8000...0x9FFF => panic!("MMU ERROR: Load from GPU not implemented"), // Load from GPU
            0xA000...0xBFFF => panic!("MMU ERROR: Load from cart RAM not implemented"), // Load from cartridge RAM
            0xC000...0xFDFF => self.wram[(addr & 0x1FFF) as usize], // Working RAM
            0xFE00...0xFE9F => panic!("MMU ERROR: Load graphics sprite information not implemented"), // Graphics - sprite information
            0xFF00...0xFF7F => panic!("MMU ERROR: Memory mapped I/O not implemented"), // Memory mapped I/O
            0xFF80...0xFFFF => self.zram[(addr & 0x7F) as usize], // Zero page RAM
            _ => 0,
        }
    }

    pub fn read_word(&mut self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) | ((self.read_byte(addr + 1) as u16) << 8)
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000...0x7FFF => self.rom[addr as usize] = value, // ROM
            0x8000...0x9FFF => panic!("MMU ERROR: Write to GPU not implemented"), // Write to GPU
            0xA000...0xBFFF => panic!("MMU ERROR: Write to cart RAM not implemented"), // Load from cartridge RAM
            0xC000...0xFDFF => self.wram[(addr & 0x1FFF) as usize] = value, // Working RAM
            0xFE00...0xFE9F => panic!("MMU ERROR: Write graphics sprite information not implemented"), // Graphics - sprite information
            0xFF00...0xFF7F => panic!("MMU ERROR: Memory mapped I/O not implemented"), // Memory mapped I/O
            0xFF80...0xFFFF => self.zram[(addr & 0x7F) as usize] = value, // Zero page RAM
            _ => (),
        }
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        panic!("write_word not yet implemented")
    }

    fn load_cart(cart_path: &str, buffer: &mut Vec<u8>) {
        let mut file = File::open(cart_path).unwrap();
        file.read_to_end(buffer).unwrap();
        println!("ROM loaded from {}", &cart_path);
    }
}
