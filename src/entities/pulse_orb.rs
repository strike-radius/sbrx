// entities/pulse_orb.rs

use crate::utils::vec2d::Vec2d;
use piston_window::*;

pub struct PulseOrb {
    pub x: f64,
    pub y: f64,
    pub vel: Vec2d,
    pub radius: f64,
    pub lifetime: f64,
    pub active: bool,
}

impl PulseOrb {
    pub fn new(x: f64, y: f64, target_x: f64, target_y: f64) -> Self {
        let speed = 750.0;
        let dx = target_x - x;
        let dy = target_y - y;
        let dist = (dx * dx + dy * dy).sqrt();

        let vel = if dist > 0.0 {
            Vec2d::new((dx / dist) * speed, (dy / dist) * speed)
        } else {
            Vec2d::new(speed, 0.0)
        };

        PulseOrb {
            x,
            y,
            vel,
            radius: 10.0,
            lifetime: 3.0, // Lasts 3 seconds
            active: true,
        }
    }

    pub fn update(&mut self, dt: f64) {
        self.x += self.vel.x * dt;
        self.y += self.vel.y * dt;
        self.lifetime -= dt;

        if self.lifetime <= 0.0 {
            self.active = false;
        }
    }

    pub fn draw(&self, c: Context, g: &mut G2d, texture: &G2dTexture) {
        if !self.active {
            return;
        }
        let w = texture.get_width() as f64;
        let h = texture.get_height() as f64;
        // Draw centered
        image(
            texture,
            c.transform.trans(self.x - w / 2.0, self.y - h / 2.0),
            g,
        );
    }
}
