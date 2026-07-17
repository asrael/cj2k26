use crate::color::Palette;
use crate::sprite::Sprite;
use crate::{GAME_H, GAME_W};

use aseprite::AsepriteFile;
use glam::{IVec2, Vec2};

const FIRE_COOLDOWN: u32 = 12;

#[derive(Default)]
pub struct Player {
    cooldown: u32,
    pos: Vec2,
    step: Vec2,
    speed: f32,
    sprite: Sprite,
}

impl Player {
    pub fn new(sprites: &AsepriteFile, speed: f32) -> Self {
        let cooldown = 0;
        let pos = Vec2::new(GAME_W as f32 / 2.0, GAME_H as f32 / 2.0);
        let step = Vec2::ZERO;
        let sprite = Sprite::from_ase(sprites, "player");

        Self {
            cooldown,
            pos,
            step,
            speed,
            sprite,
        }
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn try_fire(&mut self) -> Option<Vec2> {
        (self.cooldown == 0).then(|| {
            self.cooldown = FIRE_COOLDOWN;
            self.pos + Vec2::new(self.sprite.size.x as f32 / 2.0, 0.0)
        })
    }

    pub fn update(&mut self, direction: IVec2) {
        let before = self.pos;

        self.cooldown = self.cooldown.saturating_sub(1);
        self.pos += direction.as_vec2().normalize_or_zero() * self.speed;

        let w = self.sprite.size.x as f32;
        let h = self.sprite.size.y as f32;

        self.pos.x = self.pos.x.clamp(0.0, GAME_W as f32 - w);
        self.pos.y = self.pos.y.clamp(0.0, GAME_H as f32 - h);
        self.step = self.pos - before;
    }

    pub fn draw(&self, frame: &mut [u8], palette: &Palette, a: f32) {
        let pos = self.pos - self.step * (1.0 - a);
        self.sprite.draw_at(frame, palette, pos.round().as_ivec2());
    }
}
