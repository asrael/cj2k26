use std::sync::mpsc::{Receiver, Sender, channel};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, OutputCallbackInfo, Stream, SupportedStreamConfig};
use resid::{ChipModel, PAL_CLOCK, SamplingMethod, Sid};

const GATE: u8 = 0x01;
const TRIANGLE: u8 = 0x10;
const SAWTOOTH: u8 = 0x20;
const PULSE: u8 = 0x40;
const NOISE: u8 = 0x80;

enum Sound {
    EnemyShoot,
    Explosion,
    Hit,
    Lose,
    Shoot,
    Win,
}

struct Pending {
    config: SupportedStreamConfig,
    device: Device,
    rx: Receiver<Sound>,
    sid: Sid,
}

pub struct Sfx {
    pending: Option<Pending>,
    stream: Option<Stream>,
    tx: Sender<Sound>,
}

impl Sfx {
    pub fn new() -> Self {
        let device = cpal::default_host()
            .default_output_device()
            .expect("no audio output device");

        let config = device
            .default_output_config()
            .expect("no default audio config");

        let mut sid = Sid::new(ChipModel::Mos6581);
        let (tx, rx) = channel::<Sound>();

        sid.set_sampling_parameters(SamplingMethod::ResampleFast, PAL_CLOCK, config.sample_rate());
        Self::set_volume(&mut sid, 0x0F);
        Self::set_envelope(&mut sid, 0, 0x06, 0x00);
        Self::set_envelope(&mut sid, 1, 0x06, 0x00);
        Self::set_envelope(&mut sid, 2, 0x08, 0xA9);

        Self {
            pending: Some(Pending {
                config,
                device,
                rx,
                sid,
            }),
            stream: None,
            tx,
        }
    }

    pub fn resume(&mut self) {
        if let Some(stream) = &self.stream {
            let _ = stream.play();
            return;
        }

        let Some(Pending {
            config,
            device,
            rx,
            mut sid,
        }) = self.pending.take()
        else {
            return;
        };

        let channels = config.channels() as usize;
        let sample_rate = config.sample_rate();
        let mut scratch: Vec<i16> = Vec::new();

        let mut enemy_shoot_hz: f32 = 0.0;
        let mut explosion_hz: f32 = 0.0;
        let mut hit_t: u32 = 0;
        let mut lose_hz: f32 = 0.0;
        let mut lose_t: u32 = 0;
        let mut lose_wob: f32 = 0.0;
        let mut shoot_hz: f32 = 0.0;
        let mut win_t: u32 = 0;

        let stream = device
            .build_output_stream(
                config.into(),
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    while let Ok(sound) = rx.try_recv() {
                        match sound {
                            Sound::EnemyShoot => {
                                enemy_shoot_hz = 520.0;
                                Self::set_freq(&mut sid, 1, enemy_shoot_hz);
                                Self::gate_on(&mut sid, 1, TRIANGLE);
                            }

                            Sound::Explosion => {
                                hit_t = 0;
                                Self::set_envelope(&mut sid, 2, 0x08, 0xA9);

                                explosion_hz = 2500.0;
                                Self::set_freq(&mut sid, 2, explosion_hz);
                                Self::gate_on(&mut sid, 2, NOISE);
                            }

                            Sound::Hit => {
                                Self::set_envelope(&mut sid, 2, 0x05, 0x00);

                                hit_t = 45;
                                Self::set_freq(&mut sid, 2, 2500.0);
                                Self::gate_on(&mut sid, 2, NOISE);
                            }

                            Sound::Lose => {
                                enemy_shoot_hz = 0.0;
                                Self::set_envelope(&mut sid, 1, 0x00, 0xC6);
                                Self::set_pulse_width(&mut sid, 1, 0x0800);

                                lose_hz = 392.0;
                                lose_t = 300;
                                lose_wob = 0.0;
                                Self::set_freq(&mut sid, 1, lose_hz);
                                Self::gate_on(&mut sid, 1, PULSE);
                            }

                            Sound::Shoot => {
                                shoot_hz = 1760.0;
                                Self::set_freq(&mut sid, 0, shoot_hz);
                                Self::gate_on(&mut sid, 0, SAWTOOTH);
                            }

                            Sound::Win => {
                                shoot_hz = 0.0;
                                Self::set_envelope(&mut sid, 0, 0x00, 0xA9);
                                Self::set_pulse_width(&mut sid, 0, 0x0800);

                                win_t = 300;
                                Self::set_freq(&mut sid, 0, 987.77);
                                Self::gate_on(&mut sid, 0, PULSE);
                            }
                        }
                    }

                    let frames = data.len() / channels;
                    if scratch.len() < frames {
                        scratch.resize(frames, 0);
                    }
                    let out = &mut scratch[..frames];

                    for block in out.chunks_mut(64) {
                        Self::sweep(&mut sid, &mut shoot_hz, 0.975, 220.0, 0, SAWTOOTH);
                        Self::sweep(&mut sid, &mut enemy_shoot_hz, 0.97, 200.0, 1, TRIANGLE);
                        Self::sweep(&mut sid, &mut explosion_hz, 0.985, 150.0, 2, NOISE);

                        if hit_t > 0 {
                            hit_t -= 1;
                            if hit_t == 0 {
                                Self::gate_off(&mut sid, 2, NOISE);
                            }
                        }

                        if win_t > 0 {
                            win_t -= 1;

                            if win_t == 245 || win_t == 190 {
                                let hz = if win_t == 245 { 1318.5 } else { 1661.2 };
                                Self::set_freq(&mut sid, 0, hz);
                                Self::gate_on(&mut sid, 0, PULSE);
                            }

                            if win_t == 0 {
                                Self::gate_off(&mut sid, 0, PULSE);
                            }
                        }

                        if lose_t > 0 {
                            lose_t -= 1;
                            lose_hz *= 0.995;
                            lose_wob += 0.35;

                            let hz = lose_hz * (1.0 + 0.05 * lose_wob.sin());
                            Self::set_freq(&mut sid, 1, hz);

                            if lose_t == 0 {
                                Self::gate_off(&mut sid, 1, PULSE);
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
        self.stream = Some(stream);
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

    pub fn shoot(&self) {
        let _ = self.tx.send(Sound::Shoot);
    }

    pub fn win(&self) {
        let _ = self.tx.send(Sound::Win);
    }

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

    fn gate_off(sid: &mut Sid, voice: u8, wave: u8) {
        sid.write(voice * 7 + 4, wave);
    }

    fn gate_on(sid: &mut Sid, voice: u8, wave: u8) {
        sid.write(voice * 7 + 4, wave);
        sid.write(voice * 7 + 4, wave | GATE);
    }

    fn set_envelope(sid: &mut Sid, voice: u8, attack_decay: u8, sustain_release: u8) {
        let base = voice * 7;

        sid.write(base + 5, attack_decay);
        sid.write(base + 6, sustain_release);
    }

    fn set_freq(sid: &mut Sid, voice: u8, hz: f32) {
        let base = voice * 7;
        let freq = (hz * 16_777_216.0 / PAL_CLOCK as f32) as u16;

        sid.write(base, freq as u8);
        sid.write(base + 1, (freq >> 8) as u8);
    }

    fn set_pulse_width(sid: &mut Sid, voice: u8, width: u16) {
        let base = voice * 7;

        sid.write(base + 2, width as u8);
        sid.write(base + 3, (width >> 8) as u8);
    }

    fn set_volume(sid: &mut Sid, volume: u8) {
        sid.write(0x18, volume);
    }

    fn sweep(sid: &mut Sid, hz: &mut f32, rate: f32, floor: f32, voice: u8, wave: u8) {
        if *hz > 0.0 {
            *hz *= rate;

            if *hz < floor {
                *hz = 0.0;
                Self::gate_off(sid, voice, wave);
            } else {
                Self::set_freq(sid, voice, *hz);
            }
        }
    }
}
