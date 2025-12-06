// graphics/camera.rs

use piston_window::*;

pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }

    pub fn update(&mut self, target_x: f64, target_y: f64) {
        let follow_speed = 0.1;
        self.x += (target_x - self.x) * follow_speed;
        self.y += (target_y - self.y) * follow_speed;
    }

    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom + 0.1).min(2.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom - 0.1).max(0.5);
    }

    pub fn transform(&self, c: Context) -> Context {
        let center_x = 1920.0 / 2.0;
        let center_y = 1080.0 / 2.0;
        c.trans(center_x, center_y)
            .scale(self.zoom, self.zoom)
            .trans(-self.x, -self.y)
    }
}

pub fn screen_to_world(camera: &Camera, mouse_x: f64, mouse_y: f64) -> (f64, f64) {
    let center_x = 1920.0 / 2.0;
    let center_y = 1080.0 / 2.0;
    let world_x = (mouse_x - center_x) / camera.zoom + camera.x;
    let world_y = (mouse_y - center_y) / camera.zoom + camera.y;
    (world_x, world_y)
}
