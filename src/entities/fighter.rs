//File: fighter.rs

use crate::combat::stats::HUNTER_LVL1_STATS;
use crate::combat::stats::{Stats, RACER_LVL1_STATS, SOLDIER_LVL1_STATS};
use crate::config::boundaries::{MAX_X, MAX_Y, MIN_X, MIN_Y};
use crate::game_state::{FighterType, RacerState};
use crate::graphics::seven_segment::SevenSegmentDisplay;
use crate::mechanics::lvl_up::PROGRESSION_POINTS;
use crate::stats;
use crate::utils::vec2d::Vec2d;
use crate::CombatMode;
use piston_window::*;

use crate::HashMap;

pub struct Fighter {
    pub x: f64,
    pub y: f64,
    pub current_hp: f64,
    pub max_hp: f64,
    pub knockback_velocity: Vec2d,
    pub knockback_duration: f64,
    pub score: u32,
    pub invincible_timer: f64,
    pub state: RacerState,
    pub fighter_type: FighterType,
    pub stats: Stats,
    pub melee_damage: f64,
    pub ranged_damage: f64,
    pub run_speed: f64,
    pub fuel: f64,
    pub max_fuel: f64,
    pub fuel_tanks: HashMap<FighterType, f64>,
    pub combat_action_slowdown_timer: f64,
    pub combat_mode: CombatMode,
    pub kill_counters: HashMap<FighterType, u32>,
    pub levels: HashMap<FighterType, u32>,
    pub stat_points_to_spend: HashMap<FighterType, u32>,
    pub ammo: u32,
    pub max_ammo: u32,
    pub is_reloading: bool,
    pub reload_timer: f64,
	pub boost: bool,
	pub bike_boost_toggle_cooldown: f64,
	pub boost_indicator_timer: f64,
	pub show_gear: bool,
}

impl Fighter {
    pub fn new(x: f64, y: f64) -> Self {
        let initial_stats = RACER_LVL1_STATS;
        let mut fuel_tanks = HashMap::new();
        fuel_tanks.insert(FighterType::Racer, 100.0);
        fuel_tanks.insert(FighterType::Soldier, 100.0);
        fuel_tanks.insert(FighterType::Hunter, 100.0);

        let mut kill_counters = HashMap::new();
        kill_counters.insert(FighterType::Racer, 0);
        kill_counters.insert(FighterType::Soldier, 0);
        kill_counters.insert(FighterType::Hunter, 0);

        let mut levels = HashMap::new();
        levels.insert(FighterType::Racer, 1);
        levels.insert(FighterType::Soldier, 1);
        levels.insert(FighterType::Hunter, 1);

        let mut stat_points_to_spend = HashMap::new();
        stat_points_to_spend.insert(FighterType::Racer, 0);
        stat_points_to_spend.insert(FighterType::Soldier, 0);
        stat_points_to_spend.insert(FighterType::Hunter, 0);

        Fighter {
            x,
            y,
            current_hp: initial_stats.defense.hp,
            max_hp: initial_stats.defense.hp,
            knockback_velocity: Vec2d::new(0.0, 0.0),
            knockback_duration: 0.0,
            score: 0,
            invincible_timer: 0.0,
            state: RacerState::OnFoot,
            fighter_type: FighterType::Racer,
            stats: initial_stats,
            melee_damage: initial_stats.attack.melee_damage,
            ranged_damage: initial_stats.attack.ranged_damage,
            run_speed: initial_stats.speed.run_speed,
            fuel: 100.0,
            max_fuel: 100.0,
            fuel_tanks,
            combat_action_slowdown_timer: 0.0,
            combat_mode: CombatMode::CloseCombat,
            kill_counters,
            levels,
            stat_points_to_spend,
            ammo: 25,
            max_ammo: 25,
            is_reloading: false,
            reload_timer: 0.0,
			boost: true, // false to swap starting [SHIFT] key
			bike_boost_toggle_cooldown: 0.0,
			boost_indicator_timer: 0.0,
			show_gear: false,
        }
    }

    pub fn switch_fighter_type(&mut self, fighter_type: FighterType) -> f64 {
        // Store current fuel before switching
        self.fuel_tanks.insert(self.fighter_type, self.fuel);

        self.fighter_type = fighter_type;

        let (new_stats, new_radius) = match self.fighter_type {
            FighterType::Racer => (RACER_LVL1_STATS, 125.0),
            FighterType::Soldier => (SOLDIER_LVL1_STATS, 100.0),
            FighterType::Hunter => (HUNTER_LVL1_STATS, 150.0),
        };

        // Store the new stats object
        self.stats = new_stats;

        // Update derived fighter parameters
        self.max_hp = new_stats.defense.hp;
        // self.current_hp = self.max_hp; // BUG FIX: This line is removed to preserve HP across switches.
        self.melee_damage = new_stats.attack.melee_damage;
        self.ranged_damage = new_stats.attack.ranged_damage;
        self.run_speed = new_stats.speed.run_speed;

        // Load new fighter's fuel
        self.fuel = *self.fuel_tanks.get(&self.fighter_type).unwrap_or(&100.0);
        new_radius
    }

    // --- MODIFIED: Draws the fuel meter as a numerical percentage ---
    pub fn draw_fuel_meter(&self, original_context: Context, g: &mut G2d, glyphs: &mut Glyphs) {
        // Calculate the fuel percentage
        let fuel_percentage = (self.fuel / self.max_fuel) * 100.0;
        // Format the text string, ensuring it doesn't go below 0.0
        let fuel_text = format!("FUEL: {:.1}%", fuel_percentage.max(0.0));

        let font_size = 20;
        let text_color = [1.0, 0.5, 0.0, 1.0]; // Orange color for fuel
        let text_x = 80.0; // Position near other UI elements
        let text_y = 280.0; // FUEL METER

        // Draw the text
        text::Text::new_color(text_color, font_size)
            .draw(
                &fuel_text,
                glyphs,
                &original_context.draw_state,
                original_context.transform.trans(text_x, text_y),
                g,
            )
            .unwrap_or_else(|e| eprintln!("Failed to draw fuel text: {}", e));
    }

    pub fn draw_stats_display(
        &self,
        original_context: Context,
        g: &mut G2d,
        glyphs: &mut Glyphs,
        level_modifier: i32,
    ) {
        let lime_green = [0.0, 1.0, 0.0, 1.0]; // Lime Green

        let base_x = 20.0;
        let base_y = 125.0; // Position below other UI
        let stats_font_size = 25; // Font size for DEF, ATK, SPD
        let fighter_type_font_size = 17; // Font size for [RACER] / [SOLDIER]
        let line_height = stats_font_size as f64 + 5.0;

        let def = (self.stats.defense.hp / stats::HP_PER_DEFENSE_POINT).round() as u32;
        let atk = (self.stats.attack.melee_damage / stats::DAMAGE_PER_ATTACK_POINT).round() as u32;
        let spd = (self.stats.speed.run_speed / stats::SPEED_PER_SPEED_POINT).round() as u32;
        let base_level = *self.levels.get(&self.fighter_type).unwrap_or(&1) as i32;
        let effective_level = (base_level + level_modifier).max(1); // Ensure level doesn't go below 1
        let fighter_type_text = match self.fighter_type {
            FighterType::Racer => format!("[RACER]{}", effective_level),
            FighterType::Soldier => format!("[SOLDIER]{}", effective_level),
            FighterType::Hunter => format!("[HUNTER]{}", effective_level),
        };

        let def_text = format!("DEF: {}", def);
        let atk_text = format!("ATK: {}", atk);
        let spd_text = format!("SPD: {}", spd);

        // To keep the background size fixed to the RACER display, we calculate the width
        // based on the texts that appear for the Racer, ignoring the longer Soldier text.
        let racer_type_text_for_calc = "[RACER]15";
        // Note: All font sizes in this calculation are assumed to be the same (25) for simplicity.
        // If they were different, a more complex width calculation would be needed.
        let texts_for_width_calc: [&str; 4] =
            [&def_text, &atk_text, &spd_text, racer_type_text_for_calc];

        let max_width = texts_for_width_calc
            .iter()
            .map(|t| glyphs.width(stats_font_size, t).unwrap_or(0.0))
            .fold(0.0, f64::max);

        let bg_width = max_width + 10.0;
        let bg_height = (line_height * 4.0) + 5.0;

        rectangle(
            [0.0, 0.0, 0.0, 0.7],
            [base_x - 5.0, base_y - 5.0, bg_width, bg_height],
            original_context.transform,
            g,
        );

        // Draw DEF, ATK, SPD with their own font size
        text::Text::new_color(lime_green, stats_font_size)
            .draw(
                &def_text,
                glyphs,
                &original_context.draw_state,
                original_context
                    .transform
                    .trans(base_x, base_y + stats_font_size as f64),
                g,
            )
            .ok();
        text::Text::new_color(lime_green, stats_font_size)
            .draw(
                &atk_text,
                glyphs,
                &original_context.draw_state,
                original_context
                    .transform
                    .trans(base_x, base_y + line_height + stats_font_size as f64),
                g,
            )
            .ok();
        text::Text::new_color(lime_green, stats_font_size)
            .draw(
                &spd_text,
                glyphs,
                &original_context.draw_state,
                original_context
                    .transform
                    .trans(base_x, base_y + line_height * 2.0 + stats_font_size as f64),
                g,
            )
            .ok();

        // Draw the fighter type text with its specific font size
        text::Text::new_color(lime_green, fighter_type_font_size)
            .draw(
                &fighter_type_text,
                glyphs,
                &original_context.draw_state,
                original_context.transform.trans(
                    base_x,
                    base_y + line_height * 3.0 + fighter_type_font_size as f64,
                ),
                g,
            )
            .ok();
    }

    pub fn draw_inputs_display(
        &self,
        original_context: Context,
        g: &mut G2d,
        inputs_texture: &G2dTexture,
    ) {
        // Calculate inputs.png position
        let base_x = 600.0;
        let base_y = 1075.0;

        // Get texture dimensions
        let texture_height = inputs_texture.get_height() as f64;

        // Draw the inputs image at fixed position - using original_context for fixed screen position
        image(
            inputs_texture,
            original_context
                .transform
                .trans(base_x, base_y - texture_height),
            g,
        );
    }

    pub fn draw_bike_interaction_indicator(
        &self,
        original_context: Context,
        g: &mut G2d,
        bike_x: f64,
        bike_y: f64,
        visible: bool,
    ) {
        // Only show if the bike is visible and we're on foot
        if !visible || self.state != RacerState::OnFoot {
            return;
        }

        // Calculate distance to bike
        let dx = self.x - bike_x;
        let dy = self.y - bike_y;
        let distance_to_bike = (dx * dx + dy * dy).sqrt();

        // Only show indicator if within interaction range
        let bike_interaction_distance = 100.0;
        if distance_to_bike <= bike_interaction_distance {
            // Create smaller seven segment display for the 'V' indicator
            let display = SevenSegmentDisplay::new(15.0, 25.0, 5.0);

            // Fixed position on screen instead of above the bike
            let indicator_x = 1125.0; // Fixed X position
            let indicator_y = 600.0; // Fixed Y position

            // Draw background
            rectangle(
                [0.0, 0.0, 0.0, 0.7], // Semi-transparent black background
                [
                    indicator_x - 15.0,
                    indicator_y - 5.0,
                    display.segment_width + 10.0,
                    display.segment_height + 10.0,
                ],
                original_context.transform,
                g,
            );

            // Draw the letter 'V' manually using lines/polygons since 7-segment can't do diagonals well
            let f_color = [0.0, 1.0, 0.0, 1.0]; // Bright green
            let line_width = 2.0;

            let left_top = [indicator_x, indicator_y];
            let bottom_center = [indicator_x + display.segment_width / 2.0, indicator_y + display.segment_height];
            let right_top = [indicator_x + display.segment_width, indicator_y];

            // Draw left diagonal line
            line(
                f_color,
                line_width,
                [left_top[0], left_top[1], bottom_center[0], bottom_center[1]],
                original_context.transform,
                g,
            );

            // Draw right diagonal line
            line(
                f_color,
                line_width,
                [right_top[0], right_top[1], bottom_center[0], bottom_center[1]],
                original_context.transform,
                g,
            );
        }
    }

    pub fn draw_score(&self, _transformed_context: Context, original_context: Context, g: &mut G2d) {
        // Create seven segment display with appropriate dimensions
        let display = SevenSegmentDisplay::new(20.0, 30.0, 7.5); // 50% of previous size (was 40.0, 60.0, 15.0)

        // Calculate total width of display (3 digits plus spacing)
        let display_width = (display.segment_width + display.spacing) * 3.0;

        // Fixed position for the score (bottom left corner)
        let base_x = 20.0; // Small margin from left edge
        let base_y = 300.0; // Small margin from bottom

        // Draw background for score - using original_context for fixed screen position
        rectangle(
            [0.0, 0.0, 0.0, 0.7], // Darker background for better contrast
            [
                base_x - 10.0,
                base_y - 5.0,
                display_width + 20.0,
                display.segment_height + 10.0,
            ],
            original_context.transform,
            g,
        );

        // Format score with leading zeros and draw each digit
        let kill_count = self.kill_counters.get(&self.fighter_type).unwrap_or(&0);
        let score_str = format!("{:03}", kill_count.min(&999));
        let digits: Vec<u32> = score_str
            .chars()
            .map(|c| c.to_digit(10).unwrap_or(0))
            .collect();

        // Draw each digit - using original_context for fixed screen position
        for (i, &digit) in digits.iter().enumerate() {
            let digit_x = base_x + (display.segment_width + display.spacing) * i as f64;
            display.draw_digit(
                digit,
                digit_x,
                base_y,
                [0.0, 1.0, 0.0, 1.0], // Bright green color
                original_context,
                g,
            );
        }

        // --- NEW XP BAR LOGIC ---
        // Position the bar right below the numbers
        let bar_y = base_y + display.segment_height + 2.0;
        let bar_width = display_width;
        let bar_height = 10.0;

        // Draw bar background
        rectangle(
            [0.3, 0.3, 0.3, 0.7], // Grey background
            [base_x, bar_y, bar_width, bar_height],
            original_context.transform,
            g,
        );

        // Calculate XP progress
        let current_kills = *self.kill_counters.get(&self.fighter_type).unwrap_or(&0) as f64;
        let current_level = *self.levels.get(&self.fighter_type).unwrap_or(&1) as usize;

        let xp_percentage = if current_level > 0 && current_level - 1 < PROGRESSION_POINTS.len() {
            let kills_needed = PROGRESSION_POINTS[current_level - 1] as f64;
            if kills_needed > 0.0 {
                (current_kills / kills_needed).min(1.0)
            } else {
                1.0 // If kills needed is 0, bar is full (e.g., max level placeholder)
            }
        } else {
            1.0 // Max level, show full bar
        };

        let current_xp_bar_width = xp_percentage * bar_width;

        rectangle(
            [0.0, 0.6, 1.0, 1.0], // Blue color
            [base_x, bar_y, current_xp_bar_width, bar_height],
            original_context.transform,
            g,
        );
    }
/*
    pub fn apply_knockback(&mut self, source_x: f64, source_y: f64, force: f64) {
        // Calculate direction from source to fighter
        let dx = self.x - source_x;
        let dy = self.y - source_y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0 {
            // Normalize and apply force
            let normalized_dx = dx / distance;
            let normalized_dy = dy / distance;

            self.knockback_velocity = Vec2d::new(normalized_dx * force, normalized_dy * force);
            self.knockback_duration = 0.1; // Knockback lasts 0.1 seconds
        }
    }
*/
    pub fn update(&mut self, dt: f64, _line_y: f64) {
        if self.knockback_duration > 0.0 {
            // Apply knockback movement
            self.x += self.knockback_velocity.x * dt;
            self.y += self.knockback_velocity.y * dt;

            // Constrain to game bounds - 1920x1080
            self.x = self.x.max(MIN_X).min(MAX_X);
            self.y = self.y.max(MIN_Y).min(MAX_Y);

            // Reduce knockback duration
            self.knockback_duration -= dt;

            // Clear knockback when duration expires
            if self.knockback_duration <= 0.0 {
                self.knockback_velocity = Vec2d::new(0.0, 0.0);
            }
        }

        // Update invincibility timer
        if self.invincible_timer > 0.0 {
            self.invincible_timer -= dt;
            if self.invincible_timer < 0.0 {
                self.invincible_timer = 0.0; // Ensure it doesn't go negative
            }
        }
        // Update combat action slowdown timer
        if self.combat_action_slowdown_timer > 0.0 {
            self.combat_action_slowdown_timer -= dt;
            if self.combat_action_slowdown_timer < 0.0 {
                self.combat_action_slowdown_timer = 0.0;
            }
        }

        if self.is_reloading {
            self.reload_timer -= dt;
            if self.reload_timer <= 0.0 {
                self.is_reloading = false;
                self.ammo = self.max_ammo;
            }
        }
        if self.bike_boost_toggle_cooldown > 0.0 {
            self.bike_boost_toggle_cooldown -= dt;
        }	
        if self.boost_indicator_timer > 0.0 {
            self.boost_indicator_timer -= dt;
        }		
    }

    pub fn draw_health_bar(&self, context: Context, g: &mut G2d) {
        let hp_bar_width = 50.0;
        let hp_bar_height = 5.0;

        // Position the health bar above the racer sprite
        let bar_y = self.y - 80.0; // Adjust this value to position the bar higher/lower

        // Draw background (grey) bar
        rectangle(
            [0.5, 0.5, 0.5, 1.0],
            [
                self.x - hp_bar_width / 2.5,
                bar_y,
                hp_bar_width,
                hp_bar_height,
            ],
            context.transform,
            g,
        );

        // Draw current HP (green) bar
        let current_width = (self.current_hp / self.max_hp) * hp_bar_width;
        rectangle(
            [0.0, 1.0, 0.0, 1.0],
            [
                self.x - hp_bar_width / 2.5,
                bar_y,
                current_width.max(0.0), // Ensure width doesn't go negative
                hp_bar_height,
            ],
            context.transform,
            g,
        );

        // Draw HP text
        let _hp_text = format!("{}/{}", self.current_hp as i32, self.max_hp as i32);
        // Add text drawing here if you want to display the numbers
    }

    pub fn trigger_reload(&mut self, audio_manager: &crate::audio::AudioManager) {
        if !self.is_reloading {
            self.is_reloading = true;
            self.reload_timer = 1.0;
            audio_manager.play_sound_effect("reload").ok();
        }
    }

    pub fn draw_ammo_gauge(&self, c: Context, g: &mut G2d, glyphs: &mut Glyphs) {
        if self.fighter_type != crate::game_state::FighterType::Soldier {
            return;
        }

        let font_size = 15;
        let text_color = if self.is_reloading {
            [1.0, 0.0, 0.0, 1.0]
        } else {
            [1.0, 0.5, 0.0, 1.0]
        };
        let bar_char = "|";
        let bars = bar_char.repeat(self.ammo as usize);
        let display_text = if self.is_reloading {
            "RELOADING...".to_string()
        } else {
            format!("AMMO: {}", bars)
        };

        let x = 1.0;
        let y = 50.0; // Positioned below score

        // Background
        rectangle(
            [0.0, 0.0, 0.0, 0.9],
            [x - 5.0, y - 25.0, 350.0, 30.0],
            c.transform,
            g,
        );

        text::Text::new_color(text_color, font_size)
            .draw(
                &display_text,
                glyphs,
                &c.draw_state,
                c.transform.trans(x, y),
                g,
            )
            .ok();
    }
}