use std::sync::Arc;

use winit::window::Window;

use crate::color::Palette;
use crate::grid::Grid;
use crate::sprite::Sprite;
use crate::tilemap::Tilemap;

#[derive(Clone)]
pub struct Screen {
    pub grid: Grid,
    pub palette: Palette,
    pub sprites: Vec<Sprite>,
    pub tilemaps: Vec<Tilemap>,
    pub window: Option<Arc<Window>>,
}

impl Default for Screen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen {
    pub fn new() -> Self {
        Self {
            grid: Grid::default(),
            palette: Palette::default(),
            sprites: Vec::new(),
            tilemaps: Vec::new(),
            window: None,
        }
    }
}
