use std::borrow::Cow;
use glium::{self, glutin, texture, Surface};
use std::sync::mpsc;

pub struct Screen {
    display: glium::Display,
    texture: texture::texture2d::Texture2d,
    events_loop: glutin::EventsLoop,
    screen_data_receiver: mpsc::Receiver<Vec<u8>>,
}

impl Screen {
    pub const WIDTH: u32 = 160;
    pub const HEIGHT: u32 = 144;

    pub fn new(title: &str, scale: u32, screen_data_receiver: mpsc::Receiver<Vec<u8>>) -> Screen {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(Screen::WIDTH * scale, Screen::HEIGHT * scale);

        let context = glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let texture = texture::texture2d::Texture2d::empty_with_format(&display, texture::UncompressedFloatFormat::U8U8U8, texture::MipmapsOption::NoMipmap, Screen::WIDTH, Screen::HEIGHT).unwrap();

        Screen {
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
                match ev {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::Closed => closed = true,
                        _ => (),
                    },
                    _ => (),
                }
            });

            match self.screen_data_receiver.try_recv() {
                Ok(data) => self.draw_data(&*data),
                Err(mpsc::TryRecvError::Empty) => (),
                Err(mpsc::TryRecvError::Disconnected) => closed = true,
            }
        }
    }

    fn draw_data(&mut self, data: &[u8]) {
        let raw_image_2d = glium::texture::RawImage2d {
            data: Cow::Borrowed(data),
            width: Screen::WIDTH,
            height: Screen::HEIGHT,
            format: glium::texture::ClientFormat::U8U8U8,
        };

        self.texture.write(glium::Rect { left: 0, bottom: 0, width: Screen::WIDTH, height: Screen::HEIGHT }, raw_image_2d);

        let mut target = self.display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        let (width, height) = target.get_dimensions();
        // I have no idea why I need to double the width and height, only renders to quarter of window otherwise
        // Could be to do with my machine?
        let blit_target = glium::BlitTarget { left: 0, bottom: 0, width: 2 * width as i32, height: 2 * height as i32 };
        self.texture.as_surface().blit_whole_color_to(&target, &blit_target, glium::uniforms::MagnifySamplerFilter::Nearest);
        target.finish().unwrap();
    }
}
