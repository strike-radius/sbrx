// entities/cpu_racer.rs

use crate::AudioManager;
use crate::rand::Rng;
use crate::combat::block::BlockSystem;
use crate::combat::combo::ComboSystem;
use crate::combat::stats::{Stats, CPU_RACER_LVL1_STATS};
use crate::game_state::{CombatMode, RacerState, MovementDirection};
use crate::utils::vec2d::Vec2d;
use crate::graphics::fighter_textures::FighterTextures;
use crate::entities::cpu_entity::BleedEffect;
use crate::game_state::EntityState;
use piston_window::*;

pub struct CpuRacer {
    pub x: f64,
    pub y: f64,
    pub state: RacerState,
    pub combat_mode: CombatMode,
    pub stats: Stats,
    
    pub current_hp: f64,
    pub max_hp: f64,
    pub fuel: f64,
    pub max_fuel: f64,
    
    pub stun_timer: f64,
    pub invincible_timer: f64,
    pub knockback_velocity: Vec2d,
    pub knockback_duration: f64,
    
    pub block_system: BlockSystem,
    pub combo_system: ComboSystem,
    pub bleed_effect: Option<BleedEffect>,
    
    pub is_crashed: bool,
    pub boost: bool,
    
    // Animation & Visual states
    pub facing_left: bool,
    pub movement_active: bool,
    pub backpedal_active: bool,
    pub rush_active: bool,
    pub block_break_animation_active: bool,
    pub current_movement_direction: MovementDirection,
    
    // Advanced Movement logic
    pub in_jump_sequence: bool,
    pub waypoints: Vec<Vec2d>,
    pub current_wp: usize,
	pub is_attacking: bool,
	pub attack_frame: usize,
	pub attack_timer: f64,
	pub attack_was_blocked: bool,
	pub block_sound_played: bool,
	pub frame_duration: f64,
	pub sound_effect_timer: f64,
	pub damage_display_cooldown: f64,
	pub strike_animation_timer: f64,
	pub bike_x: f64,
	pub bike_y: f64,
	pub bike_knockback_velocity: Vec2d,
	pub bike_knockback_duration: f64,
	pub phase: u32,
	pub ranged_cooldown: f64,
	pub ranged_shoot_visible: bool,
	pub ranged_shoot_timer: f64,
	pub ranged_shoot_start_x: f64,
	pub ranged_shoot_start_y: f64,
	pub ranged_shoot_target_x: f64,
	pub ranged_shoot_target_y: f64,
	pub ranged_animation_timer: f64,
	pub rush_cooldown: f64,
	pub rush_dir_x: f64,
	pub rush_dir_y: f64,
	pub rush_has_hit: bool,
	pub rush_timer: f64,
	pub entity_state: EntityState,
	pub reset_timer: f64,
}

impl CpuRacer {
    pub fn new(x: f64, y: f64) -> Self {
        let stats = CPU_RACER_LVL1_STATS;
        Self {
            x,
            y,
            state: RacerState::OnBike,
            combat_mode: CombatMode::CloseCombat,
            stats,
            current_hp: stats.defense.hp,
            max_hp: stats.defense.hp,
            fuel: 100.0,
            max_fuel: 100.0,
            stun_timer: 0.0,
            invincible_timer: 0.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            block_system: BlockSystem::new(20),
            combo_system: ComboSystem::new(),
            bleed_effect: None,
            is_crashed: false,
            boost: false,
            facing_left: false,
            movement_active: false,
            backpedal_active: false,
            rush_active: false,
            block_break_animation_active: false,
            current_movement_direction: MovementDirection::None,
            in_jump_sequence: false,
            current_wp: 0,
            waypoints: vec![
                Vec2d::new(350.0, 2750.0),
                Vec2d::new(1500.0, 2750.0),
                Vec2d::new(3500.0, 2750.0),
                Vec2d::new(4500.0, 2700.0),
                Vec2d::new(4300.0, 2450.0),
                Vec2d::new(3500.0, 2250.0),
                Vec2d::new(2500.0, 2250.0),
                Vec2d::new(1200.0, 2200.0),
                Vec2d::new(800.0, 1800.0),
                Vec2d::new(1400.0, 1500.0),
                Vec2d::new(2500.0, 1300.0),
                Vec2d::new(3800.0, 1300.0),
                Vec2d::new(4500.0, 1500.0),
                Vec2d::new(4600.0, 900.0),
                Vec2d::new(4000.0, 600.0),
                Vec2d::new(1500.0, 600.0),
                Vec2d::new(600.0, 900.0),
                Vec2d::new(600.0, 1500.0),
                Vec2d::new(600.0, 2100.0),
            ],
			is_attacking: false,
			attack_frame: 0,
			attack_timer: 0.0,
			attack_was_blocked: false,
			block_sound_played: false,
			frame_duration: 0.1,
			sound_effect_timer: 0.0,
			damage_display_cooldown: 0.0,
			strike_animation_timer: 0.0,
			bike_x: 0.0,
			bike_y: 0.0,
			bike_knockback_velocity: Vec2d::new(0.0, 0.0),
			bike_knockback_duration: 0.0,	
			phase: 1,
			ranged_cooldown: 0.0,
			ranged_shoot_visible: false,
			ranged_shoot_timer: 0.0,
			ranged_shoot_start_x: 0.0,
			ranged_shoot_start_y: 0.0,
			ranged_shoot_target_x: 0.0,
			ranged_shoot_target_y: 0.0,
			ranged_animation_timer: 0.0,
			rush_cooldown: 0.0,
			rush_dir_x: 0.0,
			rush_dir_y: 0.0,
			rush_has_hit: false,
			rush_timer: 0.0,
			entity_state: EntityState::Neutral,
			reset_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f64, rut_mult: f64, player_x: f64, player_y: f64, audio_manager: &crate::audio::AudioManager) {
		// Centralized Crash/Defeat state transition
		if self.current_hp <= 0.0 && !self.is_crashed {
			self.current_hp = 0.0;
			self.is_crashed = true;
			self.state = RacerState::OnFoot;
			self.is_attacking = false;
			self.stun_timer = 1.0;

			if self.phase == 1 {
				self.bike_x = self.x;
				self.bike_y = self.y;
				if self.bike_knockback_duration <= 0.0 {
					let bike_angle = if self.facing_left { 0.0 } else { std::f64::consts::PI } + 0.5;
					self.bike_knockback_velocity = Vec2d::new(bike_angle.cos() * 600.0, bike_angle.sin() * 600.0);
					self.bike_knockback_duration = 0.5;
				}
			}
			if self.phase == 2 {
				self.phase = 3;
				self.reset_timer = 5.0;
			}
			println!("CpuRacer HP hit 0 centrally! Transitioned to Phase: {}", self.phase);
		}		
        // Apply Bleed
        if let Some(ref mut bleed) = self.bleed_effect {
            bleed.tick_timer += dt;
            if bleed.tick_timer >= bleed.tick_rate {
                bleed.tick_timer = 0.0;
                let damage = bleed.damage_per_tick.min(bleed.remaining_damage);
                self.current_hp -= damage;
                bleed.remaining_damage -= damage;
                if bleed.remaining_damage <= 0.0 {
                    self.bleed_effect = None;
                }
            }
        }

        if self.stun_timer > 0.0 {
            self.stun_timer -= dt;
			if self.stun_timer <= 0.0 && self.is_crashed && self.phase == 1 {
				self.is_crashed = false;
				self.state = RacerState::OnFoot;
				self.current_hp = self.max_hp;
				self.phase = 2;
				println!("CpuRacer recovered from crash! Phase 2: OnFoot active.");
			}			
        }

        if self.invincible_timer > 0.0 {
            self.invincible_timer -= dt;
        }

        if self.knockback_duration > 0.0 {
            self.x += self.knockback_velocity.x * dt;
            self.y += self.knockback_velocity.y * dt;
            self.knockback_duration -= dt;
            if self.knockback_duration <= 0.0 {
                self.knockback_velocity = Vec2d::new(0.0, 0.0);
            }
		} else if self.rush_active {
			// Rush movement logic
			self.rush_timer -= dt;
			if self.rush_timer <= 0.0 {
				self.rush_active = false;
			} else {
				let rush_speed = 1800.0;
				let speed_mult = if self.boost { 1.25 } else { 1.0 } * rut_mult;
				self.x += self.rush_dir_x * rush_speed * speed_mult * dt;
				self.y += self.rush_dir_y * rush_speed * speed_mult * dt;
				
				self.x = self.x.max(crate::config::boundaries::MIN_X).min(crate::config::boundaries::MAX_X);
				self.y = self.y.max(crate::config::boundaries::MIN_Y).min(crate::config::boundaries::MAX_Y);
			}
		} else if self.stun_timer <= 0.0 && !self.is_crashed {
			let pdx = player_x - self.x;
			let pdy = player_y - self.y;
			let p_dist = (pdx * pdx + pdy * pdy).sqrt();

			let medium_proximity = 800.0;

			if p_dist < medium_proximity && self.entity_state == EntityState::Hostile {
				// Pursuit mode (locked-on to player_racer)
				if p_dist > 0.0 {
					let speed = if self.state == RacerState::OnFoot {
						self.stats.speed.run_speed
					} else {
						let base_speed = 650.0; // Standard BIKE_SPEED
						let speed_bonus = (self.stats.speed.run_speed - CPU_RACER_LVL1_STATS.speed.run_speed).max(0.0);
						base_speed + speed_bonus
					};
					let speed_mult = if self.boost { 1.25 } else { 1.0 } * rut_mult;
					
					self.x += (pdx / p_dist) * speed * speed_mult * dt;
					self.y += (pdy / p_dist) * speed * speed_mult * dt;
					
					self.facing_left = pdx < 0.0;
					self.movement_active = true;
				}
			} else {
				// Waypoint Navigation logic
				if !self.waypoints.is_empty() {
					let target = &self.waypoints[self.current_wp];
					let dx = target.x - self.x;
					let dy = target.y - self.y;
					let dist = (dx * dx + dy * dy).sqrt();

					if dist < 200.0 {
						self.current_wp = (self.current_wp + 1) % self.waypoints.len();
					}

					// Recalculate dx/dy in case waypoint changed
					let target = &self.waypoints[self.current_wp];
					let dx = target.x - self.x;
					let dy = target.y - self.y;
					let dist = (dx * dx + dy * dy).sqrt();

					if dist > 0.0 {
						let speed = if self.state == RacerState::OnFoot {
							self.stats.speed.run_speed
						} else {
							let base_speed = 650.0; // Standard BIKE_SPEED
							let speed_bonus = (self.stats.speed.run_speed - CPU_RACER_LVL1_STATS.speed.run_speed).max(0.0);
							base_speed + speed_bonus
						};
						let speed_mult = if self.boost { 1.25 } else { 1.0 } * rut_mult;
						
						self.x += (dx / dist) * speed * speed_mult * dt;
						self.y += (dy / dist) * speed * speed_mult * dt;
						
						self.facing_left = dx < 0.0;
						self.movement_active = true;
					}
				}
			}
		}

        self.block_system.update(dt, 0.0);
        self.combo_system.update(dt);

        if self.strike_animation_timer > 0.0 {
            self.strike_animation_timer -= dt;
        }		
		if self.is_attacking {
			if self.is_crashed || self.current_hp <= 0.0 || self.stun_timer > 0.0 {
				self.is_attacking = false;
				self.attack_was_blocked = false;
				self.sound_effect_timer = 0.0;
				self.attack_frame = 0;
				} else {				
					self.attack_timer += dt;
					if self.attack_timer >= self.frame_duration {
						self.attack_timer = 0.0;
						self.attack_frame = (self.attack_frame + 1) % 3;
					}
	 
					self.sound_effect_timer += dt;
					if self.sound_effect_timer >= 0.1 {
						self.sound_effect_timer = 0.0;
						if !self.attack_was_blocked {
							audio_manager.play_sound_effect("melee").ok();
						}
					}
				}	
		} else {
			self.attack_was_blocked = false;
			self.sound_effect_timer = 0.0;
			self.attack_frame = 0;
		}

		if self.damage_display_cooldown > 0.0 {
			self.damage_display_cooldown -= dt;
		}
		
		if self.bike_knockback_duration > 0.0 {
			self.bike_x += self.bike_knockback_velocity.x * dt;
			self.bike_y += self.bike_knockback_velocity.y * dt;
			self.bike_knockback_duration -= dt;
			if self.bike_knockback_duration <= 0.0 {
				self.bike_knockback_velocity = Vec2d::new(0.0, 0.0);
			}
		}	
		
		if self.ranged_cooldown > 0.0 {
			self.ranged_cooldown -= dt;
		}

		if self.ranged_shoot_visible {
			self.ranged_shoot_timer -= dt;
			if self.ranged_shoot_timer <= 0.0 {
				self.ranged_shoot_visible = false;
			}
		}
	
		if self.ranged_animation_timer > 0.0 {
			self.ranged_animation_timer -= dt;
 		}	
		
		if self.rush_cooldown > 0.0 {
			self.rush_cooldown -= dt;
		}
		if self.phase == 3 {
			if self.reset_timer > 0.0 {
				self.reset_timer -= dt;
				if self.reset_timer <= 0.0 {
					self.current_hp = self.max_hp;
					self.entity_state = EntityState::Neutral;
					self.is_crashed = false;
					self.state = RacerState::OnBike;
					self.phase = 1;
					self.is_attacking = false;
					self.stun_timer = 0.0;
					self.knockback_velocity = Vec2d::new(0.0, 0.0);
					self.knockback_duration = 0.0;
					self.bike_knockback_velocity = Vec2d::new(0.0, 0.0);
					self.bike_knockback_duration = 0.0;
					if !self.waypoints.is_empty() {
						self.x = self.waypoints[0].x;
						self.y = self.waypoints[0].y;
						self.current_wp = 0;
					}
					println!("CpuRacer reset after 5 seconds! Phase 1 restored.");
				}
			}
		}
		
    }
	
	pub fn check_collision(
		&mut self,
		racer_x: f64,
		racer_y: f64,
		is_blocking: bool,
		_audio_manager: &AudioManager,
	) -> bool {
 			if self.is_crashed || self.current_hp <= 0.0 || self.stun_timer > 0.0 || self.entity_state != EntityState::Hostile {
 				self.is_attacking = false;
 				return false;
 			}		
		
		let collision_distance = 100.0 + crate::config::gameplay::COLLISION_THRESHOLD / 2.0;
		let dx = racer_x - self.x;
		let dy = racer_y - self.y;
		let distance_squared = dx * dx + dy * dy;

		if distance_squared < collision_distance * collision_distance {
			if !self.is_attacking {
				self.is_attacking = true;
				self.attack_frame = 0;
				self.attack_timer = 0.0;
			}
			self.attack_was_blocked = is_blocking;
			return true;
		} else {
			if self.is_attacking {
				self.is_attacking = false;
				self.attack_frame = 0;
			}
			return false;
		}
	}	

    pub fn draw(&self, c: Context, g: &mut G2d, textures: &FighterTextures, crashed_bike_tex: &G2dTexture) {
 			if self.is_crashed || self.phase >= 2 {
 				let b_w = crashed_bike_tex.get_width() as f64;
 				let b_h = crashed_bike_tex.get_height() as f64;
 				image(crashed_bike_tex, c.transform.trans(self.bike_x - b_w / 2.0, self.bike_y - b_h / 2.0), g);
 			}
			
 			if self.ranged_shoot_visible {
 				let sx = self.ranged_shoot_start_x;
 				let sy = self.ranged_shoot_start_y;
 				let ex = self.ranged_shoot_target_x;
 				let ey = self.ranged_shoot_target_y;
 				let ldxs = ex - sx;
 				let ldys = ey - sy;
 				let bls = (ldxs * ldxs + ldys * ldys).sqrt();
 				if bls > 0.0 {
 					let mut ers = rand::rng();
 					for (lm, ofr, col, w) in [
 						(0.7, 5.0, [0.8, 0.8, 1.0, 0.7], 1.0),
 						(1.2, 8.0, [0.7, 0.7, 0.9, 0.6], 1.5),
 					] {
 						let ox = ers.random_range(-ofr..ofr);
 						let oy = ers.random_range(-ofr..ofr);
 						let ndx = ldxs / bls;
 						let ndy = ldys / bls;
 						let efx = sx + ndx * bls * lm + ox;
 						let efy = sy + ndy * bls * lm + oy;
 						line(col, w, [sx + ox, sy + oy, efx, efy], c.transform, g);
 					}
 				}
 			}			
 
 			if self.is_crashed {
 				let tex = &textures.block_break;
 				let tex_w = tex.get_width() as f64;
 				let tex_h = tex.get_height() as f64;
 				let img_x = self.x - tex_w / 2.0;
 				let img_y = self.y - tex_h / 2.0;
 
 				if self.facing_left {
 					let flip_transform = c.transform.trans(img_x + tex_w, img_y).scale(-1.0, 1.0);
 					image(tex, flip_transform, g);
 				} else {
 					image(tex, c.transform.trans(img_x, img_y), g);
 				}
 				return;
 			}
 
			let tex = if self.is_attacking {
				let current_texture_index = (self.attack_frame + 1).min(textures.strike.len() - 1);
				if self.state == RacerState::OnBike {
					&textures.bike_strike[current_texture_index]
				} else {
					&textures.strike[current_texture_index]
				}
  			} else if self.ranged_animation_timer > 0.0 {
 				if self.state == RacerState::OnBike {
 					&textures.bike_ranged
 				} else {
 					&textures.ranged
 				}			
 			} else if self.rush_active {
 				if self.state == RacerState::OnBike {
 					&textures.bike_rush
 				} else {
 					&textures.rush
 				}				
			} else if self.state == RacerState::OnBike {
				if self.movement_active {
					&textures.bike_accelerate[0] 
				} else {
					&textures.bike_idle
				}
			} else {
				if self.movement_active {
					&textures.fwd
				} else {
					&textures.idle
				}
			};

			let tex_w = tex.get_width() as f64;
			let tex_h = tex.get_height() as f64;
			let img_x = self.x - tex_w / 2.0;
			let img_y = self.y - tex_h / 2.0;

        if self.facing_left {
            let flip_transform = c.transform.trans(img_x + tex_w, img_y).scale(-1.0, 1.0);
            image(tex, flip_transform, g);
        } else {
            image(tex, c.transform.trans(img_x, img_y), g);
        }

        let hp_bar_width = 50.0;
        let hp_bar_height = 5.0;
        let bar_y_offset = tex_h / 2.0 + 15.0;
        let hp_bar_world_y = self.y - bar_y_offset;

        piston_window::rectangle(
            [0.5, 0.5, 0.5, 1.0],
            [self.x - hp_bar_width / 2.0, hp_bar_world_y, hp_bar_width, hp_bar_height],
            c.transform,
            g,
        );

        let current_hp_width = (self.current_hp / self.max_hp).max(0.0) * hp_bar_width;
		let hp_color = match self.entity_state {
			EntityState::Hostile => [1.0, 0.0, 0.0, 1.0], // Red
			EntityState::Neutral => [1.0, 1.0, 0.0, 1.0], // Yellow
			EntityState::Friendly => [0.0, 1.0, 0.0, 1.0], // Green
		};		
        piston_window::rectangle(
            hp_color,
            [self.x - hp_bar_width / 2.0, hp_bar_world_y, current_hp_width, hp_bar_height],
            c.transform,
            g,
        );
    }
}