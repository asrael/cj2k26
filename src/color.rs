pub type Color = u32;

#[allow(dead_code)]
pub mod db32 {
    pub const BLACK: u8 = 0;
    pub const DARK_PURPLE: u8 = 1;
    pub const MAROON: u8 = 2;
    pub const DARK_BROWN: u8 = 3;
    pub const BROWN: u8 = 4;
    pub const ORANGE: u8 = 5;
    pub const TAN: u8 = 6;
    pub const PEACH: u8 = 7;
    pub const YELLOW: u8 = 8;
    pub const LIME: u8 = 9;
    pub const GREEN: u8 = 10;
    pub const SEA_GREEN: u8 = 11;
    pub const DARK_GREEN: u8 = 12;
    pub const OLIVE: u8 = 13;
    pub const CHARCOAL: u8 = 14;
    pub const NAVY: u8 = 15;
    pub const STEEL_BLUE: u8 = 16;
    pub const INDIGO: u8 = 17;
    pub const BLUE: u8 = 18;
    pub const CYAN: u8 = 19;
    pub const PALE_BLUE: u8 = 20;
    pub const WHITE: u8 = 21;
    pub const GRAY: u8 = 22;
    pub const STONE: u8 = 23;
    pub const DIM_GRAY: u8 = 24;
    pub const SLATE: u8 = 25;
    pub const PURPLE: u8 = 26;
    pub const RED: u8 = 27;
    pub const SALMON: u8 = 28;
    pub const PINK: u8 = 29;
    pub const MOSS: u8 = 30;
    pub const KHAKI: u8 = 31;
}

const fn to_rgb(c: Color) -> [u8; 3] {
    [(c >> 16) as u8, (c >> 8) as u8, c as u8]
}

const fn from_rgb(r: u8, g: u8, b: u8) -> Color {
    (r as u32) << 16 | (g as u32) << 8 | b as u32
}

#[derive(Clone, Copy)]
pub struct Palette([Color; 256]);

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

impl Palette {
    pub const fn new() -> Self {
        Self([0; 256])
    }

    pub fn at(&self, index: u8) -> Color {
        self.0[index as usize]
    }

    pub fn cycle(&self, base: u8, offset: u8) -> Color {
        self.0[base.wrapping_add(offset) as usize]
    }

    pub fn index_of(&self, color: Color) -> Option<u8> {
        self.0.iter().position(|&c| c == color).map(|i| i as u8)
    }

    pub fn set(&mut self, index: u8, color: Color) {
        self.0[index as usize] = color;
    }

    pub fn rgb(&self, index: u8) -> [u8; 3] {
        to_rgb(self.0[index as usize])
    }

    pub fn from_ase(src: &[aseprite::Color]) -> Self {
        let mut out = [0u32; 256];

        for (dst, s) in out.iter_mut().zip(src) {
            *dst = from_rgb(s.r, s.g, s.b);
        }

        Palette(out)
    }
}
