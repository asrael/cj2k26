use std::sync::mpsc::{Sender, channel};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream, StreamConfig};
use resid::{ChipModel, PAL_CLOCK, SamplingMethod, Sid};

fn fill(sid: &mut Sid, out: &mut [i16], sample_rate: u32) {
    let budget = PAL_CLOCK / sample_rate + 1;
    let mut i = 0;

    while i < out.len() {
        let want = (out.len() - i) as u32;
        let (n, _) = sid.sample(want * budget, &mut out[i..], 1);

        if n == 0 {
            break;
        }

        i += n;
    }
}

fn trigger(sid: &mut Sid, hz: f32) {
    let freq = (hz * 16_777_216.0 / PAL_CLOCK as f32) as u16;

    sid.write(0x00, freq as u8);
    sid.write(0x01, (freq >> 8) as u8);
    sid.write(0x04, 0x20);
    sid.write(0x04, 0x21);
}

fn new_sid(sample_rate: u32) -> Sid {
    let mut sid = Sid::new(ChipModel::Mos6581);

    sid.set_sampling_parameters(SamplingMethod::ResampleFast, PAL_CLOCK, sample_rate);
    sid.write(0x18, 0x0F);
    sid.write(0x05, 0x08);
    sid.write(0x06, 0x00);

    sid
}

enum Sound {
    Bounce,
}

pub struct Sfx {
    tx: Sender<Sound>,
    stream: Stream,
}

impl Sfx {
    pub fn new() -> Self {
        let (tx, rx) = channel::<Sound>();

        let device = cpal::default_host()
            .default_output_device()
            .expect("no audio output device");
        let config = device
            .default_output_config()
            .expect("no default audio config");
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;
        let config: StreamConfig = config.into();

        let mut sid = new_sid(sample_rate);
        let mut scratch: Vec<i16> = Vec::new();

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    while let Ok(sound) = rx.try_recv() {
                        match sound {
                            Sound::Bounce => trigger(&mut sid, 220.0),
                        }
                    }

                    let frames = data.len() / channels;
                    if scratch.len() < frames {
                        scratch.resize(frames, 0);
                    }
                    let out = &mut scratch[..frames];
                    fill(&mut sid, out, sample_rate);

                    for (frame, &s) in data.chunks_mut(channels).zip(out.iter()) {
                        frame.fill(s as f32 / 32768.0);
                    }
                },
                move |err| log::error!("audio stream error: {err}"),
                None,
            )
            .expect("failed to build audio stream");

        stream.play().expect("failed to start audio stream");

        Self { tx, stream }
    }

    pub fn bounce(&self) {
        let _ = self.tx.send(Sound::Bounce);
    }

    pub fn resume(&self) {
        let _ = self.stream.play();
    }
}
