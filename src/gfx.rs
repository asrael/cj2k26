use glam::IVec2;

use crate::color::Palette;

pub trait Drawable {
    fn draw(&self, frame: &mut [u8], palette: &Palette);
}

pub fn blit(frame: &mut [u8], palette: &Palette, pixels: &[u8], origin: IVec2, width: i32) {
    for (i, &px) in pixels.iter().enumerate() {
        if px == crate::TRANSPARENT {
            continue;
        }

        let sx = origin.x + i as i32 % width;
        let sy = origin.y + i as i32 / width;
        if sx < 0 || sx >= crate::GAME_W as i32 || sy < 0 || sy >= crate::GAME_H as i32 {
            continue;
        }

        let idx = (sy * crate::GAME_W as i32 + sx) as usize * 4;
        frame[idx..idx + 3].copy_from_slice(&palette.rgb(px));
    }
}
