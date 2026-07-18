use crate::color::Palette;
use crate::{GAME_H, GAME_W};

use embedded_graphics::Drawable;
use embedded_graphics::Pixel;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::mono_font::{MonoFont, MonoTextStyle};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::text::{Baseline, Text};
use glam::{IVec2, Vec2};

pub struct Frame<'a>(pub &'a mut [u32]);

impl OriginDimensions for Frame<'_> {
    fn size(&self) -> Size {
        Size::new(GAME_W, GAME_H)
    }
}

impl DrawTarget for Frame<'_> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        for Pixel(p, color) in pixels {
            if p.x < 0 || p.x >= GAME_W as i32 || p.y < 0 || p.y >= GAME_H as i32 {
                continue;
            }

            self.0[(p.y * GAME_W as i32 + p.x) as usize] =
                (color.r() as u32) << 16 | (color.g() as u32) << 8 | color.b() as u32;
        }

        Ok(())
    }
}

pub fn draw_text(frame: &mut [u32], font: &MonoFont, text: &str, pos: IVec2, color: u32) {
    let color = Rgb888::new((color >> 16) as u8, (color >> 8) as u8, color as u8);
    let style = MonoTextStyle::new(font, color);

    let _ = Text::with_baseline(text, Point::new(pos.x, pos.y), style, Baseline::Top)
        .draw(&mut Frame(frame));
}

pub fn blit(
    frame: &mut [u32],
    palette: &Palette,
    pixels: &[u8],
    origin: IVec2,
    width: i32,
    tint: u8,
) {
    for (i, &px) in pixels.iter().enumerate() {
        if px == 0 {
            continue;
        }

        let sx = origin.x + i as i32 % width;
        let sy = origin.y + i as i32 / width;
        if sx < 0 || sx >= GAME_W as i32 || sy < 0 || sy >= GAME_H as i32 {
            continue;
        }

        let px = if tint != 0 { tint } else { px };
        frame[(sy * GAME_W as i32 + sx) as usize] = palette.at(px);
    }
}

pub fn blit_rotated(
    frame: &mut [u32],
    palette: &Palette,
    pixels: &[u8],
    center: Vec2,
    width: i32,
    angle: f32,
    tint: u8,
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

            let px = if tint != 0 { tint } else { px };
            frame[(fy * GAME_W as i32 + fx) as usize] = palette.at(px);
        }
    }
}
