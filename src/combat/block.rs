// File: src/combat/block.rs

use crate::audio::AudioManager;
use crate::config::gameplay::COLLISION_THRESHOLD;
use crate::entities::cpu_entity::CpuEntity;
use crate::entities::fighter::Fighter;
use crate::entities::strike::Strike; // For strike visual
use crate::game_state::RacerState;
use crate::graphics::seven_segment::SevenSegmentDisplay;
use crate::DamageText;
use piston_window::*; // For strike collision check

// Base values for Kinetic Strike
pub const KINETIC_STRIKE_BASE_DAMAGE: f64 = 25.0;
pub const KINETIC_STRIKE_BASE_KNOCKBACK: f64 = 1000.0;

pub const KINETIC_STRIKE_DAMAGE_IMMUNITY_DURATION: f64 = 0.25;
pub const KINETIC_RUSH_BASE_DISTANCE_MULTIPLIER: f64 = 0.5; // Base multiplier for rush distance

// Effectiveness multipliers based on kinetic_intake_count (index i = kinetic_intake_count i)
// Index 0 is unused as KI must be > 0. Max index is 20.
pub const KINETIC_STRIKE_MULTIPLIERS: [f64; 21] = [
    0.0,  // KI = 0 (unused for strike)
    1.15, // KI = 1
    1.25, // KI = 2
    1.5,  // KI = 3
    1.75, // KI = 4
    2.0,  // KI = 5
    2.25, // KI = 6
    2.5,  // KI = 7
    2.75, // KI = 8
    3.0,  // KI = 9
    3.25, // KI = 10
    3.5,  // KI = 11
    3.75, // KI = 12
    4.0,  // KI = 13
    4.25, // KI = 14
    4.5,  // KI = 15
    4.75, // KI = 16
    5.0,  // KI = 17
    5.25, // KI = 18
    5.5,  // KI = 19
    5.75, // KI = 20
];

pub struct BlockSystem {
    pub active: bool,
    pub rmb_held: bool,
    pub block_count: i32,
    pub block_count_float: f64,
    pub max_block_count: i32,
    pub regen_timer: f64,
    pub stun_lock_timer: f64,
    pub vulnerability_timer: f64,
    pub fatigue_timer: f64,
    pub block_broken: bool,
    pub block_fatigue: bool,
    pub last_block_consumption_time: f64,
    pub min_time_between_blocks: f64,
    pub block_sound_timer: f64,
    pub block_sound_cooldown: f64,
    pub needs_dismount: bool,
    pub kinetic_intake_count: i32,
    pub kinetic_strike_damage_immunity_timer: f64,
    pub last_kinetic_strike_timer: f64,
}

impl BlockSystem {
    pub fn new(max_blocks: i32) -> Self {
        let initial_block_count = max_blocks;
        BlockSystem {
            active: false,
            rmb_held: false,
            block_count: initial_block_count,
            block_count_float: initial_block_count as f64,
            max_block_count: max_blocks,
            regen_timer: 0.0,
            stun_lock_timer: 0.0,
            vulnerability_timer: 0.0,
            fatigue_timer: 0.0,
            block_broken: false,
            block_fatigue: false,
            last_block_consumption_time: 0.0,
            min_time_between_blocks: 0.1,
            block_sound_timer: 0.0,
            block_sound_cooldown: 0.1,
            needs_dismount: false,
            kinetic_intake_count: max_blocks - initial_block_count,
            kinetic_strike_damage_immunity_timer: 0.0,
            last_kinetic_strike_timer: 0.0,
        }
    }

    fn update_kinetic_intake_count(&mut self) {
        self.kinetic_intake_count =
            (self.max_block_count - self.block_count).clamp(0, self.max_block_count);
    }

    pub fn is_kinetic_strike_active(&self) -> bool {
        // The kinetic strike is active in the brief animation period after it's triggered
        // This function will be called from the main render loop to determine the slash color
        self.kinetic_intake_count > 0
            && !self.is_stun_locked()
            && !self.block_broken
            && self.last_kinetic_strike_timer > 0.0
    }

    pub fn activate(&mut self, audio_manager: &AudioManager) -> bool {
        if !self.block_fatigue && !self.block_broken && !self.is_stun_locked() {
            audio_manager
                .play_sound_effect("raise_shield")
                .unwrap_or_else(|e| println!("Failed to play raise_shield sound: {}", e));
            self.active = true;
            self.rmb_held = true;
            self.regen_timer = 0.0;
            self.block_count_float = self.block_count as f64;

            println!(
                "[ACTIVATE_BLOCK] rmb_held: true, block_count: {}",
                self.block_count
            );
            true
        } else {
            false
        }
    }

    pub fn deactivate(&mut self) {
        let was_actively_blocking = self.active || self.rmb_held;
        self.rmb_held = false; // RMB released, even if block was not "active" due to fatigue
        self.active = false; // Stop active blocking state

        if was_actively_blocking // Only start regen delay if we were actually trying to block
            && !self.block_broken // and not if we are already broken
            && !self.block_fatigue // or fatigued
            && self.block_count < self.max_block_count
        {
            self.regen_timer = 1.25;
        }
    }

    pub fn update(&mut self, dt: f64, _current_time: f64) {
        if self.block_sound_timer > 0.0 {
            self.block_sound_timer -= dt;
        }

        let mut block_count_changed = false;

        // Update the kinetic strike timer
        if self.last_kinetic_strike_timer > 0.0 {
            self.last_kinetic_strike_timer -= dt;
        }
        // Update kinetic strike damage immunity timer
        if self.kinetic_strike_damage_immunity_timer > 0.0 {
            self.kinetic_strike_damage_immunity_timer -= dt;
        }

        if self.block_broken {
            // This state is set on block break, lasts through stun & vulnerability
            if self.stun_lock_timer > 0.0 {
                self.stun_lock_timer -= dt;
            }
            if self.vulnerability_timer > 0.0 {
                self.vulnerability_timer -= dt;
                if self.vulnerability_timer <= 0.0 {
                    // Transition from broken (stun/vuln) to fatigue
                    self.block_broken = false; // No longer "broken" in the sense of taking extra damage
                    self.block_fatigue = true; // Now in pure fatigue for recovery
                                               // fatigue_timer was set at the time of break/kinetic strike
                    println!("[RECOVERY] Vulnerability ended. Transitioning to block fatigue. Fatigue remaining: {:.2}s", self.fatigue_timer);
                }
            }
        } else if self.block_fatigue {
            self.fatigue_timer -= dt;
            if self.fatigue_timer <= 0.0 {
                self.block_fatigue = false;
                self.block_count = self.max_block_count;
                self.block_count_float = self.max_block_count as f64;
                self.regen_timer = 0.0;
                block_count_changed = true;
                println!("[RECOVERY] Block fatigue ended. Block points restored to max.");
            }
        }

        let can_regen_block_points = !self.rmb_held && !self.block_broken && !self.block_fatigue;
        if can_regen_block_points && self.block_count_float < self.max_block_count as f64 {
            if self.regen_timer > 0.0 {
                self.regen_timer -= dt;
            } else {
                let old_block_count_int = self.block_count;
                self.block_count_float += 5.0 * dt;
                self.block_count_float = self.block_count_float.min(self.max_block_count as f64);
                let new_block_count_int = self.block_count_float.floor() as i32;
                if new_block_count_int != old_block_count_int {
                    self.block_count = new_block_count_int;
                    block_count_changed = true;
                }
            }
        }

        if self.block_count_float >= self.max_block_count as f64
            && self.block_count != self.max_block_count
        {
            if self.block_count != self.max_block_count {
                self.block_count = self.max_block_count;
                block_count_changed = true;
            }
        }

        if block_count_changed {
            // Only update kinetic intake if not in any recovery state
            if !self.block_broken && !self.block_fatigue {
                self.update_kinetic_intake_count();
            }
            println!(
                "[SYNC] BlockCount: {}, KineticIntake: {}",
                self.block_count, self.kinetic_intake_count
            );
        }
    }

    pub fn process_attack(
        &mut self,
        fighter: &mut Fighter,
        cpu_entity: &mut CpuEntity,
        audio_manager: &AudioManager,
        current_time: f64,
    ) -> bool {
        if !self.active || self.block_broken || self.block_fatigue {
            // Cannot process if not active or already broken/fatigued
            return false;
        }
        cpu_entity.attack_was_blocked = true;

        let mut block_point_was_consumed_this_event = false;
        if current_time - self.last_block_consumption_time >= self.min_time_between_blocks {
            if self.block_count >= 0 {
                self.block_count -= 1;
                block_point_was_consumed_this_event = true;
                self.last_block_consumption_time = current_time;
            }
        }

        self.block_count_float = self.block_count.max(0) as f64;

        if self.block_count < 0 {
            self.block_count = 0;
            self.block_count_float = 0.0;
            self.kinetic_intake_count = 0;

            self.block_broken = true; // Enter broken state (stun/vulnerability)
            self.active = false;
            self.rmb_held = false;

            self.stun_lock_timer = 1.25;
            self.vulnerability_timer = self.stun_lock_timer + 2.5;
            self.fatigue_timer = 2.5; // This fatigue starts *after* vulnerability
            self.regen_timer = 1.25; // Delay for block points regen *after* full recovery cycle

            if fighter.state == RacerState::OnBike {
                self.needs_dismount = true;
            }
            audio_manager.play_sound_effect("block_break").ok();
            println!(
                "[BLOCK_BROKEN_NORMAL] BlockCount: 0, KineticIntake: {}. Stun/Vuln then Fatigue.",
                self.kinetic_intake_count
            );
            return false;
        } else {
            if block_point_was_consumed_this_event {
                self.update_kinetic_intake_count();
                println!(
                    "[BLOCK_CONSUMED] BlockCount: {}, KineticIntake: {}",
                    self.block_count, self.kinetic_intake_count
                );
            }

            if self.block_sound_timer <= 0.0 {
                audio_manager.play_sound_effect("block").ok();
                self.block_sound_timer = self.block_sound_cooldown;
            }
            return true;
        }
    }

    pub fn process_projectile_block(
        &mut self,
        fighter: &mut Fighter,
        audio_manager: &AudioManager,
        current_time: f64,
    ) -> bool {
        if !self.active || self.block_broken || self.block_fatigue {
            return false;
        }

        let mut block_point_was_consumed_this_event = false;
        if current_time - self.last_block_consumption_time >= self.min_time_between_blocks {
            if self.block_count >= 0 {
                self.block_count -= 1;
                block_point_was_consumed_this_event = true;
                self.last_block_consumption_time = current_time;
            }
        }

        self.block_count_float = self.block_count.max(0) as f64;

        if self.block_count < 0 {
            self.block_count = 0;
            self.block_count_float = 0.0;
            self.kinetic_intake_count = 0;

            self.block_broken = true;
            self.active = false;
            self.rmb_held = false;

            self.stun_lock_timer = 1.25;
            self.vulnerability_timer = self.stun_lock_timer + 2.5;
            self.fatigue_timer = 2.5;
            self.regen_timer = 1.25;

            if fighter.state == RacerState::OnBike {
                self.needs_dismount = true;
            }
            audio_manager.play_sound_effect("block_break").ok();
            println!("[PROJECTILE_BLOCK_BROKEN] Block broken by projectile.");
            return false;
        } else {
            if block_point_was_consumed_this_event {
                self.update_kinetic_intake_count();
                println!(
                    "[PROJECTILE_BLOCK_CONSUMED] KineticIntake: {}",
                    self.kinetic_intake_count
                );
            }

            if self.block_sound_timer <= 0.0 {
                audio_manager.play_sound_effect("block").ok();
                self.block_sound_timer = self.block_sound_cooldown;
            }
            return true;
        }
    }

    fn get_kinetic_strike_effectiveness_multiplier(&self) -> f64 {
        if self.kinetic_intake_count >= 1 && self.kinetic_intake_count <= 20 {
            KINETIC_STRIKE_MULTIPLIERS[self.kinetic_intake_count as usize]
        } else {
            0.0 // Should not happen if check kinetic_intake_count > 0 is done before calling
        }
    }

    pub fn perform_kinetic_strike(
        &mut self,
        world_x: f64,
        world_y: f64,
        fighter: &mut Fighter,
        cpu_entities: &mut Vec<CpuEntity>,
        strike_visual: &mut Strike,
        audio_manager: &AudioManager,
        line_y: f64, // For CPU respawn
        damage_texts: &mut Vec<DamageText>,
        task_system: &mut crate::task::TaskSystem,
        combo_system: &mut crate::combat::combo::ComboSystem,
		is_paused: bool,
    ) {
        if self.kinetic_intake_count == 0
            || self.is_stun_locked()
            || self.block_fatigue
            || self.block_broken
        {
            // Cannot perform kinetic strike if no charge or in recovery states
            return;
        }

        let ki_level_for_strike = self.kinetic_intake_count; // Store before reset
        let effectiveness_multiplier = self.get_kinetic_strike_effectiveness_multiplier();

        let strike_radius = if ki_level_for_strike <= 10 {
            COLLISION_THRESHOLD
        } else if ki_level_for_strike <= 17 {
            COLLISION_THRESHOLD * 3.0
        } else {
            COLLISION_THRESHOLD * 5.0
        };

        println!(
            "[KINETIC_STRIKE] Performing! KI: {}, Multiplier: {:.2}, Radius: {:.1}",
            ki_level_for_strike, effectiveness_multiplier, strike_radius
        );

        // Grant temporary invincibility to the player
        fighter.invincible_timer = KINETIC_STRIKE_DAMAGE_IMMUNITY_DURATION;
        println!(
            "[KINETIC_STRIKE_IMMUNITY] player granted {:.2}s immunity.",
            KINETIC_STRIKE_DAMAGE_IMMUNITY_DURATION
        );

        // Play a special sound? For now, use melee.
        audio_manager.play_sound_effect("death").ok(); // kinetic strike sounds effect
        strike_visual.trigger(world_x, world_y); // Use existing strike visual

        // Apply damage and knockback to CPUs
        if crate::config::CPU_ENABLED && !is_paused {
            for cpu in cpu_entities.iter_mut() {
                let strike_dx = world_x - cpu.x;
                let strike_dy = world_y - cpu.y;
                if (strike_dx * strike_dx + strike_dy * strike_dy).sqrt() < strike_radius {
                    let damage = fighter.melee_damage * effectiveness_multiplier; //kinetic_strike damage output
                    let knockback_force = KINETIC_STRIKE_BASE_KNOCKBACK * effectiveness_multiplier;

                    cpu.current_hp -= damage;
                    println!(
                        "[KINETIC_STRIKE_HIT] CPU HP: {:.1}, Damage: {:.1}",
                        cpu.current_hp, damage
                    );

                    // Add damage text for kinetic strike
                    damage_texts.push(DamageText {
                        text: format!("{:.0}", damage),
                        x: cpu.x,
                        y: cpu.y - 50.0,
                        color: [0.0, 1.0, 0.0, 1.0], // Green
                        lifetime: 0.25,
                    });

                    // Knockback is applied only if the target survives.
                    // Death processing is handled in the main loop for consistency.
                    if !cpu.is_dead() {
                        cpu.apply_knockback(world_x, world_y, knockback_force);
                    }
                }
            }
        }

        // Set the kinetic strike timer
        self.last_kinetic_strike_timer = KINETIC_STRIKE_DAMAGE_IMMUNITY_DURATION; // Match strike animation duration

        // Consequences of Kinetic Strike:
        // 1. Start the combo system timer for the next combo sequence.
        combo_system.start_timer_after_kinetic_strike();

        // 2. Reset block/fatigue state.
        self.block_count = 0;
        self.block_count_float = 0.0;
        self.update_kinetic_intake_count(); // This will set KI to max_block_count (e.g., 20) for UI

        self.active = false;
        self.rmb_held = false; // Release block hold
        self.block_broken = false; // Not "broken" in stun/vuln sense, directly to fatigue
        self.block_fatigue = true;
        self.fatigue_timer = 2.5; // Standard fatigue duration
        self.regen_timer = 1.25; // Regen delay starts after fatigue ends

        println!(
            "[KINETIC_STRIKE_POST] BlockCount: 0, KineticIntake (UI): {}. Fatigue started. Combo timer initiated.",
            self.kinetic_intake_count
        );
    }

    // Check if the player is immune to damage due to a recent kinetic strike
    pub fn is_immune_to_damage(&self) -> bool {
        self.kinetic_strike_damage_immunity_timer > 0.0
    }

    pub fn get_damage_multiplier(&self) -> f64 {
        // For incoming damage to player
        if self.block_broken && self.vulnerability_timer > 0.0 {
            // Only during actual vulnerability phase of a block break
            1.5
        } else {
            1.0
        }
    }

    pub fn is_stun_locked(&self) -> bool {
        // Player cannot act
        self.block_broken && self.stun_lock_timer > 0.0
    }

    pub fn draw_ui(&self, context: Context, g: &mut G2d) {
        // Status Text (Stunned/Vulnerable/Fatigue)
        if self.is_stun_locked() {
            // Stun has priority for text
            draw_status_text_placeholder(
                "STUNNED",
                [1.0, 0.0, 0.0, 1.0],
                20.0,  // bar_x
                465.0, // bar_y_block_bar - 15.0
                context,
                g,
            );
        } else if self.block_broken && self.vulnerability_timer > 0.0 {
            // Only shows if not stunned
            draw_status_text_placeholder(
                "VULNERABLE",
                [1.0, 0.5, 0.0, 1.0],
                20.0,  // bar_x
                465.0, // bar_y_block_bar - 15.0
                context,
                g,
            );
        } else if self.block_fatigue {
            // Only shows if not stunned or vulnerable
            draw_status_text_placeholder(
                "FATIGUE",
                [0.7, 0.3, 0.0, 1.0],
                20.0,  // bar_x
                465.0, // bar_y_block_bar - 15.0
                context,
                g,
            );
        }
    }
}

fn draw_status_text_placeholder(
    text: &str,
    color: [f32; 4],
    x: f64,
    y: f64,
    context: Context,
    g: &mut G2d,
) {
    let _ = text;
}
