// entities/sbrx_bike.rs

use crate::utils::math::safe_gen_range;

/// Represents the SBRX bike that can be mounted and dismounted by the player
pub struct SbrxBike {
    pub x: f64,
    pub y: f64,
    pub visible: bool, // track if the bike is visible
}

impl SbrxBike {
    /// Create a new bike at a random position
    pub fn new(line_y: f64) -> Self {
        println!("Creating SbrxBike with line_y = {}", line_y);
        SbrxBike {
            x: safe_gen_range(50.0, 1870.0, "SbrxBike x"),
            y: safe_gen_range(line_y, line_y + 400.0, "SbrxBike y"),
            visible: true, // Start visible
        }
    }

    /// Respawn the bike at the given coordinates
    pub fn respawn(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
        self.visible = true;
    }
}
