use screen::Screen;
use std::sync::mpsc;

const VIDEO_RAM_SIZE: usize = 0x2000;
const SCREEN_PIXELS: usize = (Screen::WIDTH * Screen::HEIGHT) as usize;
const SCREEN_BUFFER: usize = 3 * SCREEN_PIXELS;

pub struct GPU {
    next_screen_pixel_palette: [u8; SCREEN_PIXELS],
    next_screen_buffer: [u8; SCREEN_BUFFER],
    video_ram: [u8; VIDEO_RAM_SIZE],
    bg_palette: u8,
    bg_palette_map: [(u8, u8, u8); 4],
    obj_palette_0: u8,
    obj_palette_0_map: [(u8, u8, u8); 4],
    obj_palette_1: u8,
    obj_palette_1_map: [(u8, u8, u8); 4],
    oam: [u8; GPU::OAM_SIZE], // Sprite attribute table
    lcd_control: u8,
    stat: u8,
    scy: u8,
    scx: u8,
    win_y: u8,
    win_x: u8,
    ly: u8,
    lyc: u8,
    render_clock: u32,
    screen_data_sender: mpsc::SyncSender<Vec<u8>>,
    pub interrupt: u8,
}

impl GPU {
    pub const OAM_SIZE: usize = 0xA0;

    pub fn new(screen_data_sender: mpsc::SyncSender<Vec<u8>>) -> Self {
        Self {
            next_screen_pixel_palette: [0_u8; SCREEN_PIXELS],
            next_screen_buffer: [0_u8; SCREEN_BUFFER],
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
            lyc: 0,
            render_clock: 0,
            screen_data_sender,
            interrupt: 0,
        }
    }

    pub fn run_cycle(&mut self, cycles: u8) {
        if !self.is_lcd_on() {
            return;
        }

        self.process_cycles(cycles);
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
            0xFF45 => self.lyc,
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
            0xFF45 => self.lyc = value,
            0xFF46 => unreachable!("DMA write handled in mmu.rs"),
            0xFF47 => {
                self.bg_palette = value;
                self.bg_palette_map = build_palette_map(value);
            }
            0xFF48 => {
                self.obj_palette_0 = value;
                self.obj_palette_0_map = build_palette_map(value);
            }
            0xFF49 => {
                self.obj_palette_1 = value;
                self.obj_palette_1_map = build_palette_map(value);
            }
            0xFF4A => self.win_y = value,
            0xFF4B => self.win_x = value,
            _ => panic!("Unknown GPU control write operation: 0x{:X}", addr),
        }
    }

    fn process_cycles(&mut self, cycles: u8) {
        let cycles_u32 = u32::from(cycles);
        if self.render_clock + cycles_u32 >= 114 {
            self.render_clock = (self.render_clock + cycles_u32) % 114;
            self.increment_line();
            self.render_background();
            self.render_sprites();
        } else {
            self.render_clock += cycles_u32;
        }
    }

    fn is_window_bg_on(&self) -> bool {
        self.lcd_control & 0x01 > 0
    }

    fn is_window_on(&self) -> bool {
        self.lcd_control & 0x20 > 0
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
        if self.stat & 0x40 > 0 && self.ly == self.lyc {
            self.interrupt |= 0x02;
        }

        if self.ly == 144 {
            // V-Blank
            self.interrupt |= 0x01; // Mark V-Blank interrupt
            self.render_screen();
        }
    }

    fn render_background(&mut self) {
        if !self.is_window_bg_on() || self.ly >= 144 {
            // bg and window display
            return;
        }

        let winy = self.ly.wrapping_sub(self.win_y);
        let winy_tile = (u16::from(winy) & 0xFF) >> 3;
        let winy_pixel_in_tile = u16::from(winy) & 0x07;

        let bgy = self.scy.wrapping_add(self.ly);
        let bgy_tile = (u16::from(bgy) & 0xFF) >> 3;
        let bgy_pixel_in_tile = u16::from(bgy) & 0x07;

        for x in 0..Screen::WIDTH {
            let (tile_number, x_pixel_in_tile, y_pixel_in_tile): (u8, u8, u16) = if self.rendering_window(x) {
                let winx = x + 7 - u32::from(self.win_x);
                #[cfg_attr(feature = "clippy", allow(cast_possible_truncation))]
                let winx_tile = ((winx & 0xFF) >> 3) as u16;
                #[cfg_attr(feature = "clippy", allow(cast_possible_truncation))]
                let winx_pixel_in_tile = 7 - (winx & 0x07) as u8;

                let tile_number: u8 = self.read_video_ram(self.window_tile_map_addr() + winy_tile * 32 + winx_tile);
                (tile_number, winx_pixel_in_tile, winy_pixel_in_tile)
            } else {
                let bgx = u32::from(self.scx) + x;
                #[cfg_attr(feature = "clippy", allow(cast_possible_truncation))]
                let bgx_tile = ((bgx & 0xFF) >> 3) as u16;
                #[cfg_attr(feature = "clippy", allow(cast_possible_truncation))]
                let bgx_pixel_in_tile = 7 - (bgx & 0x07) as u8;

                let tile_number: u8 = self.read_video_ram(self.bg_tile_map_addr() + bgy_tile * 32 + bgx_tile);
                (tile_number, bgx_pixel_in_tile, bgy_pixel_in_tile)
            };

            let tile_addr = self.get_tile_addr(tile_number);

            let tile_line_addr = tile_addr + y_pixel_in_tile * 2;
            let (tile_line_data_1, tile_line_data_2) = (
                self.read_video_ram(tile_line_addr),
                self.read_video_ram(tile_line_addr + 1),
            );
            let pixel_in_line_mask = 1 << x_pixel_in_tile;
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
            let palette_map = self.bg_palette_map;
            self.set_pixel_color_next_screen_buffer(x, palette_color_id, &palette_map);
        }
    }

    fn rendering_window(&self, x: u32) -> bool {
        self.is_window_on() && self.win_y <= self.ly && u32::from(self.win_x) <= x + 7
    }

    fn get_tile_addr(&self, tile_number: u8) -> u16 {
        let tile_data_addr = self.bg_and_window_tile_data_addr();

        let tile_addr_offset = if tile_data_addr == 0x8000 {
            // regular reading
            u16::from(tile_number) * 16
        } else {
            // reading with offset
            #[cfg_attr(
                feature = "clippy",
                allow(cast_possible_truncation, cast_sign_loss, cast_possible_wrap)
            )]
            let adjusted_tile_number = (i16::from(tile_number as i8) + 128) as u16;
            adjusted_tile_number * 16
        };

        tile_data_addr + tile_addr_offset
    }

    fn render_sprites(&mut self) {
        if !self.is_sprite_display_on() || self.ly >= 144 {
            return;
        }

        let sprite_height = if self.is_sprite_8_by_16() { 16 } else { 8 };

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

            let y_pixel_in_tile = u16::from(if y_flip {
                sprite_y + sprite_height - self.ly
            } else {
                self.ly - sprite_y
            });

            let sprite_addr = 0x8000_u16 + (u16::from(sprite_location) * 16) + y_pixel_in_tile * 2;
            let (sprite_data_1, sprite_data_2) =
                (self.read_video_ram(sprite_addr), self.read_video_ram(sprite_addr + 1));

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
                if palette_color_id == 0 {
                    // transparent
                    continue;
                }

                let palette_map = if use_palette_0 {
                    self.obj_palette_0_map
                } else {
                    self.obj_palette_1_map
                };
                let x = sprite_x.wrapping_add(x_pixel_in_tile);

                let x_pixel = u32::from(x);
                if sprite_under_bg && self.get_palette_color_id(x_pixel) > 0 {
                    continue;
                }
                self.set_pixel_color_next_screen_buffer(x_pixel, palette_color_id, &palette_map);
            }
        }
    }

    fn bg_and_window_tile_data_addr(&self) -> u16 {
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

    fn window_tile_map_addr(&self) -> u16 {
        if self.lcd_control & 0x40 > 0 {
            0x9C00
        } else {
            0x9800
        }
    }

    fn get_palette_color_id(&self, x_pixel: u32) -> u8 {
        let pixel_addr = (u32::from(self.ly) * Screen::WIDTH + x_pixel) as usize;
        self.next_screen_pixel_palette[pixel_addr]
    }

    fn set_pixel_color_next_screen_buffer(
        &mut self,
        x_pixel: u32,
        palette_color_id: u8,
        palette_map: &[(u8, u8, u8); 4],
    ) {
        let pixel_addr = (u32::from(self.ly) * Screen::WIDTH + x_pixel) as usize;
        self.next_screen_pixel_palette[pixel_addr] = palette_color_id;

        let base_buffer_addr = pixel_addr * 3;
        let (c1, c2, c3): (u8, u8, u8) = palette_map[palette_color_id as usize];
        self.next_screen_buffer[base_buffer_addr] = c1;
        self.next_screen_buffer[base_buffer_addr + 1] = c2;
        self.next_screen_buffer[base_buffer_addr + 2] = c3;
    }

    fn render_screen(&self) {
        match self.screen_data_sender.send(self.next_screen_buffer.to_vec()) {
            Ok(_) => (),
            Err(e) => println!("Failed to send screen data: {}", e),
        };
    }
}

fn build_palette_map(palette_layout: u8) -> [(u8, u8, u8); 4] {
    [
        color_from_dot_data(palette_layout & 0b11),
        color_from_dot_data((palette_layout >> 2) & 0b11),
        color_from_dot_data((palette_layout >> 4) & 0b11),
        color_from_dot_data(palette_layout >> 6),
    ]
}

// Black and white
fn color_from_dot_data(dot_data: u8) -> (u8, u8, u8) {
    match dot_data {
        0b00 => (255, 255, 255), // 255
        0b01 => (192, 192, 192), // 192
        0b10 => (105, 106, 106), // 96
        _ => (7, 9, 9),          // 0
    }
}

// Attempt at original gameboy colors
//fn color_from_dot_data(dot_data: u8) -> (u8, u8, u8) {
//    match dot_data {
//        0b00 => (245, 250, 239), // 255
//        0b01 => (134, 194, 112), // 192
//        0b10 => (47, 105, 87), // 96
//        _ => (11, 25, 32), // 0
//    }
//}

// Orange palette
//fn color_from_dot_data(dot_data: u8) -> (u8, u8, u8) {
//    match dot_data {
//        0b00 => (252, 232, 140), // 255
//        0b01 => (220, 180, 92), // 192
//        0b10 => (152, 124, 60), // 96
//        _ => (76, 60, 28), // 0
//    }
//}
