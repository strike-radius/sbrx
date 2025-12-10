// src/vehicle/fighter_jet.rs

use crate::config::boundaries::{MAX_X, MAX_Y, MIN_X, MIN_Y}; // Assuming these are the relevant boundaries
use crate::utils::math::safe_gen_range;
use piston_window::*;

pub struct FighterJet {
    pub x: f64,
    pub y: f64,
}

impl FighterJet {
    /// Creates a new fighter_jet at a random location within the game boundaries.
	#[allow(dead_code)] // TODO: Implement fighter jet spawning
    pub fn new() -> Self {
        FighterJet {
            x: safe_gen_range(MIN_X, MAX_X, "fighter_jet x"),
            y: safe_gen_range(MIN_Y, MAX_Y, "fighter_jet y"),
        }
    }

    /// Draws the fighter_jet.
    pub fn draw(&self, context: Context, g: &mut G2d, texture: &G2dTexture) {
        let sprite_width = texture.get_width() as f64;
        let sprite_height = texture.get_height() as f64;

        // Center the image on self.x, self.y
        let image_x = self.x - sprite_width / 2.0;
        let image_y = self.y - sprite_height / 2.0;

        image(texture, context.transform.trans(image_x, image_y), g);
    }
}
