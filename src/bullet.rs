use crate::color::Palette;
use crate::color::db32::{LIGHT_RED, LIME};
use crate::math::aabb;
use crate::sprite::Sprite;
use crate::{GAME_H, GAME_W};

use glam::Vec2;

const ENEMY_BULLET_W: i32 = 2;
const ENEMY_BULLET_H: i32 = 6;
const PLAYER_BULLET_W: i32 = 3;
const PLAYER_BULLET_H: i32 = 6;
const ENEMY_SHOT_SPEED: f32 = 1.5;
const PLAYER_SHOT_SPEED: f32 = 4.0;

#[rustfmt::skip]
const ENEMY_BULLET_PIXELS: [u8; (ENEMY_BULLET_W * ENEMY_BULLET_H) as usize] = [
    LIME, LIME,
    LIME, LIME,
    LIME, LIME,
    LIME, LIME,
    LIME, LIME,
    LIME, LIME,
];

#[rustfmt::skip]
const PLAYER_BULLET_PIXELS: [u8; (PLAYER_BULLET_W * PLAYER_BULLET_H) as usize] = [
    0,      LIGHT_RED,      0,
    LIGHT_RED, LIGHT_RED, LIGHT_RED,
    0,      0,           0,
    0,      LIGHT_RED,      0,
    0,      LIGHT_RED,      0,
    0,      LIGHT_RED,      0,
];

pub struct Bullet {
    angle: Option<f32>,
    pos: Vec2,
    vel: Vec2,
    sprite: Sprite,
}

impl Bullet {
    fn new(
        pos: Vec2,
        vel: Vec2,
        angle: Option<f32>,
        width: i32,
        height: i32,
        pixels: &[u8],
    ) -> Self {
        let sprite = Sprite::new(width, height, pixels.to_vec());

        Self {
            angle,
            pos,
            vel,
            sprite,
        }
    }

    pub fn aimed(from: Vec2, target: Vec2) -> Self {
        let dir = (target - from).normalize_or_zero();
        let vel = dir * ENEMY_SHOT_SPEED;
        let pos = from + Vec2::new(-(ENEMY_BULLET_W as f32) / 2.0, 0.0);
        let angle = (-dir.x).atan2(dir.y);

        Self::new(
            pos,
            vel,
            Some(angle),
            ENEMY_BULLET_W,
            ENEMY_BULLET_H,
            &ENEMY_BULLET_PIXELS,
        )
    }

    pub fn fired(muzzle_pos: Vec2) -> Self {
        let pos =
            muzzle_pos + Vec2::new(-(PLAYER_BULLET_W as f32) / 2.0, -(PLAYER_BULLET_H as f32));

        Self::new(
            pos,
            Vec2::new(0.0, -PLAYER_SHOT_SPEED),
            None,
            PLAYER_BULLET_W,
            PLAYER_BULLET_H,
            &PLAYER_BULLET_PIXELS,
        )
    }

    pub fn hits(&self, pos: Vec2, size: Vec2) -> bool {
        aabb(self.pos, self.sprite.size.as_vec2(), pos, size)
    }

    pub fn offscreen(&self) -> bool {
        self.pos.x + self.sprite.size.x as f32 <= 0.0
            || self.pos.x >= GAME_W as f32
            || self.pos.y + self.sprite.size.y as f32 <= 0.0
            || self.pos.y >= GAME_H as f32
    }

    pub fn update(&mut self) {
        self.pos += self.vel;
    }

    pub fn draw(&self, frame: &mut [u32], palette: &Palette, a: f32) {
        let p = self.pos - self.vel * (1.0 - a);

        match self.angle {
            Some(angle) => {
                let center = p + self.sprite.size.as_vec2() / 2.0;
                self.sprite.draw_rotated(frame, palette, center, angle, 0);
            }
            None => self.sprite.draw_at(frame, palette, p.round().as_ivec2()),
        }
    }
}
