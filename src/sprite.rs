use aseprite::{AsepriteFile, CelKind};
use glam::{IVec2, Vec2};

use crate::color::Palette;
use crate::gfx;

#[derive(Clone, Default)]
pub struct Sprite {
    pixels: Vec<u8>,
    pub size: IVec2,
}

impl Sprite {
    pub fn new(width: i32, height: i32, pixels: Vec<u8>) -> Self {
        Self {
            pixels,
            size: IVec2::new(width, height),
        }
    }

    pub fn from_ase(file: &AsepriteFile, layer: &str) -> Self {
        let index = file
            .layers()
            .iter()
            .position(|l| l.name == layer)
            .unwrap_or_else(|| panic!("no layer named {layer}"));
        let cel = file.cel(file.layer_ref(index).unwrap(), 0).unwrap();

        let (CelKind::Raw { pixels, .. } | CelKind::Compressed { pixels, .. }) = &cel.kind else {
            panic!("layer {layer} has no pixel cel");
        };

        Self::new(
            pixels.width as i32,
            pixels.height as i32,
            pixels.data.clone(),
        )
    }

    pub fn draw_at(&self, frame: &mut [u8], palette: &Palette, pos: IVec2) {
        gfx::blit(frame, palette, &self.pixels, pos, self.size.x);
    }

    pub fn draw_rotated(&self, frame: &mut [u8], palette: &Palette, center: Vec2, angle: f32) {
        gfx::blit_rotated(frame, palette, &self.pixels, center, self.size.x, angle);
    }
}
