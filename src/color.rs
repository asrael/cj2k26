pub type Color = u32;

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
