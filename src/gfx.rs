use crate::color::Palette;
use crate::{GAME_H, GAME_W};

use glam::{IVec2, Vec2};

pub fn blit(frame: &mut [u8], palette: &Palette, pixels: &[u8], origin: IVec2, width: i32) {
    for (i, &px) in pixels.iter().enumerate() {
        if px == 0 {
            continue;
        }

        let sx = origin.x + i as i32 % width;
        let sy = origin.y + i as i32 / width;
        if sx < 0 || sx >= GAME_W as i32 || sy < 0 || sy >= GAME_H as i32 {
            continue;
        }

        let idx = (sy * GAME_W as i32 + sx) as usize * 4;
        frame[idx..idx + 3].copy_from_slice(&palette.rgb(px));
    }
}

pub fn blit_rotated(
    frame: &mut [u8],
    palette: &Palette,
    pixels: &[u8],
    center: Vec2,
    width: i32,
    angle: f32,
) {
    let height = pixels.len() as i32 / width;
    let half = Vec2::new(width as f32, height as f32) / 2.0;
    let (sin, cos) = angle.sin_cos();
    let r = half.length().ceil() as i32;

    for dy in -r..=r {
        for dx in -r..=r {
            let x = dx as f32 + 0.5;
            let y = dy as f32 + 0.5;
            let sx = (cos * x + sin * y + half.x).floor() as i32;
            let sy = (-sin * x + cos * y + half.y).floor() as i32;
            if sx < 0 || sx >= width || sy < 0 || sy >= height {
                continue;
            }

            let px = pixels[(sy * width + sx) as usize];
            if px == 0 {
                continue;
            }

            let fx = center.x.round() as i32 + dx;
            let fy = center.y.round() as i32 + dy;
            if fx < 0 || fx >= GAME_W as i32 || fy < 0 || fy >= GAME_H as i32 {
                continue;
            }

            let idx = (fy * GAME_W as i32 + fx) as usize * 4;
            frame[idx..idx + 3].copy_from_slice(&palette.rgb(px));
        }
    }
}

pub fn clear(frame: &mut [u8], rgb: [u8; 3]) {
    for pixel in frame.chunks_exact_mut(4) {
        pixel[..3].copy_from_slice(&rgb);
        pixel[3] = 0xFF;
    }
}
