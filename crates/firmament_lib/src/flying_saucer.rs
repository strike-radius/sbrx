// File: firmament_lib/src/flying_saucer.rs

use piston_window::*;
use rand::Rng;

pub struct FlyingSaucer {
    pub x: f64,
    pub y: f64,
    pub shields: i32,
    pub max_shields: i32,
    vel_x: f64,
    vel_y: f64,
    shoot_timer: f64,
    shoot_cooldown: f64,
    direction_change_timer: f64,
}

pub struct SaucerProjectile {
    pub x: f64,
    pub y: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub active: bool,
}

impl FlyingSaucer {
    pub fn new(x: f64, y: f64) -> Self {
        FlyingSaucer {
            x,
            y,
            shields: 15,
            max_shields: 15,
            vel_x: 200.0,
            vel_y: 100.0,
            shoot_timer: 2.0,
            shoot_cooldown: 3.0,
            direction_change_timer: 1.0,
        }
    }

    pub fn update(&mut self, dt: f64, window_width: f64, window_height: f64, player_pos: [f64; 2]) -> Option<Vec<SaucerProjectile>> {
        // Erratic movement
        self.direction_change_timer -= dt;
        if self.direction_change_timer <= 0.0 {
            let mut rng = rand::rng();
            self.vel_x = rng.gen_range(-300.0..300.0);
            self.vel_y = rng.gen_range(-200.0..200.0);
            self.direction_change_timer = rng.gen_range(0.5..2.0);
        }

        self.x += self.vel_x * dt;
        self.y += self.vel_y * dt;

        // Wrap around screen
        if self.x < 0.0 { self.x = window_width; }
        if self.x > window_width { self.x = 0.0; }
        if self.y < 0.0 { self.y = window_height; }
        if self.y > window_height { self.y = 0.0; }

        // Shooting logic
        self.shoot_timer -= dt;
        if self.shoot_timer <= 0.0 {
            self.shoot_timer = self.shoot_cooldown;
            return Some(self.create_spread_projectiles(player_pos));
        }

        None
    }

    fn create_spread_projectiles(&self, player_pos: [f64; 2]) -> Vec<SaucerProjectile> {
        let mut projectiles = Vec::new();
        let base_speed = 400.0;
        let angle_offsets_deg = [-30.0_f64, -15.0, 0.0, 15.0, 30.0];

        // Calculate base angle towards player
        let dx = player_pos[0] - self.x;
        let dy = player_pos[1] - self.y;
        let base_angle_rad = dy.atan2(dx);

        for angle_offset_deg in angle_offsets_deg.iter() {
            let current_angle_rad = base_angle_rad + angle_offset_deg.to_radians();
            projectiles.push(SaucerProjectile {
                x: self.x,
                y: self.y,
                vel_x: current_angle_rad.cos() * base_speed,
                vel_y: current_angle_rad.sin() * base_speed,
                active: true,
            });
        }

        projectiles
    }

    pub fn take_damage(&mut self) {
        if self.shields > 0 {
            self.shields -= 1;
        }
    }

    pub fn is_defeated(&self) -> bool {
        self.shields <= 0
    }

    pub fn draw(&self, context: Context, g: &mut G2d, texture: &G2dTexture) {
        let w = texture.get_width() as f64;
        let h = texture.get_height() as f64;
        image(texture, context.transform.trans(self.x - w / 2.0, self.y - h / 2.0), g);

        // Draw shield bar
        let bar_width = 100.0;
        let bar_height = 10.0;
        let bar_x = self.x - bar_width / 2.0;
        let bar_y = self.y - h / 2.0 - 20.0;

        rectangle([0.3, 0.3, 0.3, 1.0], [bar_x, bar_y, bar_width, bar_height], context.transform, g);
        let shield_width = (self.shields as f64 / self.max_shields as f64) * bar_width;
        rectangle([0.0, 0.5, 1.0, 1.0], [bar_x, bar_y, shield_width, bar_height], context.transform, g);
    }
}

	impl SaucerProjectile {
		pub fn update(&mut self, dt: f64, window_width: f64, window_height: f64) {
			self.x += self.vel_x * dt;
			self.y += self.vel_y * dt;

			// Deactivate if off screen
			if self.x < -50.0 || self.x > window_width + 50.0 || self.y < -50.0 || self.y > window_height + 50.0 {
				self.active = false;
			}
		}

		pub fn draw(&self, context: Context, g: &mut G2d, texture: Option<&G2dTexture>) {
			if self.active {
				if let Some(tex) = texture {
					let w = tex.get_width() as f64;
					let h = tex.get_height() as f64;
					image(tex, context.transform.trans(self.x - w / 2.0, self.y - h / 2.0), g);
				} else {
					// Fallback to red circle if texture not available
					ellipse([1.0, 0.0, 0.0, 1.0], [self.x - 5.0, self.y - 5.0, 10.0, 10.0], context.transform, g);         
			}
		}
	}
}