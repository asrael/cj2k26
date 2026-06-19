pub type Color = u32;

pub trait Rgb {
    fn rgb(&self) -> [u8; 3];
}

impl Rgb for Color {
    fn rgb(&self) -> [u8; 3] {
        [(self >> 16) as u8, (self >> 8) as u8, *self as u8]
    }
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
        self.0[index as usize].rgb()
    }
}
