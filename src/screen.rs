use glium::{self, glutin, texture, Surface};
use glutin::dpi::LogicalSize;
#[cfg(feature = "frame-capture")]
use image;
use input::{Key, KeyType};
use std::borrow::Cow;
#[cfg(feature = "frame-capture")]
use std::fs::File;
#[cfg(feature = "frame-capture")]
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub struct Screen {
    display: glium::Display,
    texture: texture::texture2d::Texture2d,
    events_loop: glutin::EventsLoop,
    screen_data_receiver: mpsc::Receiver<Vec<u8>>,
    key_data_sender: mpsc::Sender<Key>,
    screen_exit_sender: mpsc::Sender<()>,
    last_screen_render: Instant,
    min_render_space: Duration,
    throttled_state_sender: mpsc::Sender<bool>,
    throttled: bool,
    #[cfg(feature = "frame-capture")]
    frame_id: u64,
}

impl Screen {
    pub const WIDTH: u32 = 160;
    pub const HEIGHT: u32 = 144;

    pub fn new(
        title: &str,
        scale: u32,
        screen_data_receiver: mpsc::Receiver<Vec<u8>>,
        key_data_sender: mpsc::Sender<Key>,
        throttled_state_sender: mpsc::Sender<bool>,
        screen_exit_sender: mpsc::Sender<()>,
    ) -> Self {
        let events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title(title)
            .with_dimensions(LogicalSize::new(
                f64::from(Self::WIDTH * scale),
                f64::from(Self::HEIGHT * scale),
            ));

        let context = glutin::ContextBuilder::new();
        let display = match glium::Display::new(window, context, &events_loop) {
            Ok(d) => d,
            Err(e) => panic!("Failed to create display: {}", e),
        };
        let texture = match texture::texture2d::Texture2d::empty_with_format(
            &display,
            texture::UncompressedFloatFormat::U8U8U8,
            texture::MipmapsOption::NoMipmap,
            Self::WIDTH,
            Self::HEIGHT,
        ) {
            Ok(t) => t,
            Err(e) => panic!("Failed to create texture: {}", e),
        };

        Self {
            display,
            texture,
            events_loop,
            screen_data_receiver,
            key_data_sender,
            screen_exit_sender,
            last_screen_render: Instant::now(),
            min_render_space: Duration::new(0, 8_333_333), // 120 fps
            throttled: true,
            throttled_state_sender,
            #[cfg(feature = "frame-capture")]
            frame_id: 0,
        }
    }

    pub fn start_loop(&mut self) {
        self.main_screen_loop();
        let _ = self.screen_exit_sender.send(());
        loop {
            if let Err(mpsc::TryRecvError::Disconnected) = self.screen_data_receiver.try_recv() {
                break;
            }
        }
    }

    fn main_screen_loop(&mut self) {
        let mut closed = false;

        while !closed {
            closed = self.poll_for_window_events();

            match self.screen_data_receiver.try_recv() {
                Ok(data) => self.draw_data(&*data),
                Err(mpsc::TryRecvError::Empty) => (),
                Err(mpsc::TryRecvError::Disconnected) => closed = true,
            }

            // Sleep for 1/120th of a second (or 1/15th of that if unthrottled)
            thread::sleep(Duration::new(0, if self.throttled { 8_333_333 } else { 555_555 }));
        }
    }

    fn poll_for_window_events(&mut self) -> bool {
        let mut closed = false;
        let mut throttled = self.throttled;
        let key_sender = self.key_data_sender.clone();

        self.events_loop.poll_events(|ev| {
            if let glutin::Event::WindowEvent { event, .. } = ev {
                match event {
                    glutin::WindowEvent::CloseRequested => closed = true,
                    glutin::WindowEvent::KeyboardInput { input, .. } => {
                        let is_down = input.state == glutin::ElementState::Pressed;

                        match input.virtual_keycode {
                            Some(glutin::VirtualKeyCode::Up) => {
                                let _ = key_sender.send(Key {
                                    key_type: KeyType::Up,
                                    is_down,
                                });
                            }
                            Some(glutin::VirtualKeyCode::Down) => {
                                let _ = key_sender.send(Key {
                                    key_type: KeyType::Down,
                                    is_down,
                                });
                            }
                            Some(glutin::VirtualKeyCode::Left) => {
                                let _ = key_sender.send(Key {
                                    key_type: KeyType::Left,
                                    is_down,
                                });
                            }
                            Some(glutin::VirtualKeyCode::Right) => {
                                let _ = key_sender.send(Key {
                                    key_type: KeyType::Right,
                                    is_down,
                                });
                            }
                            Some(glutin::VirtualKeyCode::Z) => {
                                let _ = key_sender.send(Key {
                                    key_type: KeyType::A,
                                    is_down,
                                });
                            }
                            Some(glutin::VirtualKeyCode::X) => {
                                let _ = key_sender.send(Key {
                                    key_type: KeyType::B,
                                    is_down,
                                });
                            }
                            Some(glutin::VirtualKeyCode::C) => {
                                let _ = key_sender.send(Key {
                                    key_type: KeyType::Select,
                                    is_down,
                                });
                            }
                            Some(glutin::VirtualKeyCode::V) => {
                                let _ = key_sender.send(Key {
                                    key_type: KeyType::Start,
                                    is_down,
                                });
                            }
                            Some(glutin::VirtualKeyCode::Space) => {
                                throttled = !is_down;
                            }
                            Some(glutin::VirtualKeyCode::Q) => {
                                if input.modifiers.ctrl || input.modifiers.logo {
                                    closed = true;
                                }
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        });

        if throttled != self.throttled {
            self.throttled = throttled;
            let _ = self.throttled_state_sender.send(self.throttled);
        }

        closed
    }

    fn draw_data(&mut self, data: &[u8]) {
        if !self.throttled {
            let now = Instant::now();
            if now.duration_since(self.last_screen_render).lt(&self.min_render_space) {
                return;
            }

            self.last_screen_render = now;
        }

        let raw_image_2d = glium::texture::RawImage2d {
            data: Cow::Borrowed(data),
            width: Self::WIDTH,
            height: Self::HEIGHT,
            format: glium::texture::ClientFormat::U8U8U8,
        };

        #[cfg(feature = "frame-capture")]
        self.save_frame(data);

        self.texture.write(
            glium::Rect {
                left: 0,
                bottom: 0,
                width: Self::WIDTH,
                height: Self::HEIGHT,
            },
            raw_image_2d,
        );

        let target = self.display.draw();
        let (unsigned_width, unsigned_height) = target.get_dimensions();
        // I need to double the width and height as I'm developing on a retina display
        // only renders to quarter of window otherwise

        let width = i32::from(unsigned_width as u16);
        let height = i32::from(unsigned_height as u16);
        let blit_target = glium::BlitTarget {
            left: 0,
            bottom: height as u32,
            width,
            height: -height,
        };
        self.texture.as_surface().blit_whole_color_to(
            &target,
            &blit_target,
            glium::uniforms::MagnifySamplerFilter::Nearest,
        );
        if let Err(e) = target.finish() {
            println!("ERROR: Failed to write to display: {}", e)
        }
    }

    #[cfg(feature = "frame-capture")]
    fn save_frame(&mut self, data: &[u8]) {
        let image = image::ImageBuffer::from_raw(Self::WIDTH, Self::HEIGHT, data.to_vec()).unwrap();
        let image = image::DynamicImage::ImageRgb8(image);
        let mut output = File::create(&Path::new(&format!("frames/frame-{:010}.png", self.frame_id))).unwrap();
        self.frame_id += 1;
        image.save(&mut output, image::ImageFormat::PNG).unwrap();
    }
}
