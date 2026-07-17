use crate::color::Palette;
use crate::color::db32::{SALMON, WHITE};
use crate::sprite::Sprite;
use crate::{GAME_H, GAME_W};

use glam::Vec2;

const ENEMY_BULLET_W: i32 = 3;
const ENEMY_BULLET_H: i32 = 8;
const PLAYER_BULLET_W: i32 = 3;
const PLAYER_BULLET_H: i32 = 6;
const ENEMY_SHOT_SPEED: f32 = 1.5;
const PLAYER_SHOT_SPEED: f32 = 4.0;

#[rustfmt::skip]
const ENEMY_BULLET_PIXELS: [u8; (ENEMY_BULLET_W * ENEMY_BULLET_H) as usize] = [
    WHITE, 0,     0,
    0,     WHITE, 0,
    0,     0,     WHITE,
    0,     WHITE, 0,
    WHITE, 0,     0,
    0,     WHITE, 0,
    0,     0,     WHITE,
    0,     WHITE, 0,
];

#[rustfmt::skip]
const PLAYER_BULLET_PIXELS: [u8; (PLAYER_BULLET_W * PLAYER_BULLET_H) as usize] = [
    0,      SALMON,      0,
    SALMON, SALMON, SALMON,
    0,      0,           0,
    0,      SALMON,      0,
    0,      SALMON,      0,
    0,      SALMON,      0,
];

pub struct Bullet {
    pos: Vec2,
    vel: Vec2,
    sprite: Sprite,
}

impl Bullet {
    pub fn new(pos: Vec2, vel: Vec2, width: i32, height: i32, pixels: &[u8]) -> Self {
        let sprite = Sprite::new(width, height, pixels.to_vec());

        Self { pos, vel, sprite }
    }

    pub fn aimed(from: Vec2, target: Vec2) -> Self {
        let vel = (target - from).normalize_or_zero() * ENEMY_SHOT_SPEED;
        let pos = from + Vec2::new(-(ENEMY_BULLET_W as f32) / 2.0, 0.0);

        Self::new(
            pos,
            vel,
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
            PLAYER_BULLET_W,
            PLAYER_BULLET_H,
            &PLAYER_BULLET_PIXELS,
        )
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

    pub fn draw(&self, frame: &mut [u8], palette: &Palette, a: f32) {
        let p = self.pos - self.vel * (1.0 - a);
        self.sprite.draw_at(frame, palette, p.round().as_ivec2());
    }
}
