use std::sync::mpsc::{Sender, channel};

use resid::{ChipModel, Sid};
use tinyaudio::prelude::*;

const CLOCK: u32 = 985_248;
const SAMPLE_RATE: u32 = 44_100;

fn fill(sid: &mut Sid, out: &mut [i16]) {
    let mut i = 0;

    while i < out.len() {
        let want = (out.len() - i) as u32;
        let delta = want * (CLOCK / SAMPLE_RATE + 1);
        let (n, _) = sid.sample(delta, &mut out[i..], 1);

        if n == 0 {
            break;
        }

        i += n;
    }
}

fn trigger(sid: &mut Sid, hz: f32) {
    let reg = (hz * 16_777_216.0 / CLOCK as f32) as u16;

    sid.write(0x00, reg as u8);
    sid.write(0x01, (reg >> 8) as u8);
    sid.write(0x04, 0x20);
    sid.write(0x04, 0x21);
}

enum Sound {
    Bounce,
}

pub struct Sfx {
    tx: Sender<Sound>,
    _device: OutputDevice,
}

impl Sfx {
    pub fn new() -> Self {
        let mut sid = Sid::new(ChipModel::Mos6581);
        let (tx, rx) = channel::<Sound>();

        sid.write(0x18, 0x0F);
        sid.write(0x05, 0x08);
        sid.write(0x06, 0x00);

        let params = OutputDeviceParameters {
            channels_count: 1,
            channel_sample_count: 1024,
            sample_rate: SAMPLE_RATE as usize,
        };

        let mut buffer = Vec::new();
        let device = run_output_device(params, move |data| {
            while let Ok(sound) = rx.try_recv() {
                match sound {
                    Sound::Bounce => trigger(&mut sid, 587.0),
                }
            }

            if buffer.len() < data.len() {
                buffer.resize(data.len(), 0);
            }

            let out = &mut buffer[..data.len()];
            fill(&mut sid, out);

            for (d, s) in data.iter_mut().zip(out.iter()) {
                *d = *s as f32 / 32768.0;
            }
        })
        .expect("failed to create audio device");

        Self {
            tx,
            _device: device,
        }
    }

    pub fn bounce(&self) {
        let _ = self.tx.send(Sound::Bounce);
    }
}
