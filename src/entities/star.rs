// entities/star.rs

use crate::config::PERFORMANCE_MODE;
use piston_window::*;
use rand::Rng;

pub struct Star {
    pub x: f64,
    pub y: f64,
    pub brightness: f32,
    pub twinkle_speed: f32,
    pub twinkle_offset: f32,
    pub size: f64,
}

// Performance mode flag - you might want to move this to a config module later
//pub const PERFORMANCE_MODE: bool = false;

impl Star {
    pub fn new(x: f64, y: f64) -> Self {
        let mut rng = rand::thread_rng();
        Star {
            x,
            y,
            brightness: rng.gen_range(0.4..1.0),
            twinkle_speed: rng.gen_range(2.5..39.0),
            twinkle_offset: rng.gen_range(0.0..std::f32::consts::PI * 2.0),
            size: rng.gen_range(1.0..2.5),
        }
    }

    pub fn update(&mut self, dt: f64) {
        if PERFORMANCE_MODE {
            return;
        }
        // Update twinkle animation
        self.twinkle_offset += self.twinkle_speed * dt as f32;
        if self.twinkle_offset > std::f32::consts::PI * 2.0 {
            self.twinkle_offset -= std::f32::consts::PI * 2.0;
        }
    }

    pub fn current_brightness(&self) -> f32 {
        if PERFORMANCE_MODE {
            return self.brightness;
        }
        // Calculate current brightness based on sine wave
        let base_brightness = self.brightness * 0.7; // Minimum brightness
        let variable_brightness = self.brightness * 0.3; // Variable component
        base_brightness + variable_brightness * (self.twinkle_offset.sin() + 1.0) / 2.0
    }

    pub fn draw(&self, context: Context, g: &mut G2d) {
        let brightness = self.current_brightness();
        rectangle(
            [brightness, brightness, brightness, 1.0],
            [
                self.x - self.size / 2.0,
                self.y - self.size / 2.0,
                self.size,
                self.size,
            ],
            context.transform,
            g,
        );
    }
}
