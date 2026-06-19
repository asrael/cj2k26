use glam::IVec2;

use crate::color::Palette;
use crate::gfx::{Drawable, blit};

#[derive(Clone, Default)]
pub struct Tileset {
    tile_w: i32,
    tile_h: i32,
    tiles: Vec<u8>,
}

impl Tileset {
    pub fn new(tile_w: i32, tile_h: i32, tiles: Vec<u8>) -> Self {
        let size = (tile_w * tile_h) as usize;
        assert!(
            size > 0 && tiles.len() % size == 0,
            "tileset length must be a positive multiple of tile_w * tile_h"
        );

        Self {
            tile_w,
            tile_h,
            tiles,
        }
    }

    fn tile(&self, id: u16) -> &[u8] {
        let size = (self.tile_w * self.tile_h) as usize;
        let start = id as usize * size;

        &self.tiles[start..start + size]
    }
}

#[derive(Clone, Default)]
pub struct Tilemap {
    pos: IVec2,
    cols: i32,
    cells: Vec<u16>,
    tileset: Tileset,
}

impl Tilemap {
    pub fn new(pos: IVec2, cols: i32, rows: i32, cells: Vec<u16>, tileset: Tileset) -> Self {
        assert_eq!(
            cells.len(),
            (cols * rows) as usize,
            "tilemap cells length must equal cols * rows"
        );

        Self {
            pos,
            cols,
            cells,
            tileset,
        }
    }
}

impl Drawable for Tilemap {
    fn draw(&self, frame: &mut [u8], palette: &Palette) {
        let tw = self.tileset.tile_w;
        let th = self.tileset.tile_h;

        for (cell, &id) in self.cells.iter().enumerate() {
            let col = cell as i32 % self.cols;
            let row = cell as i32 / self.cols;
            let origin = self.pos + IVec2::new(col * tw, row * th);

            blit(frame, palette, origin, tw, self.tileset.tile(id));
        }
    }
}
