use crate::color::Palette;
use crate::gfx;

use aseprite::{AsepriteFile, CelKind};
use glam::{IVec2, Vec2};

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

    pub fn frames_from_ase(file: &AsepriteFile, layer: &str) -> Vec<Self> {
        let index = file
            .layers()
            .iter()
            .position(|l| l.name == layer)
            .unwrap_or_else(|| panic!("no layer named {layer}"));
        let layer_ref = file.layer_ref(index).unwrap();

        (0..file.frames().len())
            .filter_map(|f| file.resolve_cel(layer_ref, f))
            .filter_map(|cel| match &cel.kind {
                CelKind::Raw { pixels, .. } | CelKind::Compressed { pixels, .. } => {
                    Some(Self::new(
                        pixels.width as i32,
                        pixels.height as i32,
                        pixels.data.clone(),
                    ))
                }
                _ => None,
            })
            .collect()
    }

    pub fn draw_at(&self, frame: &mut [u32], palette: &Palette, pos: IVec2) {
        gfx::blit(frame, palette, &self.pixels, pos, self.size.x, 0);
    }

    pub fn draw_tinted(&self, frame: &mut [u32], palette: &Palette, pos: IVec2, tint: u8) {
        gfx::blit(frame, palette, &self.pixels, pos, self.size.x, tint);
    }

    pub fn draw_rotated(
        &self,
        frame: &mut [u32],
        palette: &Palette,
        center: Vec2,
        angle: f32,
        tint: u8,
    ) {
        gfx::blit_rotated(
            frame,
            palette,
            &self.pixels,
            center,
            self.size.x,
            angle,
            tint,
        );
    }
}
