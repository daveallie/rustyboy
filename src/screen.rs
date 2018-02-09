use std::borrow::Cow;
use glium::{self, glutin, texture, Surface};
use rand::{self, Rng};

pub struct Screen {
}

impl Screen {
    const WIDTH: u32 = 160;
    const HEIGHT: u32 = 144;

    pub fn new(title: &str, scale: u8) -> Screen {
        Screen {}
    }

    pub fn render(&mut self, data: u8) {

    }

    pub fn test_render() {
        let scale = 4;
        let mut events_loop = glutin::EventsLoop::new();
        let window = glutin::WindowBuilder::new()
            .with_title("Hello, world!")
            .with_dimensions(Screen::WIDTH * scale, Screen::HEIGHT * scale);

        let context = glutin::ContextBuilder::new();
        let display = glium::Display::new(window, context, &events_loop).unwrap();
        let texture = texture::texture2d::Texture2d::empty_with_format(&display, texture::UncompressedFloatFormat::U8U8U8, texture::MipmapsOption::NoMipmap, Screen::WIDTH, Screen::HEIGHT).unwrap();

        let mut closed = false;

        let mut rng = rand::thread_rng();

        while !closed {
            let mut datavec: Vec<u8> = (0..(Screen::WIDTH * Screen::HEIGHT)).flat_map(|i| {
                let col = match rng.gen_range(0, 4) {
                    0 => 255,
                    1 => 192,
                    2 => 96,
                    _ => 0,
                };

                vec![col, col, col]
            }).collect();
            let data = Cow::Borrowed(datavec.as_mut_slice());

            let rawimage2d = glium::texture::RawImage2d {
                data,
                width: 160,
                height: 144,
                format: glium::texture::ClientFormat::U8U8U8,
            };

            texture.write(glium::Rect { left: 0, bottom: 0, width: Screen::WIDTH, height: Screen::HEIGHT }, rawimage2d);

            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 1.0, 1.0);
            let (width, height) = target.get_dimensions();
            // I have no idea why I need to double the width and height, only renders to quarter of window otherwise
            // Could be to do with my machine?
            let blit_target = glium::BlitTarget { left: 0, bottom: 0, width: 2 * width as i32, height: 2 * height as i32 };
            texture.as_surface().blit_whole_color_to(&target, &blit_target, glium::uniforms::MagnifySamplerFilter::Nearest);
            target.finish().unwrap();

            // listing the events produced by application and waiting to be received
            events_loop.poll_events(|ev| {
                match ev {
                    glutin::Event::WindowEvent { event, .. } => match event {
                        glutin::WindowEvent::Closed => closed = true,
                        _ => (),
                    },
                    _ => (),
                }
            });
        }
    }

}
