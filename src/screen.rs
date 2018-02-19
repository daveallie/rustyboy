use std::borrow::Cow;
use glium::{self, glutin, texture, Surface};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct Screen {
    display: glium::Display,
    texture: texture::texture2d::Texture2d,
    events_loop: glutin::EventsLoop,
    screen_data_receiver: mpsc::Receiver<Vec<u8>>,
}

impl Screen {
    pub const WIDTH: u32 = 160;
    pub const HEIGHT: u32 = 144;

    pub fn new(title: &str, scale: u32, screen_data_receiver: mpsc::Receiver<Vec<u8>>) -> Self {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(Self::WIDTH * scale, Self::HEIGHT * scale);

        let context = glutin::ContextBuilder::new();
        let display = match glium::Display::new(window, context, &events_loop) {
            Ok(d) => d,
            Err(e) => panic!("Failed to create display: {}", e),
        };
        let texture = match texture::texture2d::Texture2d::empty_with_format(&display, texture::UncompressedFloatFormat::U8U8U8, texture::MipmapsOption::NoMipmap, Self::WIDTH, Self::HEIGHT) {
            Ok(t) => t,
            Err(e) => panic!("Failed to create texture: {}", e),
        };

        Self {
            display,
            texture,
            events_loop,
            screen_data_receiver,
        }
    }

    pub fn start_loop(&mut self) {
        let mut closed = false;

        while !closed {
            // listing the events produced by application and waiting to be received
            self.events_loop.poll_events(|ev| {
                if let glutin::Event::WindowEvent { event, .. } = ev {
                    if let glutin::WindowEvent::Closed = event {
                        closed = true;
                    }
                }
            });

            match self.screen_data_receiver.try_recv() {
                Ok(data) => self.draw_data(&*data),
                Err(mpsc::TryRecvError::Empty) => (),
                Err(mpsc::TryRecvError::Disconnected) => closed = true,
            }

            // Sleep for 1/60th of a second
            thread::sleep(Duration::new(0, 16_666));
        }
    }

    fn draw_data(&mut self, data: &[u8]) {
        let raw_image_2d = glium::texture::RawImage2d {
            data: Cow::Borrowed(data),
            width: Self::WIDTH,
            height: Self::HEIGHT,
            format: glium::texture::ClientFormat::U8U8U8,
        };

        self.texture.write(glium::Rect { left: 0, bottom: 0, width: Self::WIDTH, height: Self::HEIGHT }, raw_image_2d);

        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        let (unsigned_width, unsigned_height) = target.get_dimensions();
        // I have no idea why I need to double the width and height, only renders to quarter of window otherwise
        // Could be to do with my machine?
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        let width = 2 * i32::from(unsigned_width as u16);
        #[cfg_attr(feature="clippy", allow(cast_possible_truncation))]
        let height = 2 * i32::from(unsigned_height as u16);
        let blit_target = glium::BlitTarget { left: 0, bottom: 0, width, height };
        self.texture.as_surface().blit_whole_color_to(&target, &blit_target, glium::uniforms::MagnifySamplerFilter::Nearest);
        if let Err(e) = target.finish() {
            println!("ERROR: Failed to write to display: {}", e)
        }
    }
}
