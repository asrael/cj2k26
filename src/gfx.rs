use glam::IVec2;

use crate::color::Palette;

pub trait Drawable {
    fn draw(&self, frame: &mut [u8], palette: &Palette);
}

pub fn blit(frame: &mut [u8], palette: &Palette, origin: IVec2, w: i32, pixels: &[u8]) {
    for (i, &px) in pixels.iter().enumerate() {
        if px == crate::TRANSPARENT {
            continue;
        }

        let sx = origin.x + i as i32 % w;
        let sy = origin.y + i as i32 / w;
        if sx < 0 || sx >= crate::GAME_W as i32 || sy < 0 || sy >= crate::GAME_H as i32 {
            continue;
        }

        let idx = (sy as usize * crate::GAME_W + sx as usize) * 4;
        frame[idx..idx + 3].copy_from_slice(&palette.rgb(px));
    }
}
