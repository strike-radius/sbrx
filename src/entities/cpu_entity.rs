// File: entities/cpu_entity.rs

use crate::combat::skills::*;
use crate::config::{boundaries::*, CPU_ENABLED};
use crate::rand::Rng;
use crate::utils::math::safe_gen_range;
use crate::utils::vec2d::Vec2d;
use crate::AudioManager;
use piston_window::*;

use crate::{BUNKER_HEIGHT, BUNKER_ORIGIN_X, BUNKER_ORIGIN_Y, BUNKER_WIDTH};

#[derive(Clone)]
pub struct BleedEffect {
    pub remaining_damage: f64,
    pub tick_timer: f64,
    pub tick_rate: f64,
    pub damage_per_tick: f64,
}

impl BleedEffect {
    pub fn new(total_damage: f64) -> Self {
        BleedEffect {
            remaining_damage: total_damage,
            tick_timer: 0.0,
            tick_rate: 0.5,       // damage ticks every 0.5 seconds
            damage_per_tick: 5.0, // 5 damage per tick
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)] // Added for type comparison
pub enum CpuVariant {
    GiantMantis,
    BloodIdol,
    Rattlesnake,
    GiantRattlesnake, // New CPU variant
    Raptor,
    TRex,
    VoidTempest,
    LightReaver,
    NightReaver,
    RazorFiend,
}

pub enum VisualEffect {
    FlickerStrike {
        from_x: f64,
        from_y: f64,
        to_x: f64,
        to_y: f64,
    },
    ShootPulseOrb {
        start_x: f64,
        start_y: f64,
        target_x: f64,
        target_y: f64,
    },
}

pub struct CpuUpdateResult {
    pub damage_to_player: Option<f64>,
    pub visual_effect: Option<VisualEffect>,
}

pub struct CpuEntity {
    pub x: f64,
    pub y: f64,
    pub speed: f64,
    pub size: f64,
    pub facing_left: bool,
    pub current_hp: f64,
    pub max_hp: f64,
    pub knockback_velocity: Vec2d,
    pub knockback_duration: f64,
    // Attack animation fields
    pub is_attacking: bool,
    pub attack_frame: usize,
    pub attack_timer: f64,
    pub attack_was_blocked: bool,
    pub block_sound_played: bool,
    pub frame_duration: f64,
    pub sound_effect_timer: f64,
    pub damage_value: f64,
    pub variant: CpuVariant, // To distinguish CPU types for textures/stats
    pub damage_display_cooldown: f64,
    pub stun_timer: f64,
    pub bleed_effect: Option<BleedEffect>,
    pub skill_manager: SkillManager,
}

impl CpuEntity {
    // Default constructor for GiantMantis
    pub fn new_giant_mantis(line_y: f64) -> Self {
        Self {
            x: safe_gen_range(50.0, 1870.0, "CpuEntity x (GiantMantis)"),
            y: safe_gen_range(line_y, line_y + 400.0, "CpuEntity y (GiantMantis)"),
            speed: 150.0,
            size: 10.0,
            facing_left: false,
            current_hp: 500.0,
            max_hp: 500.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 12.5,
            variant: CpuVariant::GiantMantis,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager: SkillManager::new(),
        }
    }

    // Constructor for razor_fiend in cpu_entity.rs
    pub fn new_razor_fiend(x: f64, y: f64) -> Self {
        let mut skill_manager = SkillManager::new();
        skill_manager.add_skill(SkillType::FlickerStrike);
        skill_manager.add_skill(SkillType::PulseOrb);
        Self {
            x,
            y,
            speed: 250.0,
            size: 10.0,
            facing_left: false,
            current_hp: 2500.0,
            max_hp: 2500.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 27.5,
            variant: CpuVariant::RazorFiend,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager,
        }
    }

    /// Constructor for LightReaver (uses Raptor stats)
    pub fn new_light_reaver(x: f64, y: f64) -> Self {
        let mut skill_manager = SkillManager::new();
        skill_manager.add_skill(SkillType::FlickerStrike);
        Self {
            x,
            y,
            speed: 250.0,
            size: 10.0,
            facing_left: false,
            current_hp: 300.0,
            max_hp: 300.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 17.5,
            variant: CpuVariant::LightReaver,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager,
        }
    }

    /// Constructor for NightReaver (uses Raptor stats)
    pub fn new_night_reaver(x: f64, y: f64) -> Self {
        let mut skill_manager = SkillManager::new();
        skill_manager.add_skill(SkillType::PulseOrb);
        Self {
            x,
            y,
            speed: 200.0,
            size: 10.0,
            facing_left: false,
            current_hp: 300.0,
            max_hp: 300.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 17.5,
            variant: CpuVariant::NightReaver,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager,
        }
    }

    // Constructor for BloodIdol
    pub fn new_blood_idol(line_y: f64, base_mantis_hp: f64, base_mantis_speed: f64) -> Self {
        Self {
            x: safe_gen_range(50.0, 1870.0, "CpuEntity x (BloodIdol)"),
            y: safe_gen_range(line_y, line_y + 400.0, "CpuEntity y (BloodIdol)"),
            speed: 450.0,
            size: 10.0,
            facing_left: false,
            current_hp: 750.0,
            max_hp: 750.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 18.75,
            variant: CpuVariant::BloodIdol,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager: SkillManager::new(),
        }
    }

    // Constructor for Rattlesnake
    pub fn new_rattlesnake(line_y: f64) -> Self {
        Self {
            x: safe_gen_range(50.0, 1870.0, "CpuEntity x (Rattlesnake)"),
            y: safe_gen_range(line_y, line_y + 400.0, "CpuEntity y (Rattlesnake)"),
            speed: 125.0,
            size: 10.0,
            facing_left: false,
            current_hp: 100.0,
            max_hp: 100.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 6.25,
            variant: CpuVariant::Rattlesnake,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager: SkillManager::new(),
        }
    }

    // Constructor for GiantRattlesnake
    pub fn new_giant_rattlesnake(line_y: f64) -> Self {
        Self {
            x: safe_gen_range(50.0, 1870.0, "CpuEntity x (GiantRattlesnake)"),
            y: safe_gen_range(line_y, line_y + 400.0, "CpuEntity y (GiantRattlesnake)"),
            speed: 100.0,
            size: 15.0, // Assuming slightly larger size, can be adjusted
            facing_left: false,
            current_hp: 650.0,
            max_hp: 650.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1, // Standard attack frame duration
            sound_effect_timer: 0.0,
            damage_value: 25.0,
            variant: CpuVariant::GiantRattlesnake,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager: SkillManager::new(),
        }
    }

    // Constructor for Raptor
    pub fn new_raptor(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            speed: 350.0,
            size: 10.0,
            facing_left: false,
            current_hp: 250.0,
            max_hp: 250.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 17.5,
            variant: CpuVariant::Raptor,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager: SkillManager::new(),
        }
    }

    // Constructor for T-Rex
    pub fn new_t_rex(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            speed: 350.0,
            size: 20.0, // T-Rex is bigger
            facing_left: false,
            current_hp: 1500.0,
            max_hp: 1500.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 35.0,
            variant: CpuVariant::TRex,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager: SkillManager::new(),
        }
    }

    /// Constructor for VoidTempest (identical to BloodIdol stats)
    pub fn new_void_tempest(line_y: f64, base_mantis_hp: f64, base_mantis_speed: f64) -> Self {
        let mut skill_manager = SkillManager::new(); // add skill 1/3
        skill_manager.add_skill(SkillType::FlickerStrike); // add skill 2/3
        Self {
            x: safe_gen_range(50.0, 1870.0, "CpuEntity x (VoidTempest)"),
            y: safe_gen_range(line_y, line_y + 400.0, "CpuEntity y (VoidTempest)"),
            speed: 450.0,
            size: 10.0,
            facing_left: false,
            current_hp: 750.0,
            max_hp: 750.0,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            is_attacking: false,
            attack_frame: 0,
            attack_timer: 0.0,
            attack_was_blocked: false,
            block_sound_played: false,
            frame_duration: 0.1,
            sound_effect_timer: 0.0,
            damage_value: 18.75,
            variant: CpuVariant::VoidTempest,
            damage_display_cooldown: 0.0,
            stun_timer: 0.0,
            bleed_effect: None,
            skill_manager, // add skill 3/3
        }
    }

    pub fn respawn(&mut self, line_y: f64) {
        let current_variant = self.variant;
        let base_mantis_hp_for_spirit_calc = 250.0;
        let base_mantis_speed_for_spirit_calc = 150.0;

        let new_entity = match current_variant {
            CpuVariant::GiantMantis => Self::new_giant_mantis(line_y),
            CpuVariant::BloodIdol => Self::new_blood_idol(
                line_y,
                base_mantis_hp_for_spirit_calc,
                base_mantis_speed_for_spirit_calc,
            ),
            CpuVariant::Rattlesnake => Self::new_rattlesnake(line_y),
            CpuVariant::GiantRattlesnake => Self::new_giant_rattlesnake(line_y),
            CpuVariant::Raptor => Self::new_raptor(
                safe_gen_range(MIN_X, MAX_X, "Raptor x"),
                safe_gen_range(line_y, MAX_Y, "Raptor y"),
            ),
            CpuVariant::TRex => Self::new_t_rex(
                safe_gen_range(MIN_X, MAX_X, "TRex x"),
                safe_gen_range(line_y, MAX_Y, "TRex y"),
            ),

            CpuVariant::VoidTempest => Self::new_void_tempest(
                line_y,
                base_mantis_hp_for_spirit_calc,
                base_mantis_speed_for_spirit_calc,
            ),
            CpuVariant::LightReaver => Self::new_light_reaver(
                safe_gen_range(MIN_X, MAX_X, "LightReaver x"),
                safe_gen_range(line_y, MAX_Y, "LightReaver y"),
            ),
            CpuVariant::NightReaver => Self::new_night_reaver(
                safe_gen_range(MIN_X, MAX_X, "NightReaver x"),
                safe_gen_range(line_y, MAX_Y, "NightReaver y"),
            ),
            CpuVariant::RazorFiend => Self::new_razor_fiend(
                safe_gen_range(
                    BUNKER_ORIGIN_X,
                    BUNKER_ORIGIN_X + BUNKER_WIDTH,
                    "RazorFiend x",
                ),
                safe_gen_range(
                    BUNKER_ORIGIN_Y,
                    BUNKER_ORIGIN_Y + BUNKER_HEIGHT,
                    "RazorFiend y",
                ),
            ),
        };
        *self = new_entity;
    }

    pub fn is_dead(&self) -> bool {
        self.current_hp <= 0.0
    }

    pub fn apply_knockback(&mut self, source_x: f64, source_y: f64, force: f64) {
        let dx = self.x - source_x;
        let dy = self.y - source_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0 {
            let normalized_dx = dx / distance;
            let normalized_dy = dy / distance;

            self.knockback_velocity = Vec2d::new(normalized_dx * force, normalized_dy * force);
            self.knockback_duration = 0.1;
        } else {
            // Edge case: source and entity are at the same position.
            // This can happen if RazorFiend flicker strikes to the player's location.
            // Apply knockback in a random direction to prevent immunity.
            let mut rng = rand::rng();
            let random_angle = rng.gen_range(0.0..std::f64::consts::TAU);
            let normalized_dx = random_angle.cos();
            let normalized_dy = random_angle.sin();

            self.knockback_velocity = Vec2d::new(normalized_dx * force, normalized_dy * force);
            self.knockback_duration = 0.1;
        }
    }

    pub fn update(
        &mut self,
        racer_x: f64,
        racer_y: f64,
        dt: f64,
        line_y: f64,
        audio_manager: &AudioManager,
    ) -> CpuUpdateResult {
        if !CPU_ENABLED {
            return CpuUpdateResult {
                damage_to_player: None,
                visual_effect: None,
            };
        }

        // Update bleed effect
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

        self.skill_manager.update(dt);

        // --- Stun logic is a complete override ---
        if self.stun_timer > 0.0 {
            self.stun_timer -= dt;

            // Reset attack state when stunned.
            self.is_attacking = false;
            self.attack_frame = 0;
            self.attack_timer = 0.0;
            self.attack_was_blocked = false;
            self.block_sound_played = false;
            self.sound_effect_timer = 0.0;

            // Process knockback movement if it's active.
            if self.knockback_duration > 0.0 {
                self.x += self.knockback_velocity.x * dt;
                self.y += self.knockback_velocity.y * dt;
                self.knockback_duration -= dt;

                if self.knockback_duration <= 0.0 {
                    self.knockback_velocity = Vec2d::new(0.0, 0.0);
                }
            }

            // Ensure position is clamped.
            self.x = self.x.max(MIN_X).min(MAX_X);
            self.y = self.y.max(MIN_Y).min(MAX_Y);

            // Skip all other logic for this frame.
            return CpuUpdateResult {
                damage_to_player: None,
                visual_effect: None,
            };
        }

        if self.damage_display_cooldown > 0.0 {
            self.damage_display_cooldown -= dt;
        }

        if self.is_attacking {
            self.attack_timer += dt;
            if self.attack_timer >= self.frame_duration {
                self.attack_timer = 0.0;
                self.attack_frame = (self.attack_frame + 1) % 3;
            }

            self.sound_effect_timer += dt;
            if self.sound_effect_timer >= 0.1 {
                self.sound_effect_timer = 0.0;

                if self.attack_was_blocked {
                } else {
                    let attack_sound = match self.variant {
                        CpuVariant::GiantMantis => "mantis_attack",
                        CpuVariant::BloodIdol => "mantis_attack",
                        CpuVariant::Rattlesnake => "mantis_attack", // Placeholder, add "rattlesnake_attack"
                        CpuVariant::GiantRattlesnake => "mantis_attack", // Placeholder, add "giant_rattlesnake_attack"
                        CpuVariant::Raptor => "mantis_attack",
                        CpuVariant::TRex => "mantis_attack",
                        CpuVariant::VoidTempest => "mantis_attack",
                        CpuVariant::LightReaver => "mantis_attack", // Placeholder
                        CpuVariant::NightReaver => "mantis_attack", // Placeholder
                        CpuVariant::RazorFiend => "mantis_attack",  // Placeholder
                    };
                    audio_manager
                        .play_sound_effect(attack_sound)
                        .unwrap_or_else(|e| {
                            println!("Failed to play {} sound: {}", attack_sound, e)
                        });
                }
            }
        } else {
            self.attack_was_blocked = false;
            self.sound_effect_timer = 0.0;
            self.attack_frame = 0;
        }

        // --- SKILL USAGE LOGIC ---
        if self.skill_manager.is_skill_ready(SkillType::FlickerStrike) {
            let dx = racer_x - self.x;
            let dy = racer_y - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= FLICKER_STRIKE_RADIUS {
                //println!("[SKILL] {:?} using Flicker Strike!", self.variant);

                let from_x = self.x;
                let from_y = self.y;

                // Teleport to player
                self.x = racer_x;
                self.y = racer_y;

                self.skill_manager.trigger_skill(SkillType::FlickerStrike);

                let damage = self.damage_value * FLICKER_STRIKE_DAMAGE_MULTIPLIER;
				
                audio_manager.play_sound_effect("death").unwrap_or_else(|e| {
                    //println!("[Audio] Failed to play flicker strike sound: {}", e)
                });				

                return CpuUpdateResult {
                    damage_to_player: Some(damage),
                    visual_effect: Some(VisualEffect::FlickerStrike {
                        from_x,
                        from_y,
                        to_x: self.x,
                        to_y: self.y,
                    }),
                };
            }
        }

        // Pulse Orb Skill Logic & range
        if self.skill_manager.is_skill_ready(SkillType::PulseOrb) {
            let dx = racer_x - self.x;
            let dy = racer_y - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Shoot if player is within range (e.g., 600) but not too close (e.g., 100)
            if distance <= 600.0 && distance >= 100.0 {
                self.skill_manager.trigger_skill(SkillType::PulseOrb);
                return CpuUpdateResult {
                    damage_to_player: None,
                    visual_effect: Some(VisualEffect::ShootPulseOrb {
                        start_x: self.x,
                        start_y: self.y,
                        target_x: racer_x,
                        target_y: racer_y,
                    }),
                };
            }
        }

        if self.knockback_duration > 0.0 {
            self.x += self.knockback_velocity.x * dt;
            self.y += self.knockback_velocity.y * dt;
            self.knockback_duration -= dt;

            if self.knockback_duration <= 0.0 {
                self.knockback_velocity = Vec2d::new(0.0, 0.0);
            }
        } else {
            let dx = racer_x - self.x;
            let dy = racer_y - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 {
                let norm_dx = dx / distance;
                let norm_dy = dy / distance;

                self.x += norm_dx * self.speed * dt;
                self.y += norm_dy * self.speed * dt;
                self.facing_left = norm_dx < 0.0;
            }
        }

        self.x = self.x.max(MIN_X).min(MAX_X);
        self.y = self.y.max(MIN_Y).min(MAX_Y);

        CpuUpdateResult {
            damage_to_player: None,
            visual_effect: None,
        }
    }

    pub fn draw(&self, context: Context, g: &mut G2d, textures: &[G2dTexture]) {
        if !CPU_ENABLED || textures.is_empty() {
            return;
        }

        let current_texture_index = if self.is_attacking {
            (self.attack_frame + 1).min(textures.len() - 1)
        } else {
            0
        };

        if current_texture_index >= textures.len() {
            println!(
                "Error: CPU texture index out of bounds for {:?}. Index: {}, Textures len: {}. Drawing placeholder or base.",
                self.variant, current_texture_index, textures.len()
            );
            if !textures.is_empty() {
                let _current_texture = &textures[0];
            } else {
                return;
            }
        }

        let current_texture = &textures[current_texture_index];

        let sprite_width = current_texture.get_width() as f64;
        let sprite_height = current_texture.get_height() as f64;

        let cpu_image_x = self.x - sprite_width / 2.0;
        let cpu_image_y = self.y - sprite_height / 2.0;

        if self.facing_left {
            let flip_transform = context
                .transform
                .trans(cpu_image_x + sprite_width, cpu_image_y)
                .scale(-1.0, 1.0);
            image(current_texture, flip_transform, g);
        } else {
            image(
                current_texture,
                context.transform.trans(cpu_image_x, cpu_image_y),
                g,
            );
        }

        let hp_bar_width = 50.0;
        let hp_bar_height = 5.0;
        let bar_y_offset = sprite_height / 2.0 + 15.0;
        let hp_bar_world_y = self.y - bar_y_offset;

        rectangle(
            [0.5, 0.5, 0.5, 1.0],
            [
                self.x - hp_bar_width / 2.0,
                hp_bar_world_y,
                hp_bar_width,
                hp_bar_height,
            ],
            context.transform,
            g,
        );

        let current_hp_width = (self.current_hp / self.max_hp).max(0.0) * hp_bar_width;
        rectangle(
            [1.0, 0.27, 0.0, 1.0], // red orange
            [
                self.x - hp_bar_width / 2.0,
                hp_bar_world_y,
                current_hp_width,
                hp_bar_height,
            ],
            context.transform,
            g,
        );

        // Draw bleed indicator
        if self.bleed_effect.is_some() {
            let bleed_indicator_y = hp_bar_world_y - 10.0;
            let bleed_text = "BLEED";
            // Simple visual indicator (you can replace with text rendering if glyphs are available)
            rectangle(
                [1.0, 0.0, 0.0, 0.8], // Red with transparency
                [self.x - 15.0, bleed_indicator_y, 5.0, 5.0],
                context.transform,
                g,
            );
        }

        // Draw stun indicator
        if self.stun_timer > 0.0 {
            let stun_indicator_y = hp_bar_world_y - 20.0;
            rectangle(
                [1.0, 1.0, 1.0, 0.8], // White with transparency
                [self.x - 15.0, stun_indicator_y, 5.0, 5.0],
                context.transform,
                g,
            );
        }
    }

    pub fn check_collision(
        &mut self,
        racer_x: f64,
        racer_y: f64,
        is_blocking: bool,
        _audio_manager: &AudioManager,
    ) -> bool {
        if !CPU_ENABLED {
            return false;
        }

        let collision_distance = self.size + crate::config::gameplay::COLLISION_THRESHOLD / 2.0;
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
}
