use crate::color::Palette;
use crate::sprite::Sprite;

use std::rc::Rc;

use glam::Vec2;

const FRAME_TICKS: u32 = 6;

pub struct Anim {
    center: Vec2,
    frames: Rc<Vec<Sprite>>,
    tick: u32,
}

impl Anim {
    pub fn new(frames: Rc<Vec<Sprite>>, center: Vec2) -> Self {
        Self {
            center,
            frames,
            tick: 0,
        }
    }

    pub fn done(&self) -> bool {
        self.tick >= FRAME_TICKS * self.frames.len() as u32
    }

    pub fn update(&mut self) {
        self.tick += 1;
    }

    pub fn draw(&self, frame: &mut [u32], palette: &Palette) {
        let i = (self.tick / FRAME_TICKS) as usize;
        let Some(sprite) = self.frames.get(i) else {
            return;
        };

        let pos = self.center - sprite.size.as_vec2() / 2.0;
        sprite.draw_at(frame, palette, pos.round().as_ivec2());
    }
}
