// entities/fixed_crater.rs

/// Represents a crater fixed to the player's position
pub struct FixedCrater {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

impl FixedCrater {
    pub fn new(x: f64, y: f64, radius: f64) -> Self {
        FixedCrater { x, y, radius }
    }
}
