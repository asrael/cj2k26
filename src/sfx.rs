use std::sync::mpsc::{Sender, channel};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{OutputCallbackInfo, Stream};
use resid::{ChipModel, PAL_CLOCK, SamplingMethod, Sid};

enum Sound {
    EnemyShoot,
    Explosion,
    Hit,
    Lose,
    Shoot,
    Win,
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

    fn set_freq(sid: &mut Sid, voice: u8, hz: f32) {
        let base = voice * 7;
        let freq = (hz * 16_777_216.0 / PAL_CLOCK as f32) as u16;

        sid.write(base, freq as u8);
        sid.write(base + 1, (freq >> 8) as u8);
    }

    fn trigger(sid: &mut Sid, hz: f32) {
        Self::set_freq(sid, 0, hz);
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
        sid.write(0x0C, 0x06);
        sid.write(0x0D, 0x00);
        sid.write(0x13, 0x08);
        sid.write(0x14, 0xA9);

        let mut sweep_hz: f32 = 0.0;
        let mut zap_hz: f32 = 0.0;
        let mut boom_hz: f32 = 0.0;
        let mut coin_t: u32 = 0;
        let mut tink_t: u32 = 0;
        let mut lose_t: u32 = 0;
        let mut lose_hz: f32 = 0.0;
        let mut lose_wob: f32 = 0.0;

        let stream = device
            .build_output_stream(
                config.into(),
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    while let Ok(sound) = rx.try_recv() {
                        match sound {
                            Sound::EnemyShoot => {
                                zap_hz = 520.0;
                                Self::set_freq(&mut sid, 1, zap_hz);
                                sid.write(0x0B, 0x10);
                                sid.write(0x0B, 0x11);
                            }

                            Sound::Explosion => {
                                tink_t = 0;
                                sid.write(0x13, 0x08);
                                sid.write(0x14, 0xA9);

                                boom_hz = 2500.0;
                                Self::set_freq(&mut sid, 2, boom_hz);
                                sid.write(0x12, 0x80);
                                sid.write(0x12, 0x81);
                            }

                            Sound::Hit => {
                                sid.write(0x13, 0x05);
                                sid.write(0x14, 0x00);

                                tink_t = 45;
                                Self::set_freq(&mut sid, 2, 2500.0);
                                sid.write(0x12, 0x80);
                                sid.write(0x12, 0x81);
                            }

                            Sound::Lose => {
                                zap_hz = 0.0;
                                sid.write(0x0C, 0x00);
                                sid.write(0x0D, 0xC6);
                                sid.write(0x09, 0x00);
                                sid.write(0x0A, 0x08);

                                lose_hz = 392.0;
                                lose_wob = 0.0;
                                lose_t = 300;
                                Self::set_freq(&mut sid, 1, lose_hz);
                                sid.write(0x0B, 0x40);
                                sid.write(0x0B, 0x41);
                            }

                            Sound::Shoot => {
                                sweep_hz = 1760.0;
                                Self::trigger(&mut sid, sweep_hz);
                            }

                            Sound::Win => {
                                sweep_hz = 0.0;
                                sid.write(0x05, 0x00);
                                sid.write(0x06, 0xA9);
                                sid.write(0x02, 0x00);
                                sid.write(0x03, 0x08);

                                coin_t = 300;
                                Self::set_freq(&mut sid, 0, 987.77);
                                sid.write(0x04, 0x40);
                                sid.write(0x04, 0x41);
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
                                Self::set_freq(&mut sid, 0, sweep_hz);
                            }
                        }

                        if zap_hz > 0.0 {
                            zap_hz *= 0.97;

                            if zap_hz < 200.0 {
                                zap_hz = 0.0;
                                sid.write(0x0B, 0x10);
                            } else {
                                Self::set_freq(&mut sid, 1, zap_hz);
                            }
                        }

                        if tink_t > 0 {
                            tink_t -= 1;
                            if tink_t == 0 {
                                sid.write(0x12, 0x80);
                            }
                        }

                        if coin_t > 0 {
                            coin_t -= 1;

                            if coin_t == 245 || coin_t == 190 {
                                let hz = if coin_t == 245 { 1318.5 } else { 1661.2 };
                                Self::set_freq(&mut sid, 0, hz);
                                sid.write(0x04, 0x40);
                                sid.write(0x04, 0x41);
                            }

                            if coin_t == 0 {
                                sid.write(0x04, 0x40);
                            }
                        }

                        if lose_t > 0 {
                            lose_t -= 1;
                            lose_hz *= 0.995;
                            lose_wob += 0.35;

                            let hz = lose_hz * (1.0 + 0.05 * lose_wob.sin());
                            Self::set_freq(&mut sid, 1, hz);

                            if lose_t == 0 {
                                sid.write(0x0B, 0x40);
                            }
                        }

                        if boom_hz > 0.0 {
                            boom_hz *= 0.985;

                            if boom_hz < 150.0 {
                                boom_hz = 0.0;
                                sid.write(0x12, 0x80);
                            } else {
                                Self::set_freq(&mut sid, 2, boom_hz);
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

    pub fn enemy_shoot(&self) {
        let _ = self.tx.send(Sound::EnemyShoot);
    }

    pub fn explode(&self) {
        let _ = self.tx.send(Sound::Explosion);
    }

    pub fn hit(&self) {
        let _ = self.tx.send(Sound::Hit);
    }

    pub fn lose(&self) {
        let _ = self.tx.send(Sound::Lose);
    }

    pub fn win(&self) {
        let _ = self.tx.send(Sound::Win);
    }

    pub fn shoot(&self) {
        let _ = self.tx.send(Sound::Shoot);
    }
}
