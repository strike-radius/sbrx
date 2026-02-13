// raptor_nest.rs

use piston_window::*;

pub struct RaptorNest {
    pub x: f64,
    pub y: f64,
}

impl RaptorNest {
    /// Creates a new RaptorNest at a random location within the game boundaries.
    pub fn new() -> Self {
        RaptorNest {
            x: 2500.0,
            y: 1895.0,
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
