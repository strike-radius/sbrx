// File: src/entities/shoot.rs

use crate::config::CPU_ENABLED;
use crate::entities::cpu_entity::CpuEntity;
use crate::entities::fighter::Fighter;
use crate::utils::collision::check_line_collision;
use crate::DamageText;
use crate::RacerState;

/// Represents a ranged shooting attack
pub struct Shoot {
    pub x: f64,
    pub y: f64,
    pub visible: bool,
    pub timer: f64,
	#[allow(dead_code)] // TODO: Use for projectile rendering
    pub size: f64,
    pub line_visible: bool,
	#[allow(dead_code)] // TODO: Use for hit marker display
    pub marker_size: f64,
    pub start_x: f64,
    pub start_y: f64,
    pub target_x: f64,
    pub target_y: f64,
    pub active: bool,
    pub cooldown: f64,
}

impl Shoot {
    pub fn new(size: f64) -> Self {
        Shoot {
            x: 0.0,
            y: 0.0,
            visible: false,
            timer: 0.0,
            size,
            line_visible: false,
            marker_size: 20.0,
            start_x: 0.0,
            start_y: 0.0,
            target_x: 0.0,
            target_y: 0.0,
            active: false,
            cooldown: 0.0,
        }
    }

    pub fn trigger(&mut self, start_x: f64, start_y: f64, target_x: f64, target_y: f64) {
        // Only trigger if not in cooldown
        if self.cooldown <= 0.0 || true {
            // Always allow trigger, cooldown managed externally
            self.start_x = start_x;
            self.start_y = start_y;
            self.target_x = target_x;
            self.target_y = target_y;
            self.x = start_x;
            self.y = start_y;
            self.visible = true;
            self.line_visible = true;
            self.timer = 0.1;
            self.active = true;
            //self.cooldown = 0.50; // Set cooldown duration when shot is fired
            // Don't set cooldown for rapid fire - managed externally
        }
    }

    pub fn update(
        &mut self,
        dt: f64,
        cpu_entities: &mut Vec<CpuEntity>,
        fighter: &mut Fighter,
        damage_texts: &mut Vec<DamageText>,
        is_paused: bool,
    ) {
        // Timers always run, even when paused, to manage cooldowns and visual effects.
        if self.cooldown > 0.0 {
            self.cooldown -= dt;
        }

        if self.visible {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.visible = false;
                self.line_visible = false;
                self.active = false; // Also ensure the active state is reset
            }
        }

        // If paused, we skip all game-world interaction logic (collisions, damage, etc.).
        if is_paused {
            return;
        }

        if self.active && CPU_ENABLED {
            let mut hit_entity = false;
            for cpu_entity in cpu_entities.iter_mut() {
                if check_line_collision(
                    self.start_x,
                    self.start_y,
                    self.target_x,
                    self.target_y,
                    cpu_entity.x,
                    cpu_entity.y,
                ) {
                    let mut damage = fighter.ranged_damage;

                    // Apply damage reduction if on bike
                    if fighter.state == RacerState::OnBike {
                        damage *= 0.1; // 90% reduction
                    }

                    // Reduce damage by 50% for soldier rapid fire
                    if fighter.fighter_type == crate::game_state::FighterType::Soldier {
                        damage *= 0.50; // 50% reduction means 25% of original damage
                    }

                    cpu_entity.current_hp -= damage;

                    damage_texts.push(DamageText {
                        text: format!("{:.0}", damage),
                        x: cpu_entity.x,
                        y: cpu_entity.y - 50.0,
                        color: [1.0, 1.0, 1.0, 1.0], // White
                        lifetime: 0.25,
                    });

                    // Apply bleed effect for RACER only
                    if fighter.fighter_type == crate::game_state::FighterType::Racer {
                        use crate::entities::cpu_entity::BleedEffect;
                        cpu_entity.bleed_effect = Some(BleedEffect::new(50.0));

                        // Add bleed application text
                        damage_texts.push(DamageText {
                            text: "BLEED".to_string(),
                            x: cpu_entity.x,
                            y: cpu_entity.y - 70.0,
                            color: [1.0, 0.0, 0.0, 1.0], // Red
                            lifetime: 0.5,
                        });
                    }

                    // Knockback is now applied unconditionally if hit, death is handled in main loop
                    let knockback_force =
                        if fighter.fighter_type == crate::game_state::FighterType::Soldier {
                            -50.0
                        } else {
                            300.0
                        };
                    cpu_entity.apply_knockback(self.start_x, self.start_y, knockback_force);
                    hit_entity = true;
                    break; // Exit the loop after hitting one entity
                }
            }

            if hit_entity {
                self.active = false;
            }
        }
    }
}
