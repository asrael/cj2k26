use glam::IVec2;

#[derive(Clone, Default)]
pub struct Grid {
    pos: IVec2,
    cols: i32,
    rows: i32,
    tile_w: i32,
    tile_h: i32,
    solid: Vec<bool>,
}

impl Grid {
    pub fn new(pos: IVec2, cols: i32, rows: i32, tile_w: i32, tile_h: i32) -> Self {
        Self {
            pos,
            cols,
            rows,
            tile_w,
            tile_h,
            solid: vec![false; (cols * rows) as usize],
        }
    }

    pub fn aabb(&self, pos: IVec2, dim: IVec2) -> bool {
        if self.solid.is_empty() {
            return false;
        }

        let cx0 = (pos.x - self.pos.x).div_euclid(self.tile_w);
        let cx1 = (pos.x + dim.x - 1 - self.pos.x).div_euclid(self.tile_w);
        let cy0 = (pos.y - self.pos.y).div_euclid(self.tile_h);
        let cy1 = (pos.y + dim.y - 1 - self.pos.y).div_euclid(self.tile_h);

        for row in cy0..=cy1 {
            for col in cx0..=cx1 {
                if col >= 0
                    && col < self.cols
                    && row >= 0
                    && row < self.rows
                    && self.solid[(row * self.cols + col) as usize]
                {
                    return true;
                }
            }
        }

        false
    }

    pub fn set(&mut self, col: i32, row: i32, solid: bool) {
        if col >= 0 && col < self.cols && row >= 0 && row < self.rows {
            self.solid[(row * self.cols + col) as usize] = solid;
        }
    }
}
