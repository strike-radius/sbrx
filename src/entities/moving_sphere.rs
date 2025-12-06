// entities/moving_sphere.rs

/// Represents a moving sphere that can crash and explode
pub struct MovingSphere {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub size: f64,
    pub crash_y: f64,
    pub exploding: bool,
    pub explosion_timer: f64,
    pub has_exploded: bool,
    pub crater_radius: f64,
    pub crater_timer: f64,
}

impl MovingSphere {
    pub fn new(x: f64, y: f64, speed: f64, size: f64, crash_y: f64) -> Self {
        MovingSphere {
            x,
            y,
            speed,
            size,
            crash_y,
            exploding: false,
            explosion_timer: 0.0,
            has_exploded: false,
            crater_radius: 0.0,
            crater_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f64) {
        if !self.exploding && !self.has_exploded {
            self.y += self.speed * dt;

            if self.y >= self.crash_y {
                self.exploding = true;
                self.explosion_timer = 0.5;
                self.crater_radius = self.size * 2.0;
            }
        } else if self.exploding {
            self.explosion_timer -= dt;
            if self.explosion_timer <= 0.0 {
                self.exploding = false;
                self.has_exploded = true;
                self.crater_timer = 3.0;
            }
        } else if self.has_exploded {
            self.crater_timer -= dt;
        }
    }

    pub fn is_visible(&self) -> bool {
        self.y <= 1080.0 + self.size && (!self.has_exploded || self.crater_timer > 0.0)
    }

    pub fn check_explosion_collision(&self, player_x: f64, player_y: f64) -> bool {
        if self.exploding {
            let explosion_size = self.size * (1.0 - self.explosion_timer / 0.5) * 2.0;
            let dx = player_x - self.x;
            let dy = player_y - self.y;
            let distance_squared = dx * dx + dy * dy;

            if distance_squared <= explosion_size * explosion_size {
                return true;
            }
        }
        false
    }
}
