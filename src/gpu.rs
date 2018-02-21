use std::sync::mpsc;
use screen::Screen;

const VIDEO_RAM_SIZE: usize = 0x2000;

pub struct GPU {
    next_screen_buffer: Vec<u8>,
    video_ram: [u8; VIDEO_RAM_SIZE],
    bg_palette: u8,
    bg_palette_map: [u8; 4],
    obj_palette_0: u8,
    obj_palette_0_map: [u8; 4],
    obj_palette_1: u8,
    obj_palette_1_map: [u8; 4],
    oam: [u8; GPU::OAM_SIZE], // Sprite attribute table
    lcd_control: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    win_y: u8,
    win_x: u8,
    ly: u8,
    render_clock: u32,
    screen_data_sender: mpsc::Sender<Vec<u8>>,
    pub interrupt: u8,
}

impl GPU {
    pub const OAM_SIZE: usize = 0xA0;

    pub fn new(screen_data_sender: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            next_screen_buffer: vec![0_u8; (3 * Screen::WIDTH * Screen::HEIGHT) as usize],
            video_ram: [0_u8; VIDEO_RAM_SIZE],
            bg_palette: 0,
            bg_palette_map: build_palette_map(0),
            obj_palette_0: 0,
            obj_palette_0_map: build_palette_map(0),
            obj_palette_1: 0,
            obj_palette_1_map: build_palette_map(0),
            oam: [0_u8; 160],
            lcd_control: 0x91,
            stat: 0,
            scy: 0,
            scx: 0,
            win_y: 0,
            win_x: 0,
            ly: 0,
            render_clock: 0,
            screen_data_sender,
            interrupt: 0,
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

    pub fn read_control(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcd_control,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF46 => unreachable!("DMA Address is write only"),
            0xFF47 => self.bg_palette,
            0xFF48 => self.obj_palette_0,
            0xFF49 => self.obj_palette_1,
            0xFF4A => self.win_y,
            0xFF4B => self.win_x,
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
            0xFF46 => unreachable!("DMA write handled in mmu.rs"),
            0xFF47 => {
                self.bg_palette = value;
                self.bg_palette_map = build_palette_map(value);
            },
            0xFF48 => {
                self.obj_palette_0 = value;
                self.obj_palette_0_map = build_palette_map(value);
            },
            0xFF49 => {
                self.obj_palette_1 = value;
                self.obj_palette_1_map = build_palette_map(value);
            },
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,
            _ => panic!("Unknown GPU control write operation: 0x{:X}", addr),
        }
    }

    fn process_cycles(&mut self, cycles: u32) -> u8 {
        if self.render_clock + cycles >= 114 {
            #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
            let used_cycles = (self.render_clock + cycles - 114) as u8;
            self.render_clock = 0;
            self.increment_line();
            self.render_background();
            self.render_sprites();
            used_cycles
        } else {
            self.render_clock += cycles;
            #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
            let cycles_u8 = cycles as u8;
            cycles_u8
        }
    }

    fn is_window_bg_on(&self) -> bool {
        self.lcd_control & 0x01 > 0
    }

    fn is_sprite_display_on(&self) -> bool {
        self.lcd_control & 0x02 > 0
    }

    fn is_sprite_8_by_16(&self) -> bool {
        self.lcd_control & 0x04 > 0
    }

    fn is_lcd_on(&self) -> bool {
        self.lcd_control & 0x80 > 0
    }

    fn increment_line(&mut self) {
        self.ly = (self.ly + 1) % 154;
        if self.ly >= 144 { // V-Blank
            if self.ly == 144 {
                self.interrupt |= 0x01; // Mark V-Blank interrupt
            }
            self.render_screen();
        }
    }

    fn render_background(&mut self) {
        if !self.is_window_bg_on() || self.ly >= 144 { // bg and window display
            return
        }

        let bg_tile_map_addr = self.bg_tile_map_addr();
        let bg_tile_data_addr = self.bg_tile_data_addr();
        let bgy = self.scy.wrapping_add(self.ly);
        let bgy_tile = (u16::from(bgy) & 0xFF) >> 3;
        let bgy_pixel_in_tile = u16::from(bgy) & 0x07;

        for x in 0 .. Screen::WIDTH {
            let bgx = u32::from(self.scx) + x;
            #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
            let bgx_tile = ((bgx & 0xFF) >> 3) as u16;
            #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
            let bgx_pixel_in_tile = 7 - (bgx & 0x07) as u8;

            let tile_number_addr = bg_tile_map_addr + bgy_tile * 32 + bgx_tile;
            let tile_number: u8 = self.read_video_ram(tile_number_addr);

            let tile_addr_offset = if bg_tile_data_addr == 0x8000 {
                // regular reading
                u16::from(tile_number) * 16
            } else {
                // reading with offset
                #[cfg_attr(feature="clippy", allow(cast_possible_truncation, cast_sign_loss, cast_possible_wrap))]
                let adjusted_tile_number = (i16::from(tile_number as i8) + 128) as u16;
                adjusted_tile_number * 16
            };
            let tile_addr = bg_tile_data_addr + tile_addr_offset;

            let tile_line_addr = tile_addr + bgy_pixel_in_tile * 2;
            let (tile_line_data_1, tile_line_data_2) = (self.read_video_ram(tile_line_addr), self.read_video_ram(tile_line_addr + 1));
            let pixel_in_line_mask = 1 << bgx_pixel_in_tile;
            let pixel_data_1: u8 = if tile_line_data_1 & pixel_in_line_mask > 0 {
                0b01
            } else {
                0b00
            };
            let pixel_data_2: u8 = if tile_line_data_2 & pixel_in_line_mask > 0 {
                0b10
            } else {
                0b00
            };

            let palette_color_id = pixel_data_1 | pixel_data_2;
            let color: u8 = self.bg_palette_map[palette_color_id as usize];

            self.set_pixel_color_next_screen_buffer(x, color);
        }
    }

    fn render_sprites(&mut self) {
        if !self.is_sprite_display_on() || self.ly >= 144 {
            return;
        }

        let sprite_height = if self.is_sprite_8_by_16() {
            16
        } else {
            8
        };

        for sprite_id in 0..40_u16 {
            let sprite_attr_addr = sprite_id * 4;
            let sprite_y = self.read_oam(sprite_attr_addr).wrapping_sub(16);
            let sprite_x = self.read_oam(sprite_attr_addr + 1).wrapping_sub(0x08);
            let sprite_location = self.read_oam(sprite_attr_addr + 2);
            let sprite_attributes = self.read_oam(sprite_attr_addr + 3);

            let sprite_under_bg = sprite_attributes & 0x80 > 0;
            let y_flip = sprite_attributes & 0x40 > 0;
            let x_flip = sprite_attributes & 0x20 > 0;
            let use_palette_0 = sprite_attributes & 0x10 == 0;

            if (self.ly < sprite_y) || (self.ly >= (sprite_y + sprite_height)) {
                continue;
            }

            let y_pixel_in_tile = if y_flip {
                sprite_y + sprite_height - self.ly
            } else {
                self.ly - sprite_y
            } as u16;

            let sprite_addr = 0x8000_u16 + (sprite_location as u16 * 16) + y_pixel_in_tile * 2;
            let (sprite_data_1, sprite_data_2) = (self.read_video_ram(sprite_addr), self.read_video_ram(sprite_addr + 1));

            for x_pixel_in_tile in 0..8_u8 {
                let pixel_in_line_mask = if x_flip {
                    1 << x_pixel_in_tile
                } else {
                    1 << (7 - x_pixel_in_tile)
                };

                let pixel_data_1: u8 = if sprite_data_1 & pixel_in_line_mask > 0 {
                    0b01
                } else {
                    0b00
                };
                let pixel_data_2: u8 = if sprite_data_2 & pixel_in_line_mask > 0 {
                    0b10
                } else {
                    0b00
                };

                let palette_color_id = pixel_data_1 | pixel_data_2;
                let color: u8 = if use_palette_0 {
                    self.obj_palette_0_map[palette_color_id as usize]
                } else {
                    self.obj_palette_1_map[palette_color_id as usize]
                };

                let x = sprite_x + x_pixel_in_tile;

                if sprite_under_bg && color == 255 {
                    continue;
                }
                self.set_pixel_color_next_screen_buffer(u32::from(x), color);
            }
        }
    }

    fn bg_tile_data_addr(&self) -> u16 {
        if self.lcd_control & 0x10 > 0 {
            0x8000
        } else {
            0x8800
        }
    }

    fn bg_tile_map_addr(&self) -> u16 {
        if self.lcd_control & 0x08 > 0 {
            0x9C00
        } else {
            0x9800
        }
    }

    fn set_pixel_color_next_screen_buffer(&mut self, x_pixel: u32, color: u8) {
        let base_addr = (u32::from(self.ly) * Screen::WIDTH + x_pixel) as usize * 3;
        self.next_screen_buffer[base_addr] = color;
        self.next_screen_buffer[base_addr + 1] = color;
        self.next_screen_buffer[base_addr + 2] = color;
    }

    fn render_screen(&self) {
        match self.screen_data_sender.send(self.next_screen_buffer.to_vec()) {
            Ok(_) => (),
            Err(e) => println!("Failed to send screen data: {}", e),
        };
    }
}

fn build_palette_map(palette_layout: u8) -> [u8; 4] {
    [
        color_from_dot_data(palette_layout >> 6),
        color_from_dot_data((palette_layout >> 4) & 0b11),
        color_from_dot_data((palette_layout >> 2) & 0b11),
        color_from_dot_data(palette_layout & 0b11),
    ]
}

fn color_from_dot_data(dot_data: u8) -> u8 {
    match dot_data {
        0b11 => 255,
        0b01 => 192,
        0b10 => 96,
        _ => 0,
    }
}
