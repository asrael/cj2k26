use glam::Vec2;

pub fn aabb(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    a_pos.x < b_pos.x + b_size.x
        && b_pos.x < a_pos.x + a_size.x
        && a_pos.y < b_pos.y + b_size.y
        && b_pos.y < a_pos.y + a_size.y
}

pub fn bezier(p: [Vec2; 4], t: f32) -> Vec2 {
    let u = 1.0 - t;
    u * u * u * p[0] + 3.0 * u * u * t * p[1] + 3.0 * u * t * t * p[2] + t * t * t * p[3]
}
