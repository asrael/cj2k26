use crate::color::Palette;
use crate::color::db32::{LIGHT_RED, PINK, PURPLE};
use crate::gfx;
use crate::math;
use crate::rng::Rng;
use crate::sprite::Sprite;
use crate::{GAME_H, GAME_W};

use std::f32::consts::TAU;

use aseprite::AsepriteFile;
use glam::Vec2;

const ROT_STEPS: f32 = 16.0;

#[derive(Default)]
enum State {
    Dive {
        path: [Vec2; 4],
        fired: u32,
        shots: u32,
        t: f32,
    },
    #[default]
    Formation,
}

pub struct Enemy {
    base: Vec2,
    flash: u32,
    hp: u32,
    pos: Vec2,
    step: Vec2,
    sprite: Sprite,
    state: State,
}

impl Enemy {
    pub fn new(sprites: &AsepriteFile, layer: &str, base: Vec2, hp: u32) -> Self {
        let pos = base;
        let step = Vec2::ZERO;
        let sprite = Sprite::from_ase(sprites, layer);
        let state = State::default();

        Self {
            base,
            flash: 0,
            hp,
            pos,
            step,
            sprite,
            state,
        }
    }

    pub fn damage(&mut self) -> bool {
        self.hp = self.hp.saturating_sub(1);
        self.flash = 12;
        self.hp == 0
    }

    pub fn center(&self) -> Vec2 {
        self.pos + self.size() / 2.0
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn size(&self) -> Vec2 {
        self.sprite.size.as_vec2()
    }

    pub fn start_dive(&mut self, player_x: f32, shots: u32, rng: &mut Rng) {
        if matches!(self.state, State::Dive { .. }) {
            return;
        }

        let side = if self.pos.x < 70.0 {
            1.0
        } else if self.pos.x > GAME_W as f32 - 70.0 {
            -1.0
        } else if rng.chance(0.5) {
            1.0
        } else {
            -1.0
        };

        let min = Vec2::new(4.0, 4.0);
        let max = Vec2::new(GAME_W as f32 - 20.0, GAME_H as f32 - 20.0);
        let swing = (self.pos + Vec2::new(side * 70.0, -40.0)).clamp(min, max);
        let plunge = Vec2::new(player_x, GAME_H as f32 - 20.0).clamp(min, max);

        let path = [self.pos, swing, plunge, self.base];

        self.state = State::Dive {
            path,
            fired: 0,
            shots,
            t: 0.0,
        };
    }

    pub fn update(&mut self, sway: f32, player: Vec2) -> Option<Vec2> {
        let before = self.pos;
        let mut shot = None;

        self.flash = self.flash.saturating_sub(1);

        match &mut self.state {
            State::Formation => {
                let target = self.base.x + sway;
                self.pos.x += (target - self.pos.x) * 0.2;
            }

            State::Dive {
                path,
                fired,
                shots,
                t,
            } => {
                *t += 1.0 / 240.0;

                if *t >= 1.0 {
                    self.pos = self.base;
                    self.state = State::Formation;
                } else {
                    self.pos = math::bezier(*path, *t);

                    let center = self.pos + self.sprite.size.as_vec2() / 2.0;
                    let facing = self.step.normalize_or_zero();
                    let to_player = (player - center).normalize_or_zero();

                    let next = 0.25 + 0.5 * *fired as f32 / *shots as f32;
                    if *fired < *shots && *t > next && facing.dot(to_player) > 0.8 {
                        *fired += 1;
                        shot = Some(center);
                    }
                }
            }
        }

        self.step = self.pos - before;
        shot
    }

    pub fn draw(&self, frame: &mut [u32], palette: &Palette, a: f32, tick: u32) {
        let pos = self.pos - self.step * (1.0 - a);
        let tint = if self.flash > 0 && (tick / 2).is_multiple_of(2) {
            LIGHT_RED
        } else {
            0
        };

        if matches!(self.state, State::Dive { .. }) && self.step.length_squared() > 0.001 {
            let dir = self.step.normalize();
            let angle = (-dir.x).atan2(dir.y);
            let angle = (angle * ROT_STEPS / TAU).round() * (TAU / ROT_STEPS);
            let center = pos + self.sprite.size.as_vec2() / 2.0;
            self.sprite
                .draw_rotated(frame, palette, center, angle, tint);

            let half = self.sprite.size.y as f32 / 2.0;
            for i in 0..3 {
                let tail = center - dir * (half + 1.0 + i as f32 * 2.0);
                let c = [PURPLE, PINK][(tick / 4 + i) as usize % 2];
                gfx::blit(frame, palette, &[c], tail.round().as_ivec2(), 1, 0);
            }
        } else {
            self.sprite
                .draw_tinted(frame, palette, pos.round().as_ivec2(), tint);
        }
    }
}
