use crate::color::Palette;
use crate::color::db32::{BLUE, CYAN, GRAY, LIGHT_RED, LIME, PALE_BLUE, WHITE, YELLOW};
use crate::rng::Rng;
use crate::{GAME_H, GAME_W};

const STAR_COLORS: [u8; 8] = [WHITE, GRAY, PALE_BLUE, CYAN, BLUE, YELLOW, LIGHT_RED, LIME];

struct Star {
    color: u8,
    speed: f32,
    x: i32,
    y: f32,
}

#[derive(Default)]
pub struct Starfield {
    stars: Vec<Star>,
}

impl Starfield {
    pub fn new(count: usize, rng: &mut Rng) -> Self {
        let stars = (0..count)
            .map(|i| {
                let speed = match i % 3 {
                    0 => 0.5,
                    1 => 1.0,
                    _ => 1.5,
                };

                Star {
                    color: STAR_COLORS[rng.range(STAR_COLORS.len() as u32) as usize],
                    speed,
                    x: rng.range(GAME_W) as i32,
                    y: rng.range(GAME_H) as f32,
                }
            })
            .collect();

        Self { stars }
    }

    pub fn update(&mut self) {
        for star in &mut self.stars {
            star.y += star.speed;
            if star.y >= GAME_H as f32 {
                star.y -= GAME_H as f32;
            }
        }
    }

    pub fn draw(&self, frame: &mut [u32], palette: &Palette, a: f32) {
        for star in &self.stars {
            let y = star.y - star.speed * (1.0 - a);
            let y = (y as i32).rem_euclid(GAME_H as i32);

            frame[(y * GAME_W as i32 + star.x) as usize] = palette.at(star.color);
        }
    }
}
