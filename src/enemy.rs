use crate::GAME_H;
use crate::color::Palette;
use crate::rng::Rng;
use crate::sprite::Sprite;

use std::f32::consts::TAU;

use aseprite::AsepriteFile;
use glam::Vec2;

const ROT_STEPS: f32 = 16.0;

fn bezier(p: [Vec2; 4], t: f32) -> Vec2 {
    let u = 1.0 - t;
    u * u * u * p[0] + 3.0 * u * u * t * p[1] + 3.0 * u * t * t * p[2] + t * t * t * p[3]
}

enum State {
    Dive { path: [Vec2; 4], t: f32, fired: u32 },
    Formation,
}

impl Default for State {
    fn default() -> Self {
        State::Formation
    }
}

pub struct Enemy {
    base: Vec2,
    pos: Vec2,
    step: Vec2,
    sprite: Sprite,
    state: State,
}

impl Enemy {
    pub fn new(sprites: &AsepriteFile, layer: &str, base: Vec2) -> Self {
        let pos = base;
        let step = Vec2::ZERO;
        let sprite = Sprite::from_ase(sprites, layer);
        let state = State::default();

        Self {
            base,
            pos,
            step,
            sprite,
            state,
        }
    }

    pub fn start_dive(&mut self, player_x: f32, rng: &mut Rng) {
        if matches!(self.state, State::Dive { .. }) {
            return;
        }

        let side = if rng.chance(0.5) { 1.0 } else { -1.0 };
        let path = [
            self.pos,
            self.pos + Vec2::new(side * 70.0, -40.0),
            Vec2::new(player_x, GAME_H as f32 + 60.0),
            self.base,
        ];

        self.state = State::Dive {
            path,
            t: 0.0,
            fired: 0,
        };
    }

    pub fn update(&mut self, sway: f32) -> Option<Vec2> {
        let before = self.pos;
        let mut shot = None;

        match &mut self.state {
            State::Formation => {
                let target = self.base.x + sway;
                self.pos.x += (target - self.pos.x) * 0.2;
            }

            State::Dive { path, t, fired } => {
                *t += 1.0 / 240.0;

                if *t >= 1.0 {
                    self.pos = self.base;
                    self.state = State::Formation;
                } else {
                    self.pos = bezier(*path, *t);

                    if (*fired == 0 && *t > 0.35) || (*fired == 1 && *t > 0.55) {
                        *fired += 1;
                        shot = Some(self.pos);
                    }
                }
            }
        }

        self.step = self.pos - before;
        shot
    }

    pub fn draw(&self, frame: &mut [u8], palette: &Palette, a: f32) {
        let pos = self.pos - self.step * (1.0 - a);

        if matches!(self.state, State::Dive { .. }) && self.step.length_squared() > 0.001 {
            let dir = self.step.normalize();
            let angle = (-dir.x).atan2(dir.y);
            let angle = (angle * ROT_STEPS / TAU).round() * (TAU / ROT_STEPS);
            let center = pos + self.sprite.size.as_vec2() / 2.0;
            self.sprite.draw_rotated(frame, palette, center, angle);
        } else {
            self.sprite.draw_at(frame, palette, pos.round().as_ivec2());
        }
    }
}
