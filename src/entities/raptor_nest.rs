// raptor_nest.rs

use crate::config::boundaries::{MAX_X, MAX_Y, MIN_X, MIN_Y};
use crate::utils::math::safe_gen_range;
use piston_window::*;

pub struct RaptorNest {
    pub x: f64,
    pub y: f64,
}

impl RaptorNest {
    /// Creates a new RaptorNest at a random location within the game boundaries.
    pub fn new() -> Self {
        RaptorNest {
            x: safe_gen_range(MIN_X, MAX_X, "RaptorNest x"),
            y: safe_gen_range(MIN_Y, MAX_Y, "RaptorNest y"),
        }
    }

    /// Draws the RaptorNest.
    pub fn draw(&self, context: Context, g: &mut G2d, texture: &G2dTexture) {
        let sprite_width = texture.get_width() as f64;
        let sprite_height = texture.get_height() as f64;

        // Center the image on self.x, self.y
        let image_x = self.x - sprite_width / 2.0;
        let image_y = self.y - sprite_height / 2.0;

        image(texture, context.transform.trans(image_x, image_y), g);
    }
}
