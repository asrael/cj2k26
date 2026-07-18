use crate::color::Palette;
use crate::color::db32::{ORANGE, YELLOW};
use crate::gfx;
use crate::sprite::Sprite;
use crate::{GAME_H, GAME_W};

use aseprite::AsepriteFile;
use glam::{IVec2, Vec2};

const FIRE_COOLDOWN: u32 = 12;
const MOUSE_MAX: f32 = 2.0;

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

    pub fn win_anim(&mut self, t: u32) {
        let before = self.pos;

        if t < 40 {
            self.pos.y += 0.3;
        } else {
            self.pos.y -= ((t - 40) as f32 * 0.25).min(8.0);
        }

        self.step = self.pos - before;
    }

    pub fn pos(&self) -> Vec2 {
        self.pos
    }

    pub fn size(&self) -> Vec2 {
        self.sprite.size.as_vec2()
    }

    pub fn try_fire(&mut self) -> Option<Vec2> {
        (self.cooldown == 0).then(|| {
            self.cooldown = FIRE_COOLDOWN;
            self.pos + Vec2::new(self.sprite.size.x as f32 / 2.0, 0.0)
        })
    }

    pub fn update(&mut self, direction: IVec2, mouse: Vec2) {
        let before = self.pos;

        self.cooldown = self.cooldown.saturating_sub(1);

        let dir = direction.as_vec2().normalize_or_zero();
        self.pos += dir * self.speed + mouse.clamp_length_max(MOUSE_MAX);

        let w = self.sprite.size.x as f32;
        let h = self.sprite.size.y as f32;

        self.pos.x = self.pos.x.clamp(0.0, GAME_W as f32 - w);
        self.pos.y = self.pos.y.clamp(0.0, GAME_H as f32 - h);
        self.step = self.pos - before;
    }

    pub fn draw(&self, frame: &mut [u32], palette: &Palette, a: f32, tick: u32) {
        let pos = (self.pos - self.step * (1.0 - a)).round().as_ivec2();
        self.sprite.draw_at(frame, palette, pos);

        if self.step.length_squared() > 0.01 {
            let flame = if (tick / 4).is_multiple_of(2) {
                ORANGE
            } else {
                YELLOW
            };
            let exhaust = [flame, flame];
            let nozzle = pos + IVec2::new(self.sprite.size.x / 2, self.sprite.size.y);
            gfx::blit(frame, palette, &exhaust, nozzle, 1, 0);
        }
    }
}
