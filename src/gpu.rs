use std::sync::mpsc;
use screen;
use rand::{self, Rng};

const VIDEO_RAM_SIZE: usize = 0x2000;

pub struct GPU {
    video_ram: [u8; VIDEO_RAM_SIZE],
    bg_palette: u8,
    obj_palette_0: u8,
    obj_palette_1: u8,
    oam: [u8; 160], // Sprite attribute table
    lcd_control: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    ly: u8,
    render_clock: u32,
    screen_data_sender: mpsc::SyncSender<Vec<u8>>,
}

impl GPU {
    pub fn new(screen_data_sender: mpsc::SyncSender<Vec<u8>>) -> Self {
        Self {
            video_ram: [0_u8; VIDEO_RAM_SIZE],
            bg_palette: 0,
            obj_palette_0: 0,
            obj_palette_1: 0,
            oam: [0_u8; 160],
            lcd_control: 0x91,
            stat: 0,
            scy: 0,
            scx: 0,
            ly: 0,
            render_clock: 0,
            screen_data_sender,
        }
    }

    pub fn run_cycle(&mut self, cycles: u8) {
        if !self.is_lcd_on() {
            return
        }

        let mut cycles_to_process = cycles;

        while cycles_to_process > 0 {
            cycles_to_process -= self.process_cycles(u32::from(cycles_to_process));
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        self.oam[(addr & 0xFF) as usize]
    }

    pub fn write_oam(&mut self, addr: u16, value: u8) {
        self.oam[(addr & 0xFF) as usize] = value;
    }

    pub fn read_video_ram(&self, addr: u16) -> u8 {
        self.video_ram[(addr & 0x1FFF) as usize]
    }

    pub fn write_video_ram(&mut self, addr: u16, value: u8) {
        self.video_ram[(addr & 0x1FFF) as usize] = value;
    }

    pub fn read_control(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcd_control,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF47 => self.bg_palette,
            0xFF48 => self.obj_palette_0,
            0xFF49 => self.obj_palette_1,
            _ => panic!("Unknown GPU control read operation: 0x{:X}", addr),
        }
    }

    pub fn write_control(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF40 => self.lcd_control = value,
            0xFF41 => self.stat = value,
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => (), // read only
            0xFF47 => self.bg_palette = value,
            0xFF48 => self.obj_palette_0 = value,
            0xFF49 => self.obj_palette_1 = value,
            _ => panic!("Unknown GPU control write operation: 0x{:X}", addr),
        }
    }

    fn is_lcd_on(&self) -> bool {
        self.lcd_control & 0x80 > 0
    }

    fn process_cycles(&mut self, cycles: u32) -> u8 {
        if self.render_clock + cycles >= 114 {
            let used_cycles = (self.render_clock + cycles - 114) as u8;
            self.render_clock = 0;
            self.increment_line();
            used_cycles
        } else {
            self.render_clock += cycles;
            cycles as u8
        }
    }

    fn increment_line(&mut self) {
        self.ly = (self.ly + 1) % 154;
        if self.ly >= 144 { // V-Blank
            self.render_random();
        }
    }

    pub fn render_random(&self) {
        let mut rng = rand::thread_rng();
        let datavec: Vec<u8> = (0..(screen::Screen::WIDTH * screen::Screen::HEIGHT)).flat_map(|_i| {
            let col = match rng.gen_range(0, 4) {
                0 => 255,
                1 => 192,
                2 => 96,
                _ => 0,
            };

            vec![col, col, col]
        }).collect();

        match self.screen_data_sender.send(datavec) {
            Ok(_) => (),
            Err(e) => println!("Failed to send screen data: {}", e),
        };
    }
}
