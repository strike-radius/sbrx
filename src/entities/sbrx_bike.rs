// entities/sbrx_bike.rs

use crate::utils::math::safe_gen_range;
use crate::config::boundaries::{MIN_X, MAX_X, MIN_Y, MAX_Y};

/// Represents the SBRX bike that can be mounted and dismounted by the player
pub struct SbrxBike {
    pub x: f64,
    pub y: f64,
    pub visible: bool, // track if the bike is visible
	pub is_crashed: bool,
	pub knockback_velocity: crate::utils::vec2d::Vec2d,
	pub knockback_duration: f64,
}

impl SbrxBike {
    /// Create a new bike at a random position
    pub fn new(line_y: f64) -> Self {
        println!("Creating SbrxBike with line_y = {}", line_y);
        SbrxBike {
            x: safe_gen_range(50.0, 1870.0, "SbrxBike x"),
            y: safe_gen_range(line_y, line_y + 400.0, "SbrxBike y"),
            visible: true, // Start visible
			is_crashed: false,
			knockback_velocity: crate::utils::vec2d::Vec2d::new(0.0, 0.0),
			knockback_duration: 0.0,			
        }
    }
	
	pub fn update(&mut self, dt: f64) {
		if self.knockback_duration > 0.0 {
			self.x += self.knockback_velocity.x * dt;
			self.y += self.knockback_velocity.y * dt;
			
			// Constrain to player boundaries to prevent unreachable vehicles
			self.x = self.x.max(MIN_X).min(MAX_X);
			self.y = self.y.max(MIN_Y).min(MAX_Y);			
			
			self.knockback_duration -= dt;
			if self.knockback_duration <= 0.0 {
				self.knockback_velocity = crate::utils::vec2d::Vec2d::new(0.0, 0.0);
			}
		}
	}	

    /// Respawn the bike at the given coordinates
    pub fn respawn(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
        self.visible = true;
    }
}
