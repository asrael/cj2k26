use glam::IVec2;

use crate::color::Palette;
use crate::gfx::{self, Drawable};

#[derive(Clone, Default)]
pub struct Sprite {
    pixels: Vec<u8>,
    pub pos: IVec2,
    pub size: IVec2,
}

impl Sprite {
    pub fn new(width: i32, height: i32, pos: IVec2, pixels: Vec<u8>) -> Self {
        assert_eq!(
            pixels.len(),
            (width * height) as usize,
            "sprite pixel buffer length must equal width * height"
        );

        Self {
            pixels,
            pos,
            size: IVec2::new(width, height),
        }
    }
}

impl Drawable for Sprite {
    fn draw(&self, frame: &mut [u8], palette: &Palette) {
        gfx::blit(frame, palette, &self.pixels, self.pos, self.size.x);
    }
}
