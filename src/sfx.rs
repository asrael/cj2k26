use std::sync::mpsc::{Sender, channel};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream};
use resid::{ChipModel, PAL_CLOCK, SamplingMethod, Sid};

enum Sound {
    Shoot,
}

pub struct Sfx {
    tx: Sender<Sound>,
    stream: Stream,
}

impl Sfx {
    fn fill(sid: &mut Sid, out: &mut [i16], sample_rate: u32) {
        let budget = PAL_CLOCK / sample_rate + 1;
        let mut i = 0;

        while i < out.len() {
            let target = (out.len() - i) as u32;
            let (n, _) = sid.sample(target * budget, &mut out[i..], 1);

            if n == 0 {
                break;
            }

            i += n;
        }
    }

    fn set_freq(sid: &mut Sid, hz: f32) {
        let freq = (hz * 16_777_216.0 / PAL_CLOCK as f32) as u16;

        sid.write(0x00, freq as u8);
        sid.write(0x01, (freq >> 8) as u8);
    }

    fn trigger(sid: &mut Sid, hz: f32) {
        Self::set_freq(sid, hz);
        sid.write(0x04, 0x20);
        sid.write(0x04, 0x21);
    }

    pub fn new() -> Self {
        let device = cpal::default_host()
            .default_output_device()
            .expect("no audio output device");

        let config = device
            .default_output_config()
            .expect("no default audio config");

        let channels = config.channels() as usize;
        let sample_rate = config.sample_rate();
        let mut scratch: Vec<i16> = Vec::new();
        let mut sid = Sid::new(ChipModel::Mos6581);
        let (tx, rx) = channel::<Sound>();

        sid.set_sampling_parameters(SamplingMethod::ResampleFast, PAL_CLOCK, sample_rate);
        sid.write(0x18, 0x0F);
        sid.write(0x05, 0x06);
        sid.write(0x06, 0x00);

        let mut sweep_hz: f32 = 0.0;

        let stream = device
            .build_output_stream(
                config.into(),
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    while let Ok(sound) = rx.try_recv() {
                        match sound {
                            Sound::Shoot => {
                                sweep_hz = 1760.0;
                                Self::trigger(&mut sid, sweep_hz);
                            }
                        }
                    }

                    let frames = data.len() / channels;
                    if scratch.len() < frames {
                        scratch.resize(frames, 0);
                    }
                    let out = &mut scratch[..frames];

                    for block in out.chunks_mut(64) {
                        if sweep_hz > 0.0 {
                            sweep_hz *= 0.975;

                            if sweep_hz < 220.0 {
                                sweep_hz = 0.0;
                                sid.write(0x04, 0x20);
                            } else {
                                Self::set_freq(&mut sid, sweep_hz);
                            }
                        }

                        Self::fill(&mut sid, block, sample_rate);
                    }

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

    pub fn resume(&self) {
        let _ = self.stream.play();
    }

    pub fn shoot(&self) {
        let _ = self.tx.send(Sound::Shoot);
    }
}
