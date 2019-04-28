use cpal;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct Player {
    bit_buf: Arc<Mutex<Vec<(f32, f32)>>>,
}

impl Player {
    pub fn new() -> Self {
        let device = cpal::default_output_device().expect("Failed to get default output device");
        let format = cpal::Format {
            channels: 2,
            sample_rate: cpal::SampleRate(44_100),
            data_type: cpal::SampleFormat::F32,
        };

        let event_loop = cpal::EventLoop::new();
        let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
        event_loop.play_stream(stream_id);

        let bit_buf = Arc::new(Mutex::new(Vec::new()));

        let bb_clone = bit_buf.clone();
        thread::spawn(move || run_event_loop(event_loop, bb_clone));

        Player { bit_buf }
    }

    pub fn play(&mut self, l_stream: &[f32], r_stream: &[f32]) {
        let mut in_bit_buf = self.bit_buf.lock().unwrap();
        for (l, r) in l_stream.iter().zip(r_stream) {
            in_bit_buf.push((*l, *r));
        }
    }
}

fn run_event_loop(event_loop: cpal::EventLoop, bit_buf: Arc<Mutex<Vec<(f32, f32)>>>) {
    event_loop.run(move |_, data| {
        let mut in_bit_buf = bit_buf.lock().unwrap();

        match data {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                for (ref mut out, (in_l, in_r)) in buffer.chunks_mut(2).zip(in_bit_buf.iter()) {
                    out[0] = (in_l * f32::from(i16::max_value()) + f32::from(u16::max_value()) / 2.0) as u16;
                    out[1] = (in_r * f32::from(i16::max_value()) + f32::from(u16::max_value()) / 2.0) as u16;
                }
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                for (ref mut out, (in_l, in_r)) in buffer.chunks_mut(2).zip(in_bit_buf.iter()) {
                    out[0] = (in_l * f32::from(i16::max_value())) as i16;
                    out[1] = (in_r * f32::from(i16::max_value())) as i16;
                }
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for (ref mut out, (in_l, in_r)) in buffer.chunks_mut(2).zip(in_bit_buf.iter()) {
                    out[0] = *in_l;
                    out[1] = *in_r;
                }
            },
            _ => (),
        };

        in_bit_buf.clear();
    })
}
