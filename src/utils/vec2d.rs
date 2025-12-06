// utils/vec2d.rs

/// A 2D vector implementation for game physics and positioning
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

impl Vec2d {
    /// Creates a new vector with the given x and y components
    pub fn new(x: f64, y: f64) -> Self {
        Vec2d { x, y }
    }
}
