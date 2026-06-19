use glam::IVec2;

use crate::color::Palette;
use crate::gfx::{Drawable, blit};

#[derive(Clone, Default)]
pub struct Sprite {
    pixels: Vec<u8>,
    pub bbox: [IVec2; 2],
    pub vel: IVec2,
}

impl Sprite {
    pub fn new(pos: IVec2, w: i32, h: i32, pixels: Vec<u8>, vel: IVec2) -> Self {
        assert_eq!(
            pixels.len(),
            (w * h) as usize,
            "sprite pixel buffer length must equal width * height"
        );

        Self {
            bbox: [pos, IVec2::new(w, h)],
            pixels,
            vel,
        }
    }
}

impl Drawable for Sprite {
    fn draw(&self, frame: &mut [u8], palette: &Palette) {
        blit(frame, palette, self.bbox[0], self.bbox[1].x, &self.pixels);
    }
}
