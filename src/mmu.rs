use std::sync::mpsc;
use clock::Clock;
use gpu::GPU;
use input::{Input, Key};
use serial::Serial;
use sound::Sound;
use mbc::{self, MBC};

// Gameboy only needs 0x2000 working RAM
// In the future if CGB support is needed,
// this should be expanded to 0x8000 to support
// the switchable memory modules.
const WRAM_SIZE: usize = 0x2000;
const HRAM_SIZE: usize = 0x80;

pub struct MMU {
    mbc: Box<MBC>,
    wram: [u8; WRAM_SIZE], // Working RAM
    hram: [u8; HRAM_SIZE], // High RAM
    gpu: GPU,
    serial: Serial,
    clock: Clock,
    input: Input,
    sound: Sound,
    interrupt_flags: u8,
    interrupt_enabled: u8,
}

impl MMU {
    pub fn new(cart_path: &str, screen_data_sender: mpsc::SyncSender<Vec<u8>>, key_data_receiver: mpsc::Receiver<Key>) -> Self {
        Self {
            mbc: mbc::new(cart_path),
            wram: [0_u8; WRAM_SIZE],
            hram: [0_u8; HRAM_SIZE],
            gpu: GPU::new(screen_data_sender),
            serial: Serial::new(),
            clock: Clock::new(),
            input: Input::new(key_data_receiver),
            sound: Sound::new(),
            interrupt_flags: 0,
            interrupt_enabled: 0,
        }
    }

    pub fn run_cycle(&mut self, cpu_cycles: u8) {
        self.gpu.run_cycle(cpu_cycles);
        self.interrupt_flags |= self.gpu.interrupt;
        self.gpu.interrupt = 0;

        self.clock.run_cycle(cpu_cycles);
        self.interrupt_flags |= self.clock.interrupt;
        self.clock.interrupt = 0;

        self.input.run_cycle();
        self.interrupt_flags |= self.input.interrupt;
        self.input.interrupt = 0;
    }

    // http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x7FFF | 0xA000...0xBFFF => self.mbc.read_byte(addr), // ROM and cart RAM
            0x8000...0x9FFF => self.gpu.read_video_ram(addr), // Load from GPU
            0xC000...0xFDFF => self.wram[(addr & 0x1FFF) as usize], // Working RAM
            0xFE00...0xFE9F => self.gpu.read_oam(addr), // Graphics - sprite information
            0xFF00 => self.input.read(), // Input read
            0xFF01...0xFF02 => self.serial.read(addr), // Serial read
            0xFF04...0xFF07 => self.clock.read_byte(addr), // read Clock values
            0xFF0F => self.interrupt_flags, // Interrupt flags
            0xFF10...0xFF26 => self.sound.read_byte(addr), // Sound control
//            0xFF30...0xFF3F => 0, // Sound wave pattern RAM
            0xFF40...0xFF4B => self.gpu.read_control(addr),
//            0xFF4C...0xFF7F => panic!("MMU ERROR: Memory mapped I/O (read) (CGB only) not implemented"), // Memory mapped I/O CGB ONLY
            0xFF80...0xFFFE => self.hram[(addr & 0x7F) as usize], // High RAM
            0xFFFF => self.interrupt_enabled, // Interrupt enable
            _ => 0,
        }
    }

    pub fn read_word(&mut self, addr: u16) -> u16 {
        u16::from(self.read_byte(addr)) | (u16::from(self.read_byte(addr + 1)) << 8)
    }

    // http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000...0x7FFF | 0xA000...0xBFFF => self.mbc.write_byte(addr, value), // ROM and cart RAM
            0x8000...0x9FFF => self.gpu.write_video_ram(addr, value), // Write to GPU
            0xC000...0xFDFF => self.wram[(addr & 0x1FFF) as usize] = value, // Working RAM
            0xFE00...0xFE9F => self.gpu.write_oam(addr, value), // Graphics - sprite information
            0xFF00 => self.input.write(value), // Input write
            0xFF01...0xFF02 => self.serial.write(addr, value), // Serial write
            0xFF04...0xFF07 => self.clock.write_byte(addr, value), // write Clock values
            0xFF0F => self.interrupt_flags = value, // Interrupt flags
            0xFF10...0xFF26 => self.sound.write_byte(addr, value), // Sound control
//            0xFF30...0xFF3F => (), // Sound wave pattern RAM
            0xFF46 => self.dma_into_oam(value),
            0xFF40...0xFF45 | 0xFF47...0xFF4B => self.gpu.write_control(addr, value),
//            0xFF4C...0xFF7F => panic!("MMU ERROR: Memory mapped I/O (write) (CGB only) not implemented. Addr: 0x{:X}", addr), // Memory mapped I/O CGB ONLY
            0xFF80...0xFFFE => self.hram[(addr & 0x7F) as usize] = value, // High RAM
            0xFFFF => self.interrupt_enabled = value, // Interrupt enable
            _ => (),
        }
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        self.write_byte(addr, (value & 0xFF) as u8);
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        self.write_byte(addr + 1, (value >> 8) as u8);
    }

    pub fn get_triggered_interrupts(&self) -> u8 {
        self.interrupt_flags & self.interrupt_enabled
    }

    pub fn reset_interrupt(&mut self, flag: u8) {
        self.interrupt_flags &= !flag;
    }

    fn dma_into_oam(&mut self, dma_start: u8) {
        // DMA start can be addressed as 0x0000, 0x0100, 0x0200, etc
        let actual_dma_start = u16::from(dma_start) << 8; // turns 0x01 to 0x0100
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        for i in 0..(GPU::OAM_SIZE as u16) {
            let value = self.read_byte(actual_dma_start + i);
            self.gpu.write_oam(i, value);
        }
    }
}
