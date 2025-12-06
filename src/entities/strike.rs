//File: /entities/strike.rs

use crate::utils::math::safe_gen_range;

/// Represents a melee strike attack
pub struct Strike {
    pub x: f64,
    pub y: f64,
    pub visible: bool,
    pub timer: f64,
    pub radius: f64,
    pub size: f64,
    pub angle: f64,
    pub length: f64,
}

impl Strike {
    pub fn new(radius: f64, size: f64) -> Self {
        Strike {
            x: 0.0,
            y: 0.0,
            visible: false,
            timer: 0.0,
            radius,
            size,
            angle: 0.0,
            length: 10.0,
        }
    }

    pub fn trigger(&mut self, x: f64, y: f64) {
        let vertical_radius = self.radius * 0.5;
        let crater_top = y - vertical_radius;
        let crater_bottom = y + vertical_radius;
        self.y = y.clamp(crater_top, crater_bottom);
        self.x = x;
        self.visible = true;
        self.timer = 0.1;
        // Use safe range for angle
        self.angle = safe_gen_range(-125.0, 125.0, "Strike angle");
    }

    pub fn update(&mut self, dt: f64) {
        if self.visible {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.visible = false;
            }
        }
    }
}
