// Copyright (C) 2025 IJB <strike_radius@protonmail.com>
//
// This file is part of sbrx.
//
// sbrx is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of
// the License, or (at your option) any later version.
//
// sbrx is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with sbrx. If not, see <https://www.gnu.org/licenses/>.

//File: src/main.rs

//#![allow(deprecated)]
//#![allow(unused_imports)]
//#![allow(unused_variables)]
//#![allow(unused_mut)]
//#![allow(dead_code)]

mod audio;
mod combat;
mod config;
mod entities;
mod game_state;
mod graphics;
mod map_system;
mod utils;
mod game {
    pub mod input_handler;
}
mod area;
mod chatbox;
mod fog_of_war;
mod mechanics;
mod task;
mod vehicle; 

// Crates
extern crate find_folder;
extern crate firmament_lib;
extern crate piston_window;
extern crate rand;
extern crate rodio;

use crate::entities::collision_barriers::{CollisionBarrierManager, JumpZoneType};

use crate::game_state::AmbientTrackState;

use crate::combat::block::KINETIC_STRIKE_MULTIPLIERS;
use crate::combat::block::KINETIC_RUSH_BASE_DISTANCE_MULTIPLIER;
use crate::combat::block::KINETIC_STRIKE_DAMAGE_IMMUNITY_DURATION;

use crate::piston_window::MouseCursorEvent;
use crate::combat::block::BlockSystem;
use crate::combat::combo::ComboSystem;
use crate::combat::field_traits::{FieldTraitManager, StatAttribute, TraitTarget};
use crate::combat::stats;
use piston_window::Image;
// NEW: Import stats constants for group UI
use crate::combat::stats::{RAPTOR_LVL1_STATS, RACER_LVL1_STATS, SOLDIER_LVL1_STATS};
use crate::piston_window::MouseScrollEvent;
use graphics::camera::{screen_to_world, Camera};
use graphics::crater::draw_crater;
use graphics::fighter_textures::{
    is_high_priority_animation_active, load_fighter_textures, update_current_textures,
};

use crate::entities::cpu_entity::VisualEffect;
use crate::game_state::{
    CombatMode, DeathType, FighterType, GameState, MovementDirection, RacerState,
};
use crate::vehicle::fighter_jet::FighterJet;
use entities::cpu_entity::{CpuEntity, CpuVariant};
use entities::fighter::Fighter;
use entities::fixed_crater::FixedCrater;
use entities::fuel_pump::FuelPump;
use entities::moving_sphere::MovingSphere;
use entities::pulse_orb::PulseOrb;
//use entities::pyramid::{generate_border_pyramids, Pyramid};
//use utils::animation_queue::AnimationQueue;
use entities::raptor_nest::RaptorNest;
use entities::sbrx_bike::SbrxBike;
use entities::shoot::Shoot;
use entities::star::Star;
use entities::strike::Strike;
use entities::track::Track;
use audio::AudioManager;
use chatbox::{ChatBox, MessageType}; // Import the new ChatBox system
use crate::map_system::{FieldId as SbrxFieldId, MapSystem as SbrxMapSystem};
use utils::collision::check_line_collision;
use utils::math::safe_gen_range;
use utils::vec2d::Vec2d;
use config::gameplay::{
    BIKE_INTERACTION_DISTANCE, COLLISION_THRESHOLD,
    RAPTOR_NEST_INTERACTION_DISTANCE, FIGHTER_JET_INTERACTION_DISTANCE,
};
use config::movement::{
    BIKE_SPEED, MOVEMENT_BUFFER_DURATION, ON_FOOT_HOLD_DURATION,
    RUSH_DURATION,
};
use config::resolution::{HEIGHT, HORIZON_LINE, WIDTH};
use config::{
    boundaries::{MAX_X, MAX_Y, MIN_X, MIN_Y},
    FOG_OF_WAR_ENABLED,
};
use config::{CPU_ENABLED, PERFORMANCE_MODE};
use piston_window::{
    clear, ellipse, image, line, polygon, rectangle, text, AdvancedWindow, 
	Button, CharacterCache, Flip, G2dTexture, G2dTextureContext,
    ImageSize, Key, MouseButton, PistonWindow, PressEvent,
    ReleaseEvent, RenderEvent, Texture, TextureSettings, Transformed,
    UpdateEvent, Window, WindowSettings,
};
use rand::Rng;
use rodio::Sink;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};
use crate::area::area::{AreaState, AREA_HEIGHT, AREA_ORIGIN_X, AREA_ORIGIN_Y, AREA_WIDTH};
use crate::area::area::{AreaType, BUNKER_HEIGHT, BUNKER_ORIGIN_X, BUNKER_ORIGIN_Y, BUNKER_WIDTH};
use crate::entities::ground_assets::GroundAssetManager;
use crate::fog_of_war::FogOfWar;
use crate::mechanics::wave::WaveManager;
use crate::task::TaskSystem;

const PARTICLE_COUNT_CPU: usize = 5;
const FUEL_DEPLETION_RATE: f64 = 1.0; // move to fighter.rs
const FUEL_REPLENISH_AMOUNT: f64 = 1.0; // move to fighter.rs
const RACER_RANGED_COOLDOWN: f64 = 0.5; // move to fighter.rs
const BOUNDARY_WARNING_COOLDOWN_TIME: f64 = 3.0;
const DEFAULT_FIGHTER_JET_WORLD_X: f64 = MIN_X + (MAX_X - MIN_X) / 2.0;
const DEFAULT_FIGHTER_JET_WORLD_Y: f64 = MIN_Y + (MAX_Y - MIN_Y) / 4.0;
const BIKE_ACCELERATE_FRAME_DURATION: f64 = 0.08;
const RAPTOR_INTERACTION_DISTANCE: f64 = 150.0;
const INFO_POST_INTERACTION_DISTANCE: f64 = 150.0;
const SOLDIER_RAPID_FIRE_RATE: f64 = 0.09; // Match CPU entity attack rate
const MELEE_RAPID_FIRE_RATE: f64 = 0.125; // Match CPU entity damage application rate
const ESC_HOLD_DURATION_TO_EXIT: f64 = 3.0;
const DEATH_SCREEN_COOLDOWN_TIME: f64 = 0.5;
const DEMO_END_ZONE: [f64; 4] = [0.0, 500.0, 2750.0, 3250.0]; // min_x, max_x, min_y, max_y
const RACETRACK_SPAWN_POINT: (f64, f64) = (250.0, 3000.0);

#[derive(Debug, Clone, Copy, PartialEq)]
enum BunkerEntryChoice {
    None,
    AwaitingInput,
}

#[derive(Debug, Clone, Copy)]
enum LvlUpState {
    None,
    PendingTab {
        fighter_type: FighterType,
    },
    SelectingStat,
    ConfirmingStat {
        stat_to_increase: StatChoice,
        fighter_type: FighterType,
    },
}

#[derive(Debug, Clone, Copy)]
enum StatChoice {
    Def,
    Atk,
    Spd,
}

pub struct DamageText {
    text: String,
    x: f64,
    y: f64,
    color: [f32; 4],
    lifetime: f64,
}

pub struct TaskRewardNotification {
    pub text: String,
    pub lifetime: f64,
}

/// Notification for displaying current background track name
pub struct TrackNotification {
    pub track_name: String,
    pub lifetime: f64,
}

struct FlickerStrikeEffectInstance {
    x: f64,
    y: f64,
    lifetime: f64,
    max_lifetime: f64,
}

struct KineticStrikeEffectInstance {
    x: f64,
    y: f64,
    lifetime: f64,
    max_lifetime: f64,
    texture_index: usize,
}

struct KineticRushLine {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    lifetime: f64,
    max_lifetime: f64,
}

struct CrashedFighterJetSite {
    sbrx_field_id: SbrxFieldId,
    world_x: f64,
    world_y: f64,
}

struct Particle {
    pos: Vec2d,
    vel: Vec2d,
    lifetime: f64,
    max_lifetime: f64,
    color: [f32; 4],
    size: f64,
}

// Represents a single, placed ground asset in the world for rendering.
struct PlacedAsset {
    texture_name: String,
    x: f64,
    y: f64,
}

struct FortSiloSurvivor {
	x: f64,
	y: f64,
	interaction_triggered: bool,
}

struct Survivor {
	x: f64,
	y: f64,
	fighter_type: FighterType,
	is_rescued: bool,
}

fn spawn_particles(particles: &mut Vec<Particle>, pos_x: f64, pos_y: f64, count: usize) {
    let mut rng = rand::rng();
    let speed_range = (150.0, 250.0);
    let lifetime_range = (0.3, 0.5);
    let base_size = 10.0;

    for _ in 0..count {
        let angle = rng.random_range(0.0..std::f64::consts::TAU);
        let speed = rng.random_range(speed_range.0..speed_range.1);
        let lifetime = rng.random_range(lifetime_range.0..lifetime_range.1);

        particles.push(Particle {
            pos: Vec2d::new(pos_x, pos_y),
            vel: Vec2d::new(angle.cos() * speed, angle.sin() * speed),
            lifetime,
            max_lifetime: lifetime,
            color: [1.0, 1.0, 1.0, 1.0], // Solid white
            size: base_size,             // Fixed size
        });
    }
}

fn find_assets_folder(exe_dir: &Path) -> PathBuf {
    let mut potential_asset_dirs: Vec<Option<PathBuf>> = Vec::new();
    potential_asset_dirs.push(Some(Path::new("assets").to_path_buf()));
    potential_asset_dirs.push(Some(exe_dir.join("assets")));
    potential_asset_dirs.push(exe_dir.parent().map(|p| p.join("assets")));
    potential_asset_dirs.push(Some(Path::new(".").join("assets")));
    for dir_option in potential_asset_dirs {
        if let Some(dir) = dir_option {
            if dir.exists() {
                return dir;
            }
        }
    }
    println!("Warning: Could not find assets directory. Using default path 'assets'.");
    PathBuf::from("assets")
}

fn handle_melee_strike<'a>(
    fighter: &mut Fighter,
    camera: &Camera,
    mouse_x: f64,
    mouse_y: f64,
    fixed_crater: &mut FixedCrater,
    combo_system: &mut ComboSystem,
    strike: &mut Strike,
    audio_manager: &AudioManager,
	chatbox: &mut ChatBox,
    current_strike_textures: &'a Vec<G2dTexture>,
    current_racer_texture: &mut &'a G2dTexture,
    strike_frame: &mut usize,
    strike_animation_timer: &mut f64,
    movement_active: &mut bool,
    backpedal_active: &mut bool,
    frontal_strike_timer: &mut f64,
    frontal_strike_angle: &mut f64,
    frontal_strike_color: &mut [f32; 4],
    frontal_strike_is_special: &mut bool,
    combo_finisher_slash_count: &mut u32,
    cpu_entities: &mut Vec<CpuEntity>,
    damage_texts: &mut Vec<DamageText>,
	is_paused: bool,
) {
    let (mut wmx, mut wmy) = screen_to_world(camera, mouse_x, mouse_y);
    let dx = wmx - fixed_crater.x;
    let dy = wmy - fixed_crater.y;
    let hr = fixed_crater.radius;
    let vr = fixed_crater.radius * 0.75;
    let dsq = (dx * dx) / (hr * hr) + (dy * dy) / (vr * vr);

    if fighter.combat_mode == CombatMode::CloseCombat && dsq > 1.0 {
        let distance_factor = dsq.sqrt();
        wmx = fighter.x + (wmx - fighter.x) / distance_factor;
        wmy = fighter.y + (wmy - fighter.y) / distance_factor;
    }

    if let Some(result) = combo_system.handle_strike_for_fighter(fighter.fighter_type) {
        audio_manager
            .play_sound_effect(if result.is_combo_finisher {
                "slash_combo"
            } else {
                "melee"
            })
            .ok();
        strike.trigger(wmx, wmy);
        *combo_finisher_slash_count = result.finisher_slash_count;
        if !current_strike_textures.is_empty() {
            *current_racer_texture =
                &current_strike_textures[*strike_frame % current_strike_textures.len()];
            *strike_frame = (*strike_frame + 1) % current_strike_textures.len();
        }
        *strike_animation_timer = 0.25;
        *movement_active = false;
        *backpedal_active = false;

        let strike_is_special = result.is_combo_finisher || combo_system.is_combo_strike_active();
        *frontal_strike_is_special = strike_is_special;
        *frontal_strike_color = if strike_is_special {
            [0.0, 1.0, 0.2, 0.4]
        } else {
            [0.25, 0.25, 0.25, 0.25]
        };
        *frontal_strike_timer = 0.1;
        *frontal_strike_angle = (wmy - fighter.y).atan2(wmx - fighter.x);

        if CPU_ENABLED && !is_paused {
            let mut point_damage = fighter.melee_damage * result.damage_multiplier;
			
            // Damage reduction if in Ranged mode on foot (Racer only)
            if fighter.fighter_type == FighterType::Racer && fighter.state == RacerState::OnFoot && !fighter.boost {
                point_damage *= 0.90; // 10% reduction
            }			

            // Apply damage reduction if on bike
            if fighter.state == RacerState::OnBike {
                if fighter.fighter_type == FighterType::Racer && fighter.boost {
                    point_damage *= 0.50; // 25% increase over base 0.1 (0.1 * 1.25)
                } else {
                    point_damage *= 0.25; // 90% reduction
                }
            }
            // 25% damage boost while ATOMIC-STATE is active
            if fighter.invincible_timer > 1.0 {
                point_damage *= 1.25;
            }			
            let frontal_damage = point_damage / 2.0;
            let mut point_hit_applied = false;

            for cpu in cpu_entities.iter_mut() {
                let mut total_damage_this_cpu = 0.0;
                let mut was_point_hit = false;
                let mut was_frontal_hit = false;

                if !point_hit_applied {
                    let sdx = wmx - cpu.x;
                    let sdy = wmy - cpu.y;
                    if (sdx * sdx + sdy * sdy).sqrt() < COLLISION_THRESHOLD {
                        total_damage_this_cpu += point_damage;
                        was_point_hit = true;
                        point_hit_applied = true;
                    }
                }

                if !was_point_hit {
                    let dist_sq_from_player =
                        (cpu.x - fighter.x).powi(2) + (cpu.y - fighter.y).powi(2);
                    if dist_sq_from_player <= fixed_crater.radius.powi(2) {
                        let mouse_vec_x = wmx - fighter.x;
                        let mouse_vec_y = wmy - fighter.y;
                        let enemy_vec_x = cpu.x - fighter.x;
                        let enemy_vec_y = cpu.y - fighter.y;
                        let dot_product = mouse_vec_x * enemy_vec_x + mouse_vec_y * enemy_vec_y;

                        if dot_product > 0.0 {
                            total_damage_this_cpu += frontal_damage;
                            was_frontal_hit = true;
                        }
                    }
                }

                if total_damage_this_cpu > 0.0 {
                    cpu.current_hp -= total_damage_this_cpu;
					
                    // Track that a hit connected during the T3 combo (strikes 1-4) for Racer
                    if was_point_hit
                        && result.finisher_hit_count < 5
                        && fighter.fighter_type == FighterType::Racer
                    {
                        combo_system.racer_combo_hit_connected = true;
                    }					

                    let text_color = if fighter.invincible_timer > 1.0 {
                        [0.7, 1.0, 0.0, 1.0] // yellow-green — ATOMIC-STATE active
                    } else if was_point_hit && result.is_combo_finisher {
                        [0.0, 1.0, 0.0, 1.0]
                    } else if was_point_hit {
                        [1.0, 1.0, 1.0, 1.0]
                    } else {
                        [0.8, 0.8, 0.8, 1.0]
                    };

                    damage_texts.push(DamageText {
                        text: format!("{:.0}", total_damage_this_cpu),
                        x: cpu.x,
                        y: cpu.y - 50.0,
                        color: text_color,
                        lifetime: 0.25,
                    });

                    if !cpu.is_dead() {
                        if was_point_hit {
                            if result.knockback {
                                cpu.apply_knockback(wmx, wmy, result.knockback_force);
                            }	
							
							// Achievement: 5-Hit Combo Reward for RACER
							// We check result.finisher_hit_count directly instead of result.apply_stun
							if result.finisher_hit_count == 5 && fighter.fighter_type == FighterType::Racer {
								fighter.invincible_timer = 2.5;
								chatbox.add_interaction(vec![
									("ATOMIC-STATE", MessageType::Warning),
								]);
							}							
							
                            if result.apply_stun {
                                if result.finisher_hit_count == 3 {
                                    cpu.stun_timer = 1.0;
                                    damage_texts.push(DamageText {
                                        text: "STUN".to_string(),
                                        x: cpu.x,
                                        y: cpu.y - 90.0,
                                        color: [1.0, 1.0, 1.0, 1.0],
                                        lifetime: 0.5,
                                    });
                                } else if result.finisher_hit_count == 5 {
                                    cpu.stun_timer = 2.0;
									
									// Achievement: 5-Hit Combo Reward for RACER
									if fighter.fighter_type == FighterType::Racer {
										fighter.invincible_timer = 2.5;
										chatbox.add_interaction(vec![
											("ATOMIC-STATE", MessageType::Warning),
										]);
									}									
                                }
                            }
                        } else if was_frontal_hit && result.is_combo_finisher {
                            let frontal_knockback_force = result.knockback_force / 2.0;
                            cpu.apply_knockback(wmx, wmy, frontal_knockback_force);
                        }
                    }
                }
            }
            // Racer ATOMIC-STATE: grant if 5th strike fires and any of strikes 1-4 connected
            if result.finisher_hit_count == 5
                && fighter.fighter_type == FighterType::Racer
                && combo_system.racer_combo_hit_connected
                && fighter.invincible_timer <= 1.0
            {
                fighter.invincible_timer = 2.5;
                chatbox.add_interaction(vec![
                    ("ATOMIC-STATE", MessageType::Warning),
                ]);
                combo_system.racer_combo_hit_connected = false;
            }			
        }
    }
}

fn award_kill_score(
    fighter: &mut Fighter,
    points: u32,
    chatbox: &mut ChatBox,
    lvl_up_state: &mut LvlUpState,
    reason: &str,
) {
    let current_kills = fighter
        .kill_counters
        .entry(fighter.fighter_type)
        .or_insert(0);
    *current_kills += points;
    println!("Awarded {} kill score points for {}.", points, reason);

    let current_level = fighter.levels.entry(fighter.fighter_type).or_insert(1);
    let stat_points = fighter
        .stat_points_to_spend
        .entry(fighter.fighter_type)
        .or_insert(0);
    let levels_gained = mechanics::lvl_up::check_for_level_up(current_kills, current_level);

    if levels_gained > 0 {
        *stat_points += levels_gained;
        *lvl_up_state = LvlUpState::PendingTab {
            fighter_type: fighter.fighter_type,
        };
        let fighter_name = match fighter.fighter_type {
            FighterType::Racer => "RACER",
            FighterType::Soldier => "SOLDIER",
            FighterType::Raptor => "RAPTOR",
        };
        chatbox.add_interaction(vec![(
            &format!(
                "!! [TAB] TO LVL UP [{}] +[{}] !!",
                fighter_name, *stat_points
            ),
            MessageType::Warning,
        )]);
    }
}

/// Helper to load a texture or exit with error message
fn load_texture_or_exit(
    texture_context: &mut G2dTextureContext,
    path: &Path,
    settings: &TextureSettings,
    name: &str,
) -> G2dTexture {
    Texture::from_path(texture_context, path, Flip::None, settings)
        .unwrap_or_else(|e| {
            eprintln!("Fatal: Failed to load {} texture at {:?}: {}", name, path, e);
            std::process::exit(1);
        })
}

fn load_cpu_textures(
    texture_context: &mut G2dTextureContext,
    assets_path: &Path,
    variant_folder: &str,
) -> Vec<G2dTexture> {
    let mut textures = Vec::new();
    let settings = TextureSettings::new();
    let base_path = assets_path.join(format!("entity/{}/{}.png", variant_folder, variant_folder));
	let base_texture = match Texture::from_path(texture_context, &base_path, Flip::None, &settings) {
		Ok(tex) => tex,
		Err(e) => {
			eprintln!(
				"Fatal: Failed to load {} base texture at {:?}: {}",
				variant_folder, base_path, e
			);
			std::process::exit(1);
		}
	};
    textures.push(base_texture);
    for i in 1..=3 {
        let strike_path = assets_path.join(format!(
            "entity/{}/{}Strike{}.png",
            variant_folder, variant_folder, i
        ));
        if strike_path.exists() {
			match Texture::from_path(texture_context, &strike_path, Flip::None, &settings) {
				Ok(tex) => textures.push(tex),
				Err(e) => {
					eprintln!(
						"Fatal: Failed to load {} strike{} texture at {:?}: {}",
						variant_folder, i, strike_path, e
					);
					std::process::exit(1);
				}
			}
        } else {
            if let Some(first_tex) = textures.first().cloned() {
                textures.push(first_tex);
            }
        }
    }
    while textures.len() < 4 {
        if let Some(first_tex) = textures.first().cloned() {
            textures.push(first_tex);
        } else {
            eprintln!(
                "No textures loaded for CPU variant {}, cannot create fallbacks.",
                variant_folder
            );
			std::process::exit(1);
        }
    }
    textures
}

fn check_and_display_demonic_presence(
    current_field_id: &SbrxFieldId,
    cpu_entities: &[CpuEntity],
    chatbox: &mut ChatBox,
    fog_of_war: &FogOfWar,
) {
    if FOG_OF_WAR_ENABLED && fog_of_war.is_fog_enabled(*current_field_id) {
        if cpu_entities
            .iter()
            .any(|e| e.variant == CpuVariant::BloodIdol && !e.is_dead())
        {
            chatbox.add_interaction(vec![(
                "WARNING: STRONG ENCOUNTER",
                MessageType::Warning,
            )]);
        }
    }
}

fn handle_ambient_playlist(
    audio_manager: &AudioManager,
    bgm_sink: &mut Option<Sink>,
    index: &mut usize,
) -> Option<String> {
    let playlist = [
        "sdtrk1", "sdtrk2", "sdtrk3", "sdtrk4", "sdtrk5",
        "sdtrk6", "sdtrk7", "sdtrk8", "sdtrk9", "sdtrk10",
        "sdtrk11", "sdtrk12", "sdtrk13", "sdtrk14", "sdtrk15"
    ];	

    if bgm_sink.as_ref().map_or(true, |s| s.empty()) {
        let track = playlist[*index];
        if let Ok(sink) = audio_manager.play_sfx_with_sink(track) {
            *bgm_sink = Some(sink);
            *index = (*index + 1) % playlist.len();
			return Some(track.to_string());
        }
    }
	None
}

fn main() {
    let screen_width = WIDTH;
    let screen_height = HEIGHT;
    let line_y = HORIZON_LINE;
    let movement_buffer_duration = MOVEMENT_BUFFER_DURATION;
    let rush_duration = RUSH_DURATION;

    let bike_interaction_distance = BIKE_INTERACTION_DISTANCE;
    let max_stars = if PERFORMANCE_MODE { 25 } else { 25 };
    let sky_width = 7250.0;
    let mut game_time = 0.0;
    let mut firmament_boss_defeated = false;
    let mut bunker_entry_choice = BunkerEntryChoice::None;
    let mut fort_silo_gravity_message_shown = false;
    let mut has_blood_idol_fog_spawned_once: bool = false;
    let mut void_tempest_spawned_for_survivors: bool = false;

    // Helper function to spawn a random CPU entity for the arena mode
    fn spawn_random_cpu(line_y: f64, stage: u32) -> CpuEntity {
        let mut rng = rand::rng();
        // Determine the range of enemies to spawn based on the arena stage
        let max_variant = if stage >= 2 {
            10 // Stage 2+ includes all 10 variants
        } else {
            5 // Stage 1 includes the first 5 variants
        };

        let variant_choice = rng.random_range(0..max_variant);

        // All arena enemies can spawn anywhere on the map
        let x = safe_gen_range(MIN_X, MAX_X, "random_spawn_x");
        let y = safe_gen_range(MIN_Y, MAX_Y, "random_spawn_y");

        let mut new_cpu = match variant_choice {
            // stage 1 enemies
            0 => CpuEntity::new_giant_mantis(line_y),
            1 => CpuEntity::new_rattlesnake(line_y),
            2 => CpuEntity::new_giant_rattlesnake(line_y),
            3 => {
                let (base_hp, base_speed) = (250.0, 150.0);
                CpuEntity::new_blood_idol(line_y, base_hp, base_speed)
            }
            4 => CpuEntity::new_raptor(x, y), // Raptor constructor takes x,y directly

            // Stage 2+ enemies
            5 => {
                let (base_hp, base_speed) = (250.0, 150.0);
                CpuEntity::new_void_tempest(line_y, base_hp, base_speed)
            }
            6 => CpuEntity::new_t_rex(x, y),
            7 => CpuEntity::new_night_reaver(x, y),
            8 => CpuEntity::new_light_reaver(x, y),
            _ => CpuEntity::new_razor_fiend(x, y), // Case 9 and default
        };

        // Override default spawn positions to fill the entire arena
        new_cpu.x = x;
        new_cpu.y = y;
        new_cpu
    }

    println!("Initializing sbrx0.2.16 game with line_y = {}", line_y);

 	let exe_path = match env::current_exe() {
 	    Ok(path) => path,
 	    Err(e) => {
 	        eprintln!("Fatal: Failed to get executable path: {}", e);
 	        std::process::exit(1);
 	    }
 	};
 	let exe_dir = match exe_path.parent() {
 	    Some(dir) => dir,
 	    None => {
 	        eprintln!("Fatal: Failed to get executable directory");
 	        std::process::exit(1);
 	    }
 	};
 	let audio_manager = match AudioManager::new() {
 	    Ok(manager) => manager,
 	    Err(e) => {
 	        eprintln!("Fatal: Failed to initialize audio: {}", e);
 	        std::process::exit(1);
 	    }
 	};
    audio_manager
        .load_sfx_directory(&exe_dir)
        .unwrap_or_else(|e| {
            println!(
                "Warning: Failed to load sound effects: {}. Game will continue.",
                e
            )
        });

    let mut window: PistonWindow =
        WindowSettings::new("Sabercross", [screen_width as u32, screen_height as u32])
            .resizable(false)
            .decorated(false)
            .exit_on_esc(false)
            .build()
 	        .unwrap_or_else(|e| {
 	            eprintln!("Fatal: Failed to build PistonWindow: {}", e);
 	            std::process::exit(1);
 	        });
    window.set_position([0, 0]);
	window.window.window.set_cursor_visible(false);
    println!("sbrx0.2.16 Window created.");

    let sbrx_assets_path = find_assets_folder(&exe_dir);
    let mut texture_context = window.create_texture_context();

    // Initialize ChatBox after texture_context is created
    let mut chatbox = ChatBox::new(&mut window, &sbrx_assets_path);

    // --- NEW: Load all ground asset textures ---
    let mut ground_asset_textures: HashMap<String, G2dTexture> = HashMap::new();
    let ground_asset_paths = [
        ("broken_post", "ground/broken_post.png"),
        ("cactus", "ground/cactus.png"),
        ("cactus2", "ground/cactus2.png"),
        ("campfire_lit", "ground/campfire_lit.png"),
        ("campfire_loaded", "ground/campfire_loaded.png"),
        ("campfire_out", "ground/campfire_out.png"),
        ("cow_skull", "ground/cow_skull.png"),
        ("dead_tree", "ground/dead_tree.png"),
        ("fence", "ground/fence.png"),
        ("fence_corner", "ground/fence_corner.png"),
        ("fence_side", "ground/fence_side.png"),
        ("fence_side_double", "ground/fence_side_double.png"),
        ("log", "ground/log.png"),
        ("log_pile", "ground/log_pile.png"),
        ("plant", "ground/plant.png"),
        ("rock", "ground/rock.png"),
        ("rock2", "ground/rock2.png"),
        ("rock3", "ground/rock3.png"),
        ("rock4", "ground/rock4.png"),
        ("rock5", "ground/rock5.png"),
        ("rock6", "ground/rock6.png"),
        ("tall_grass", "ground/tall_grass.png"),
        ("wagon", "ground/wagon.png"),
        ("yucca", "ground/yucca.png"),
        ("yucca2", "ground/yucca2.png"),
    ];

    let texture_settings = TextureSettings::new();
    for (name, path) in ground_asset_paths.iter() {
        let full_path = sbrx_assets_path.join(path);
        match Texture::from_path(
            &mut texture_context,
            &full_path,
            Flip::None,
            &texture_settings,
 	    ) {
 	        Ok(texture) => {
 	            ground_asset_textures.insert(name.to_string(), texture);
 	        }
 	        Err(e) => {
 	            eprintln!(
 	                "Fatal: Failed to load ground asset texture '{}' at {:?}: {}",
 	                name, full_path, e
 	            );
 	            std::process::exit(1);
 	        }
 	    }
    }

    let font_path = sbrx_assets_path.join("fonts").join("Segment16A.ttf");
	let mut glyphs = match window.load_font(font_path.clone()) {
 	    Ok(g) => g,
 	    Err(e) => {
 	        eprintln!(
 	            "Failed to load font at {:?}: {}. Attempting fallback.",
 	            font_path, e
 	        );
 	        let fallback_font_path_win = Path::new("C:\\Windows\\Fonts\\arial.ttf");
 	        let fallback_font_path_linux = Path::new("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
 	        if fallback_font_path_win.exists() {
 	            match window.load_font(fallback_font_path_win) {
 	                Ok(g) => g,
 	                Err(fe) => {
 	                    eprintln!(
 	                        "Fatal: Failed to load Windows fallback font: {:?}, original error: {:?}",
 	                        fe, e
 	                    );
 	                    std::process::exit(1);
 	                }
 	            }
 	        } else if fallback_font_path_linux.exists() {
 	            match window.load_font(fallback_font_path_linux) {
 	                Ok(g) => g,
 	                Err(fe) => {
 	                    eprintln!(
 	                        "Fatal: Failed to load Linux fallback font: {:?}, original error: {:?}",
 	                        fe, e
 	                    );
 	                    std::process::exit(1);
 	                }
 	            }
 	        } else {
 	            eprintln!(
 	                "Fatal: Failed to load primary font and no fallback fonts found: original error: {:?}",
 	                e
 	            );
 	            std::process::exit(1);
 	        }
 	    }
 	};

 	let title_screen_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("sabercrossTITLE.png"),
 	    &texture_settings,
 	    "title_screen",
 	);		
 	let pause_screen_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("pause_screen1.png"),
 	    &texture_settings,
 	    "pause_screen",
 	);			
 	let inputs_display_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("inputs.png"),
 	    &texture_settings,
 	    "inputs_display",
 	);
	let gear_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("gear.png"),
 	    &texture_settings,
 	    "gear",
 	);	
 	let block_fatigue_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("block_fatigue.png"),
 	    &texture_settings,
 	    "block_fatigue",
 	);
 	let flicker_strike_effect_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("effects/FlickerStrikeEffect.png"),
 	    &texture_settings,
 	    "flicker_strike_effect",
 	);
	let atomic_state_texture = load_texture_or_exit(
	    &mut texture_context,
	    &sbrx_assets_path.join("effects/atomic_state.png"),
	    &texture_settings,
	    "atomic_state",
	);	

 	let kinetic_strike_effect_texture1 = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("effects/KineticStrikeEffect1.png"),
 	    &texture_settings,
 	    "kinetic_strike_effect1",
 	);
 	let kinetic_strike_effect_texture2 = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("effects/KineticStrikeEffect2.png"),
 	    &texture_settings,
 	    "kinetic_strike_effect2",
 	);
	let kinetic_strike_effect_texture3 = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("effects/KineticStrikeEffect3.png"),
 	    &texture_settings,
 	    "kinetic_strike_effect3",
 	);

    let kinetic_strike_textures = vec![
        kinetic_strike_effect_texture1,
        kinetic_strike_effect_texture2,
        kinetic_strike_effect_texture3,
    ];

 	let aim_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("aim.png"),
 	    &texture_settings,
 	    "aim",
 	);
 	let strike_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("strike.png"),
 	    &texture_settings,
 	    "strike",
 	);
 	let track_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("racetrack.png"),
 	    &texture_settings,
 	    "track",
 	);
 	let rocketbay_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("rocketbay.png"),
 	    &texture_settings,
 	    "rocketbay",
 	);
	let fort_silo2_texture = load_texture_or_exit(
	    &mut texture_context,
	    &sbrx_assets_path.join("fort_silo2.png"),
	    &texture_settings,
	    "fort_silo2",
	);	
	
    let info_post_texture_path = sbrx_assets_path.join("info_post.png");
 	let info_post_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &info_post_texture_path,
 	    &texture_settings,
 	    "info_post",
 	);
    let fort_silo_texture_path = sbrx_assets_path.join("fort_silo.png");
 	let fort_silo_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &fort_silo_texture_path,
 	    &texture_settings,
 	    "fort_silo",
 	);
    let remains_texture_path = sbrx_assets_path.join("remains.png");
 	let remains_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &remains_texture_path,
 	    &texture_settings,
 	    "remains",
 	);
    let raptor_block_break_nest_texture_path =
        sbrx_assets_path.join("player/raptor/block_break.png");
 	let raptor_block_break_nest_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &raptor_block_break_nest_texture_path,
 	    &texture_settings,
 	    "raptor_block_break_nest",
 	);

    // --- NEW: Load ground textures ---
    let ground_texture_path = sbrx_assets_path.join("ground/ground_texture.png");
    let mut ground_textures = Vec::new();
    let flips = [Flip::None, Flip::Horizontal, Flip::Vertical, Flip::Both];
    for flip in &flips {
	    match Texture::from_path(
 	        &mut texture_context,
 	        &ground_texture_path,
 	        *flip,
 	        &texture_settings,
 	    ) {
 	        Ok(tex) => ground_textures.push(tex),
 	        Err(e) => {
 	            eprintln!(
 	                "Fatal: Failed to load ground texture at {:?} with flip {:?}: {}",
 	                ground_texture_path, flip, e
 	            );
 	            std::process::exit(1);
 	        }
 	    }
    }

	    let racer_textures = load_fighter_textures(&mut window, "racer", sbrx_assets_path.clone())
	        .unwrap_or_else(|e| {
	            eprintln!("Fatal error loading racer textures: {}", e);
	            std::process::exit(1);
	        });
	    let soldier_textures = load_fighter_textures(&mut window, "soldier", sbrx_assets_path.clone())
	        .unwrap_or_else(|e| {
	            eprintln!("Fatal error loading soldier textures: {}", e);
	            std::process::exit(1);
	        });
	    let raptor_textures = load_fighter_textures(&mut window, "raptor", sbrx_assets_path.clone())
	        .unwrap_or_else(|e| {
	            eprintln!("Fatal error loading raptor textures: {}", e);
	            std::process::exit(1);
	        });
    let mut current_idle_texture = &racer_textures.idle;
    let mut current_fwd_texture = &racer_textures.fwd;
    let mut current_backpedal_texture = &racer_textures.backpedal;
    let mut current_block_texture = &racer_textures.block;
    let mut current_block_break_texture = &racer_textures.block_break;
    let mut current_ranged_texture = &racer_textures.ranged;
    let mut current_rush_texture = &racer_textures.rush;
    let mut current_strike_textures = &racer_textures.strike;
    let mut current_ranged_marker_texture = &racer_textures.ranged_marker;
    let mut current_ranged_blur_texture = &racer_textures.ranged_blur;
    let mut current_racer_texture = current_idle_texture;
    let sbrx_bike_texture_path = sbrx_assets_path.join("player/racer/sbrx_bike.png");
 	let sbrx_bike_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_bike_texture_path,
 	    &texture_settings,
 	    "sbrx_bike",
 	);
 	let sbrx_bike_crashed_texture = load_texture_or_exit(
 		&mut texture_context,
 		&sbrx_assets_path.join("player/racer/sbrx_bike_crashed.png"),
 		&texture_settings,
 		"sbrx_bike_crashed",
 	);	
    let sbrx_quad_texture_path = sbrx_assets_path.join("player/soldier/quad.png");
 	let sbrx_quad_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_quad_texture_path,
 	    &texture_settings,
 	    "sbrx_quad",
 	);
 	let sbrx_quad_crashed_texture = load_texture_or_exit(
 		&mut texture_context,
 		&sbrx_assets_path.join("player/soldier/quad_crashed.png"),
 		&texture_settings,
 		"sbrx_quad_crashed",
 	);	

    // Load Pulse Orb Texture
    let pulse_orb_texture_path = sbrx_assets_path.join("projectile/pulse_orb.png");
 	let pulse_orb_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &pulse_orb_texture_path,
 	    &texture_settings,
 	    "pulse_orb",
 	);
	
    // Load Shift Function Indicator Textures
    let set_boost_texture_path = sbrx_assets_path.join("set_boost.png");
 	let set_boost_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &set_boost_texture_path,
 	    &texture_settings,
 	    "set_boost",
 	);
 
    let set_ranged_texture_path = sbrx_assets_path.join("set_ranged.png");
 	let set_ranged_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &set_ranged_texture_path,
 	    &texture_settings,
 	    "set_ranged",
 	);
	

    // --- NEW: Load Group Icon Textures ---
    let mut group_icons: HashMap<FighterType, G2dTexture> = HashMap::new();
    let mut group_icons_selected: HashMap<FighterType, G2dTexture> = HashMap::new();

    let fighter_types_for_assets = ["racer", "soldier", "raptor"];
    let fighter_type_enums = [
        FighterType::Racer,
        FighterType::Soldier,
        FighterType::Raptor,
    ];

    for (i, name) in fighter_types_for_assets.iter().enumerate() {
        let icon_path = sbrx_assets_path.join(format!("player/{}/group_icon.png", name));
        let icon_selected_path =
            sbrx_assets_path.join(format!("player/{}/group_icon_selected.png", name));

 	    let icon_tex = load_texture_or_exit(
 	        &mut texture_context,
 	        &icon_path,
 	        &texture_settings,
 	        &format!("{}_group_icon", name),
 	    );
 	    let icon_selected_tex = load_texture_or_exit(
 	        &mut texture_context,
 	        &icon_selected_path,
 	        &texture_settings,
 	        &format!("{}_group_icon_selected", name),
 	    );

        group_icons.insert(fighter_type_enums[i], icon_tex);
        group_icons_selected.insert(fighter_type_enums[i], icon_selected_tex);
    }
    // CPU TEXTURES
    let random_image_x = 500.0;
    let random_image_y = 700.0;
    let mantis_cpu_textures =
        load_cpu_textures(&mut texture_context, &sbrx_assets_path, "giant_mantis");
    let blood_idol_cpu_textures =
        load_cpu_textures(&mut texture_context, &sbrx_assets_path, "blood_idol");
    let rattlesnake_cpu_textures =
        load_cpu_textures(&mut texture_context, &sbrx_assets_path, "rattlesnake");
    let giant_rattlesnake_cpu_textures =
        load_cpu_textures(&mut texture_context, &sbrx_assets_path, "giant_rattlesnake");
    let raptor_cpu_textures = load_cpu_textures(&mut texture_context, &sbrx_assets_path, "raptor");
    let t_rex_cpu_textures = load_cpu_textures(&mut texture_context, &sbrx_assets_path, "t-rex");
    let void_tempest_cpu_textures =
        load_cpu_textures(&mut texture_context, &sbrx_assets_path, "void_tempest");
    let light_reaver_cpu_textures =
        load_cpu_textures(&mut texture_context, &sbrx_assets_path, "light_reaver");
    let night_reaver_cpu_textures =
        load_cpu_textures(&mut texture_context, &sbrx_assets_path, "night_reaver");
    let razor_fiend_cpu_textures =
        load_cpu_textures(&mut texture_context, &sbrx_assets_path, "razor_fiend");

    let grand_commander_down_texture_path = sbrx_assets_path.join("grand_commanderDown.png");
 	let grand_commander_down_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &grand_commander_down_texture_path,
 	    &texture_settings,
 	    "grand_commander_down",
 	);

    let grand_commander_texture_path = sbrx_assets_path.join("grand_commander.png");
 	let grand_commander_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &grand_commander_texture_path,
 	    &texture_settings,
 	    "grand_commander",
 	);

    let fighter_jet_texture_path = sbrx_assets_path.join("vehicle/fighter_jet.png");
	let fighter_jet_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &fighter_jet_texture_path,
 	    &texture_settings,
 	    "fighter_jet",
 	);

    let fuel_pump_texture_path = sbrx_assets_path.join("FuelPump.png");
	let fuel_pump_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &fuel_pump_texture_path,
 	    &texture_settings,
 	    "fuel_pump",
 	);

    // Position at top-left of playable area (MIN_X, MIN_Y is horizon line)
    let fuel_pump = FuelPump::new(MIN_X + 50.0, MIN_Y + 50.0);

    let crashed_fighter_jet_texture_path = sbrx_assets_path.join("vehicle/crashed_fighter_jet.png");
	let crashed_fighter_jet_texture = load_texture_or_exit(
	    &mut texture_context,
	    &crashed_fighter_jet_texture_path,
	    &texture_settings,
	    "crashed_fighter_jet",
	);

    let raptor_nest_texture_path = sbrx_assets_path.join("raptor_nest.png");
	let raptor_nest_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &raptor_nest_texture_path,
 	    &texture_settings,
 	    "raptor_nest",
 	);
	
	let loading_screen_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("loading_screen.png"),
 	    &texture_settings,
 	    "loading_screen",
 	);
	
	let racer_lineup_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("RacerLineup.png"),
 	    &texture_settings,
 	    "racer_lineup",
 	);
	
	let race_spectators_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("RaceSpectators.png"),
 	    &texture_settings,
 	    "race_spectators",
 	);
 	let race_spectators2_texture = load_texture_or_exit(
 	    &mut texture_context,
 	    &sbrx_assets_path.join("RaceSpectators2.png"),
 	    &texture_settings,
 	    "race_spectators2",
 	);
    let mut sbrx_bike = SbrxBike::new(line_y);	
	
    // Use fixed spawn point for Racetrack field (0, 0)
    let mut fighter = Fighter::new(RACETRACK_SPAWN_POINT.0, RACETRACK_SPAWN_POINT.1);
    sbrx_bike.x = fighter.x + 100.0;
    sbrx_bike.y = fighter.y;	
    let mut lvl_up_state = LvlUpState::None;

    // Create the map of mutable stats for each fighter
    let mut fighter_stats_map: HashMap<FighterType, combat::stats::Stats> = HashMap::new();
    fighter_stats_map.insert(FighterType::Racer, combat::stats::RACER_LVL1_STATS);
    fighter_stats_map.insert(FighterType::Soldier, combat::stats::SOLDIER_LVL1_STATS);
    fighter_stats_map.insert(FighterType::Raptor, combat::stats::RAPTOR_LVL1_STATS);

    let mut base_fighter_stats_map: HashMap<FighterType, combat::stats::Stats> =
        fighter_stats_map.clone();
    let mut buffed_fighters: HashSet<FighterType> = HashSet::new();
    let field_trait_manager = FieldTraitManager::new();

    let mut camera = Camera::new();
    camera.x = fighter.x;
    camera.y = fighter.y;
    let track = Track::centered(line_y, screen_height, 2020.0, 1233.0);
    let mut stars: Vec<Star> = (0..max_stars)
        .map(|_| {
            Star::new(
                safe_gen_range(-250.0, sky_width + 250.0, "Star x"),
                safe_gen_range(-250.0, line_y - 5.0, "Star y"),
            )
        })
        .collect();
    let mut fixed_crater = FixedCrater {
        x: fighter.x,
        y: fighter.y,
        radius: 125.0,
    };
    let mut strike = Strike::new(fixed_crater.radius, 10.0);
    let mut shoot = Shoot::new(10.0);
    let mut cpu_entities: Vec<CpuEntity> = Vec::new();
    if CPU_ENABLED {
        if cpu_entities.len() < 10 {
            cpu_entities.push(CpuEntity::new_giant_mantis(line_y));
        }
    }
    let mut spawned_blood_idol_scores: HashSet<u32> = HashSet::new();
    let mut spheres: Vec<Box<MovingSphere>> = Vec::new();
    let mut game_state = GameState::TitleScreen;
    let mut is_paused = false;
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;
    let mut task_system = TaskSystem::new();
    let mut block_system = BlockSystem::new(20);
    let mut combo_system = ComboSystem::new();
    let mut wave_manager = WaveManager::new();
    let mut combo_finisher_slash_count: u32 = 1;
    let mut block_break_animation_active = false;
    let mut key_w_pressed = false;
    let mut key_s_pressed = false;
    let mut key_a_pressed = false;
    let mut key_d_pressed = false;
    let mut movement_timer = 0.0;
    let mut movement_active = false;
    let mut current_movement_direction = MovementDirection::None;
    let mut backpedal_timer = 0.0;
    let mut backpedal_active = false;
    let mut strike_animation_timer = 0.0;
    let mut strike_frame = 0;
    let mut rush_timer = 0.0;
    let mut rush_active = false;
    let mut rush_cooldown = 0.0;
    let mut frontal_strike_timer = 0.0;
    let mut frontal_strike_angle = 0.0;
    let mut frontal_strike_color: [f32; 4] = [0.25, 0.25, 0.25, 0.25];
    let mut frontal_strike_is_special = false;
    let mut spawn_timer = 0.0;
    let mut next_spawn = 1.0;
    let mut continuous_move_timer = 0.0;
    let mut is_in_continuous_move = false;
    let mut last_move_direction = (0.0, 0.0);
    let mut soldier_visible = true;
    let soldier_interaction_distance = 100.0;
    let mut show_soldier_interaction_prompt = false;
    let mut soldier_has_joined = false;
    let mut raptor_is_trapped_in_nest = false;
    let mut show_raptor_interaction_prompt = false;
    let mut raptor_has_joined = false;
    let mut show_raptor_in_nest_graphic = false;
	let mut show_racetrack_soldier_raptor_assets = true; // during racetrack_active 
    let mut t_rex_is_active = false;
    let info_post_position = (950.0, 750.0);

    let mut show_info_post_prompt = false;
    let mut racetrack_info_post_interacted = false;
    let mut show_finale_info_post_prompt = false;
    let mut finale_info_post_interacted = false;

    let mut fort_silo_survivor = FortSiloSurvivor {
        x: 650.0,
        y: 750.0,
        interaction_triggered: false,
    };
    let mut show_fort_silo_survivor_prompt = false;

    // Grand Commander state
    let mut razor_fiend_defeated_flag = false;
    let mut grand_commander_dialogue_triggered = false;
    let mut show_grand_commander_prompt = false;

    let mut racetrack_soldier_dialogue_triggered = false;
    let mut show_racetrack_soldier_prompt = false;


    let mut survivors: HashMap<SbrxFieldId, Vec<Survivor>> = HashMap::new();
    let mut show_survivor_interaction_prompt = false;
    let mut nearby_survivor_index: Option<usize> = None;
    let mut rocketbay_dialogue_triggered = false;

    // --- NEW: Ground Asset Management ---
    let ground_asset_manager = GroundAssetManager::new();
    // Stores generated assets for each visited field to keep them consistent.
    let mut placed_ground_assets: HashMap<SbrxFieldId, Vec<PlacedAsset>> = HashMap::new();
	let collision_barrier_manager = CollisionBarrierManager::new();
	let mut in_jump_sequence = false;  // Tracks if player is mid-jump from '1' -> '2' -> '3'
	

    // --- NEW: Ground Texture State ---
    let mut field_ground_texture_indices: HashMap<SbrxFieldId, usize> = HashMap::new();
    let excluded_ground_texture_fields: HashSet<SbrxFieldId> = HashSet::new();

    // Boundary warning message state is now managed by a simple cooldown.
    let mut boundary_warning_cooldown = 0.0;	
    let mut bike_accelerate_sound_sink: Option<Sink> = None;
    let mut bike_idle_sound_sink: Option<Sink> = None;
    let mut title_sound_played = false;
	let mut title_sound_sink: Option<Sink> = None;
    let mut current_bgm_sink: Option<Sink> = None;
    let mut crickets_sound_sink: Option<Sink> = None;
	let mut ambient_track_state = AmbientTrackState::Background; // [M] key cycles
    let mut sbrx_map_system = SbrxMapSystem::new("FLATLINE".to_string(), SbrxFieldId(0, 0));
    let mut fog_of_war = FogOfWar::new();
    let mut rattlesnakes_spawned_in_field0_score3 = false;
    let mut last_field_id_for_rattlesnake_spawn: Option<SbrxFieldId> = None;
    let mut bike_accelerate_anim_timer = 0.0;
    
    let mut spawned_giant_rattlesnake_scores: HashSet<u32> = HashSet::new();
    let mut fighter_jet_instance: Option<FighterJet> = None;
    let mut fighter_jet_current_sbrx_location: SbrxFieldId = SbrxFieldId(-2, 5);

    let mut fighter_jet_world_x: f64 = DEFAULT_FIGHTER_JET_WORLD_X;
    let mut fighter_jet_world_y: f64 = DEFAULT_FIGHTER_JET_WORLD_Y;
    let mut next_firmament_entry_field_id: firmament_lib::FieldId3D =
        firmament_lib::FieldId3D(-2, 5, 0);
    let mut show_fighter_jet_prompt: bool = false;
    let mut crashed_fighter_jet_sites: Vec<CrashedFighterJetSite> = Vec::new();
    let mut damage_texts: Vec<DamageText> = Vec::new();
    let mut particles: Vec<Particle> = Vec::new();
    let mut active_visual_effects: Vec<FlickerStrikeEffectInstance> = Vec::new();
    let mut pulse_orbs: Vec<PulseOrb> = Vec::new();
    let mut active_kinetic_strike_effects: Vec<KineticStrikeEffectInstance> = Vec::new();
	let mut kinetic_rush_lines: Vec<KineticRushLine> = Vec::new();
    let mut raptor_nests: Vec<RaptorNest> = Vec::new();
    let mut show_raptor_nest_prompt: bool = false;
    let mut show_raptor_nest_exit_prompt = false;
    let mut fort_silo_bunkers: Vec<(f64, f64)> = Vec::new(); // (x, y) positions
    let mut show_bunker_prompt: bool = false;
    let mut show_bunker_exit_prompt = false;
    let mut show_bunker_floor_transition_prompt = false;
    let mut target_floor_from_prompt: Option<i32> = None;
    let mut area_entrance_x: f64 = 0.0;
    let mut area_entrance_y: f64 = 0.0;
    let mut current_area: Option<AreaState> = None;
    let mut next_game_state_after_event: Option<GameState> = None;
    let mut t_rex_spawn_pending: bool = false;
    let mut esc_key_held = false;
    let mut esc_hold_timer = 0.0;
	let mut enter_key_held = false;
    let mut lmb_held = false;
    let mut soldier_rapid_fire_timer = 0.0;
    let mut melee_rapid_fire_timer = 0.0;
    let mut shift_held = false;
    let mut shift_override_active = false;

    // Timers for field-specific continuous spawning
    let mut night_reaver_spawn_timer = 0.0;
    let mut next_night_reaver_spawn = 5.0; // Initial delay
    let mut ambient_playlist_index = 0;


    // --- WAVE SYSTEM: Track completed floors ---
    let mut completed_bunker_waves: HashSet<i32> = HashSet::new();
    let mut bunker_waves_fully_completed = false;
	
    // --- FOG OF WAR: Track which bunker floors have been explored ---
    let mut explored_bunker_floors: HashSet<i32> = HashSet::new();	

    // --- NEW: State for persistent fighter HP and downed status ---
    let mut fighter_hp_map: HashMap<FighterType, f64> = HashMap::new();
    fighter_hp_map.insert(FighterType::Racer, fighter.current_hp);
    let mut downed_fighters: Vec<FighterType> = Vec::new();
    let mut revival_kill_score: u32 = 0;
    let mut group_animation_timers: HashMap<FighterType, f64> = HashMap::new();

    // Field restriction system
    let get_allowed_fields = |soldier_joined: bool, raptor_joined: bool| -> HashSet<SbrxFieldId> {
        if !soldier_joined {
            // Only origin field allowed before soldier joins
            let mut fields = HashSet::new();
            fields.insert(SbrxFieldId(0, 0));
            fields
        } else if !raptor_joined {
            // Only origin and raptor nest fields allowed before raptor joins
            let mut fields = HashSet::new();
            fields.insert(SbrxFieldId(0, 0));
            fields.insert(SbrxFieldId(1, 0));
            fields
        } else {
            // All fields allowed after raptor joins
            HashSet::new() // Empty set means no restrictions
        }
    };

    let mut last_field_entry_point = (RACETRACK_SPAWN_POINT.0, RACETRACK_SPAWN_POINT.1);
    let mut death_screen_cooldown = 0.0;
    
    let mut task_reward_notification: Option<TaskRewardNotification> = None;
	let mut track_notification: Option<TrackNotification> = None;

    let mut racetrack_active = crate::config::ARENA_MODE;
    if crate::config::ARENA_MODE {
        println!("[CONFIG] Arena Mode pre-activated.");
    }
	
	let mut show_collision_debug = 1; // 0: DISABLE ALL, 1: BARRIERS ONLY, 2: ALL
    let mut endless_arena_mode_active = false;
	
    if crate::config::ARENA_MODE {
        endless_arena_mode_active = true;
    }	
	
	let mut arena_kill_count = 0;
	let mut last_arena_milestone = 0;
    let mut endless_arena_timer = 0.0;
    let mut endless_arena_stage = 1; // 1: initial, 2: all enemies, 3: buffs

    let mut firmament_load_requested = false; // New flag to control the loading sequence

    println!("sbrx0.2.16 Starting game loop...");
    while let Some(e) = window.next() {
        // This block now handles the blocking load AFTER the loading screen has been rendered.
        if firmament_load_requested {
            firmament_load_requested = false; // Reset the flag

            // This is the heavy, blocking operation. It runs while the loading screen is displayed.
            let firmament_target_field_id = next_firmament_entry_field_id;
            match firmament_lib::Game::new(
                &mut window,
                Some(firmament_target_field_id),
                firmament_boss_defeated,
				task_system.is_task_complete("LAND ON FORT SILO"),
            ) {
                Ok(mut firmament_game_instance) => {
                    firmament_game_instance.task_bar_open = task_system.open;
                    game_state = GameState::FirmamentMode(Box::new(firmament_game_instance));
                    println!(
                        "Transitioned to FirmamentMode. Target field: {:?}",
                        firmament_target_field_id
                    );								

                    // Reset input states for the new mode
                    key_w_pressed = false;
                    key_s_pressed = false;
                    key_a_pressed = false;
                    key_d_pressed = false;
                    block_system.deactivate();
                }
                Err(err) => {
                    eprintln!("Failed to initialize Firmament game: {}", err);
                    game_state = GameState::Playing; // Fall back on error
                }
            }
            continue; // Skip the rest of this event cycle
        }

        if let Some(Button::Keyboard(Key::Escape)) = e.press_args() {
            esc_key_held = true;
            esc_hold_timer = 0.0;
        }
        if let Some(Button::Keyboard(Key::Escape)) = e.release_args() {
            esc_key_held = false;
            esc_hold_timer = 0.0;
        }
		
        if let Some(Button::Keyboard(Key::Return)) = e.press_args() {
            enter_key_held = true;
        }
        if let Some(Button::Keyboard(Key::Return)) = e.release_args() {
            enter_key_held = false;
        }		
		
        if let Some(args) = e.update_args() {
            if esc_key_held {
                esc_hold_timer += args.dt;
                if esc_hold_timer >= ESC_HOLD_DURATION_TO_EXIT {
                    println!("ESC held for {:.2}s. Exiting.", esc_hold_timer);
                    window.set_should_close(true);
                }
            }
        }
        if let Some(Button::Keyboard(Key::Escape)) = e.press_args() {
            if matches!(game_state, GameState::Playing) && fighter.stun_timer > 0.0 {
                // Ignore Escape during crash stun
            } else {
            match game_state {
                GameState::Playing | GameState::FirmamentMode(_) => {
                    // Check for conditions that prevent pausing
                    if let GameState::Playing = game_state {
                        if block_system.block_broken || block_system.block_fatigue || fighter.stun_timer > 0.0 {
                            println!("Cannot pause while block is broken or fatigued.");
                            continue; // Skip pause logic
                        }
                    }

                    is_paused = !is_paused;
                    combo_system.reset();
                    if is_paused {
                        println!("Game Paused.");
						// Pause bike sounds only - BGM/ambient continues seamlessly
                        if let Some(ref sink) = bike_accelerate_sound_sink {
                            sink.pause();
                        }
                        if let Some(ref sink) = bike_idle_sound_sink {
                            sink.pause();
                        }
                        if let GameState::FirmamentMode(ref mut firmament_game) = game_state {
                            firmament_game.is_paused = true;
                        }
                    } else {
                        println!("Game Unpaused.");
                        // Resume bike sounds - BGM/ambient was never stopped
                        if let Some(ref sink) = bike_accelerate_sound_sink {
                            sink.play();
                        }
                        if let Some(ref sink) = bike_idle_sound_sink {
                            sink.play();
                        }			
				
                        if let GameState::FirmamentMode(ref mut firmament_game) = game_state {
                            firmament_game.is_paused = false;
                        }
                    }
                }
                _ => {}
            }
			}
        }

        // Handle chatbox input separately from game state logic for global access.
        if let Some(Button::Keyboard(key)) = e.press_args() {
            chatbox.handle_key_press(key);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            mouse_x = pos[0];
            mouse_y = pos[1];
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::LShift | Key::RShift => {
                    shift_held = true;
					
                    // Play boost.wav if raptor is active
                    if matches!(game_state, GameState::Playing) 
                        && fighter.fighter_type == FighterType::Raptor
                    {
                        audio_manager.play_sound_effect("boost").unwrap_or_else(|_e| {
                            //println!("[Audio] Failed to play boost sound for raptor: {}", e);
                        });
                    }					
					
                    // Check if Racer is active and Boost mode is enabled [F]
                    if matches!(game_state, GameState::Playing)
                        && fighter.fighter_type == FighterType::Racer
                        && fighter.boost
                    {
                        audio_manager.play_sound_effect("boost").unwrap_or_else(|_e| {
                            //println!("[Audio] Failed to play boost sound: {}", e);
                        });
                    }
                }
                Key::L => {
                    if is_paused {
                        // While paused, [M] cycles the same ambient track state as unpaused
                        // This affects the BGM that continues playing during pause
                        ambient_track_state = match ambient_track_state {
                            AmbientTrackState::Background => {
                                if let Some(sink) = current_bgm_sink.take() {
                                    sink.stop();
                                }
                                match audio_manager.play_sfx_loop("crickets") {
                                    Ok(sink) => crickets_sound_sink = Some(sink),
                                    Err(e) => eprintln!("Failed to play crickets sound: {}", e),
                                }
                                AmbientTrackState::Crickets
                            }
                            AmbientTrackState::Crickets => {
                                if let Some(sink) = crickets_sound_sink.take() {
                                    sink.stop();
                                }
                                if let Some(sink) = current_bgm_sink.take() {
                                    sink.stop();
                                }
                                AmbientTrackState::Muted
                            }
                            AmbientTrackState::Muted => {
                                // Start the next track in the playlist
                                let playlist = [
								"sdtrk1", 
								"sdtrk2", 
								"sdtrk3", 
								"sdtrk4", 
								"sdtrk5",
								"sdtrk6", 
								"sdtrk7", 
								"sdtrk8", 
								"sdtrk9", 
								"sdtrk10",
								"sdtrk11", 
								"sdtrk12", 
								"sdtrk13", 
								"sdtrk14", 
								"sdtrk15"								
								];
                                let track = playlist[ambient_playlist_index];
                                if let Ok(sink) = audio_manager.play_sfx_with_sink(track) {
                                    current_bgm_sink = Some(sink);
                                }
                                ambient_playlist_index = (ambient_playlist_index + 1) % playlist.len();							
                                AmbientTrackState::Background
                            }
                        };
                    } else {
                        // Cycle ambient track state: Background -> Crickets -> Muted -> Background
                        ambient_track_state = match ambient_track_state {
                            AmbientTrackState::Background => {
                                if let Some(sink) = current_bgm_sink.take() {
                                    sink.stop();
                                }
                                match audio_manager.play_sfx_loop("crickets") {
                                    Ok(sink) => crickets_sound_sink = Some(sink),
                                    Err(e) => eprintln!("Failed to play crickets sound: {}", e),
                                }
                                //println!("[Audio] Ambient track: CRICKETS");
                                AmbientTrackState::Crickets
                            }
                            AmbientTrackState::Crickets => {
                                if let Some(sink) = crickets_sound_sink.take() {
                                    sink.stop();
                                }
                                if let Some(sink) = current_bgm_sink.take() {
                                    sink.stop();
                                }
                                //println!("[Audio] Ambient track: MUTED");
                                AmbientTrackState::Muted
                            }
                            AmbientTrackState::Muted => {
                                // Start the next track in the playlist
                                let playlist = [
								"sdtrk1", 
								"sdtrk2", 
								"sdtrk3", 
								"sdtrk4", 
								"sdtrk5",
								"sdtrk6", 
								"sdtrk7", 
								"sdtrk8", 
								"sdtrk9", 
								"sdtrk10",
								"sdtrk11", 
								"sdtrk12", 
								"sdtrk13", 
								"sdtrk14", 
								"sdtrk15"								
								];
                                let track = playlist[ambient_playlist_index];
                                if let Ok(sink) = audio_manager.play_sfx_with_sink(track) {
                                    current_bgm_sink = Some(sink);
                                }
                                ambient_playlist_index = (ambient_playlist_index + 1) % playlist.len();								
                                AmbientTrackState::Background
                            }
                        };
                    }
                }	
                Key::Comma => {
                    // Previous track in playlist
                    if let Some(sink) = current_bgm_sink.take() {
                        sink.stop();
                    }
                    let playlist = [
                        "sdtrk1", "sdtrk2", "sdtrk3", "sdtrk4", "sdtrk5",
                        "sdtrk6", "sdtrk7", "sdtrk8", "sdtrk9", "sdtrk10",
                        "sdtrk11", "sdtrk12", "sdtrk13", "sdtrk14", "sdtrk15"
                    ];
                    // Go back TWO tracks (one to get current, one more for previous)
                    // because index always points to the NEXT track to play
                    if ambient_playlist_index < 2 {
                        ambient_playlist_index = playlist.len() + ambient_playlist_index - 2;
                    } else {
                        ambient_playlist_index -= 2;
                    }
                    let track = playlist[ambient_playlist_index];
                    if let Ok(sink) = audio_manager.play_sfx_with_sink(track) {
                        current_bgm_sink = Some(sink);
						track_notification = Some(TrackNotification {
						    track_name: track.to_string(),
						    lifetime: 3.0,
						});	
                        // Advance index so handle_ambient_playlist plays the NEXT track when this one finishes
                        ambient_playlist_index = (ambient_playlist_index + 1) % playlist.len();
                    }
                    // Set state to Background if it was Muted or Crickets
                    if ambient_track_state != AmbientTrackState::Background {
                        if let Some(sink) = crickets_sound_sink.take() {
                            sink.stop();
                        }
                        ambient_track_state = AmbientTrackState::Background;
                    }
                }
                Key::Period => {
                    // Next track in playlist
                    if let Some(sink) = current_bgm_sink.take() {
                        sink.stop();
                    }
                    let playlist = [
                        "sdtrk1", "sdtrk2", "sdtrk3", "sdtrk4", "sdtrk5",
                        "sdtrk6", "sdtrk7", "sdtrk8", "sdtrk9", "sdtrk10",
                        "sdtrk11", "sdtrk12", "sdtrk13", "sdtrk14", "sdtrk15"
                    ];
					// Index already points to the next track, so just use it
                    let track = playlist[ambient_playlist_index];
                    if let Ok(sink) = audio_manager.play_sfx_with_sink(track) {
                        current_bgm_sink = Some(sink);
					    track_notification = Some(TrackNotification {
						    track_name: track.to_string(),
							lifetime: 3.0,
				 	    });
                        // Advance index so handle_ambient_playlist plays the NEXT track when this one finishes
                        ambient_playlist_index = (ambient_playlist_index + 1) % playlist.len();
                    }
                    // Set state to Background if it was Muted or Crickets
                    if ambient_track_state != AmbientTrackState::Background {
                        if let Some(sink) = crickets_sound_sink.take() {
                            sink.stop();
                        }
                        ambient_track_state = AmbientTrackState::Background;
                    }
                }				
                _ => {}
            }
        }

        if let Some(args) = e.mouse_scroll_args() {
            if args[1] > 0.0 {
                camera.zoom_in();
            } else if args[1] < 0.0 {
                camera.zoom_out();
            }
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            match key {
                Key::LShift | Key::RShift => shift_held = false,
                _ => {}
            }
        }

        match game_state {
            GameState::TitleScreen => {
                if !title_sound_played {
                    if let Ok(sink) = audio_manager.play_sfx_with_sink("title") {
                        title_sound_sink = Some(sink);
                    }
                    title_sound_played = true;
                }
                if e.press_args().is_some() {
                    // Stop title sound when starting the game
                    if let Some(sink) = title_sound_sink.take() {
                        sink.stop();
                    }					
                    game_state = GameState::Playing;
                    // Reset chatbox and fog of war on new game/restart
                    chatbox.clear();
                    fog_of_war = FogOfWar::new();
                    placed_ground_assets.clear();

                    task_system.active = true;
					
                    // Ensure config-based Arena Mode persists through start-press
                    if crate::config::ARENA_MODE {
                        racetrack_active = true;
                        endless_arena_mode_active = true;
                    }					
					
                    last_field_id_for_rattlesnake_spawn = Some(sbrx_map_system.current_field_id);
                    spawned_giant_rattlesnake_scores.clear();
                    spawned_blood_idol_scores.clear();
                    fighter_jet_current_sbrx_location = SbrxFieldId(-2, 5);
                    fort_silo_gravity_message_shown = false;
                    fighter_jet_world_x = DEFAULT_FIGHTER_JET_WORLD_X;
                    fighter_jet_world_y = DEFAULT_FIGHTER_JET_WORLD_Y;
                    next_firmament_entry_field_id = firmament_lib::FieldId3D(-2, 5, 0);
                    crashed_fighter_jet_sites.clear();
                    println!("Transitioning to sbrx0.2.16 Playing state.");
                    has_blood_idol_fog_spawned_once = false;
                    check_and_display_demonic_presence(
                        &sbrx_map_system.current_field_id,
                        &cpu_entities,
                        &mut chatbox,
                        &fog_of_war,
                    );
                }
                if let Some(_) = e.render_args() {
                    window.draw_2d(&e, |c, g, device| {
                        clear([0.0, 0.0, 0.0, 1.0], g);
                        image(
                            &title_screen_texture,
                            c.transform.trans(
                                (screen_width - title_screen_texture.get_width() as f64) / 2.0,
                                (screen_height - title_screen_texture.get_height() as f64) / 2.0,
                            ),
                            g,
                        );

                        // Draw Version Number
                        let version_text = "v0 . 2 . 16";
                        let font_size = 20;
                        let text_color = [0.0, 1.0, 0.0, 1.0]; // GrEEN
                        let text_x = 10.0;
                        let text_y = screen_height - 20.0;

                        text::Text::new_color(text_color, font_size)
                            .draw(
                                version_text,
                                &mut glyphs,
                                &c.draw_state,
                                c.transform.trans(text_x, text_y),
                                g,
                            )
                            .ok();

                        chatbox.draw(c, g, &mut glyphs); // Draw chatbox on title screen if open
                        glyphs.factory.encoder.flush(device);
                    });
                }
            }
            GameState::Playing => {
                if let Some(args) = e.update_args() {
                    let dt = args.dt;
					sbrx_bike.update(dt);

                    if !is_paused {
                        // Ensure textures are up to date with boost state every frame
                        let tex_set = match fighter.fighter_type {
                            FighterType::Racer => &racer_textures,
                            FighterType::Soldier => &soldier_textures,
                            FighterType::Raptor => &raptor_textures,
                        };
                        update_current_textures(
                            &fighter,
                            tex_set,
                            &mut current_idle_texture,
                            &mut current_fwd_texture,
                            &mut current_backpedal_texture,
                            &mut current_block_texture,
                            &mut current_block_break_texture,
                            &mut current_ranged_texture,
                            &mut current_ranged_marker_texture,
                            &mut current_ranged_blur_texture,
                            &mut current_rush_texture,
                            &mut current_strike_textures,
                            shift_held,
                        );						
						
                        // --- DEMO END ZONE & ARENA MODE ---
                        if racetrack_active
                            && sbrx_map_system.current_field_id == SbrxFieldId(0, 0)
                            && !endless_arena_mode_active
                        {
                            if fighter.x >= DEMO_END_ZONE[0]
                                && fighter.x <= DEMO_END_ZONE[1]
                                && fighter.y >= DEMO_END_ZONE[2]
                                && fighter.y <= DEMO_END_ZONE[3]
                            {
                                endless_arena_mode_active = true;
                                endless_arena_timer = 0.0; // Reset timer on start
                                endless_arena_stage = 1; // Reset stage
								arena_kill_count = 0;    // Ensure score is 0
								last_arena_milestone = 0; // Reset milestones to allow buffs to trigger again								
                                task_system.mark_proceed_to_starting_line_complete();
                                chatbox.add_interaction(vec![(
                                    "RACING MODE IN DEVELOPMENT. INITIATING ENDLESS ARENA MODE.",
                                    MessageType::Warning,
                                )]);
                                //println!("[ARENA MODE] Player entered demo end zone. Endless arena activated.");
                            }
                        }
						
						// Update background_track notification lifetime
						if let Some(ref mut notif) = track_notification {
							notif.lifetime -= dt;
							if notif.lifetime <= 0.0 {
								track_notification = None;
							}
						}						

                        if endless_arena_mode_active {
                            endless_arena_timer += dt; // Increment timer

                            // Check for stage transitions
                            if endless_arena_timer >= 10.0 && endless_arena_stage < 2 {
                                endless_arena_stage = 2;
                                chatbox.add_interaction(vec![(
                                    "ARENA: MORE POWERFUL FOES APPEAR!",
                                    MessageType::Warning,
                                )]);
                                //println!("[ARENA MODE] Reached 10 seconds. Stage 2 activated.");
                            }
                            if endless_arena_timer >= 20.0 && endless_arena_stage < 3 {
                                endless_arena_stage = 3;
                                chatbox.add_interaction(vec![(
                                    "ARENA: FRENZY ACTIVATED!",
                                    MessageType::Warning,
                                )]);
                                //println!("[ARENA MODE] Reached 20 seconds. Stage 3 activated. Buffing all enemies.");
                                // Buff all existing enemies
                                for cpu in &mut cpu_entities {
                                    cpu.speed *= 2.0;
                                    cpu.damage_value *= 2.0;
                                }
                            }

                            if cpu_entities.len() < 10 {
                                cpu_entities.push(spawn_random_cpu(line_y, endless_arena_stage));
                            }
                        }

                        // --- FIELD TRAIT APPLICATION ---
                        let active_traits = field_trait_manager
                            .get_active_traits_for_field(&sbrx_map_system.current_field_id);

                        let all_fighter_types = [
                            FighterType::Racer,
                            FighterType::Soldier,
                            FighterType::Raptor,
                        ];
                        for ft in all_fighter_types.iter() {
                            let mut should_be_buffed = false;
                            let mut total_level_mod = 0;

                            for trait_instance in &active_traits {
                                if let TraitTarget::Fighter(target_ft) = trait_instance.target {
                                    if target_ft == *ft {
                                        match trait_instance.attribute {
                                            StatAttribute::Level => {
                                                total_level_mod += trait_instance.modifier;
                                                should_be_buffed = true;
                                            }
                                            _ => {} // Handle ATK, DEF, SPD later if needed
                                        }
                                    }
                                }
                            }

                            let is_currently_buffed = buffed_fighters.contains(ft);

                            if should_be_buffed && !is_currently_buffed {
                                // Apply buff
                                if let (Some(stats_to_modify), Some(original_stats)) = (
                                    fighter_stats_map.get_mut(ft),
                                    base_fighter_stats_map.get(ft),
                                ) {
                                    stats_to_modify.defense.hp = original_stats.defense.hp
                                        + (total_level_mod as f64
                                            * combat::stats::HP_PER_DEFENSE_POINT);
                                    stats_to_modify.attack.melee_damage =
                                        original_stats.attack.melee_damage
                                            + (total_level_mod as f64
                                                * combat::stats::DAMAGE_PER_ATTACK_POINT);
                                    stats_to_modify.attack.ranged_damage =
                                        original_stats.attack.ranged_damage
                                            + (total_level_mod as f64
                                                * combat::stats::DAMAGE_PER_ATTACK_POINT);
                                    stats_to_modify.speed.run_speed =
                                        original_stats.speed.run_speed
                                            + (total_level_mod as f64
                                                * combat::stats::SPEED_PER_SPEED_POINT);

                                    buffed_fighters.insert(*ft);
                                    println!("Applied field trait to {:?}", ft);

                                    // If this is the active fighter, update their stats immediately.
                                    if fighter.fighter_type == *ft {
                                        fighter.stats = *stats_to_modify;
										let old_max_hp = fighter.max_hp;
                                        fighter.max_hp = stats_to_modify.defense.hp;
                                        // Scale current HP proportionally, or set to new max if was at full health
                                        if old_max_hp > 0.0 && fighter.current_hp >= old_max_hp {
                                            fighter.current_hp = fighter.max_hp; // Was at full, stay at full
                                        } else if old_max_hp > 0.0 {
                                            // Scale proportionally
                                            fighter.current_hp = (fighter.current_hp / old_max_hp) * fighter.max_hp;
                                        }
                                        fighter.melee_damage = stats_to_modify.attack.melee_damage;
                                        fighter.ranged_damage =
                                            stats_to_modify.attack.ranged_damage;
                                        fighter.run_speed = stats_to_modify.speed.run_speed;
                                        // Update fighter_hp_map with new HP
                                        fighter_hp_map.insert(*ft, fighter.current_hp);										
                                    } else {
                                        // For non-selected fighters, update their HP in the map
                                        let old_max_hp = original_stats.defense.hp;
                                        let new_max_hp = stats_to_modify.defense.hp;
                                        if let Some(current_hp) = fighter_hp_map.get(ft).copied() {
                                            let new_hp = if old_max_hp > 0.0 && current_hp >= old_max_hp {
                                                new_max_hp // Was at full, stay at full
                                            } else if old_max_hp > 0.0 {
                                                (current_hp / old_max_hp) * new_max_hp // Scale proportionally
                                            } else {
                                                new_max_hp
                                            };
                                            fighter_hp_map.insert(*ft, new_hp);
                                        }
                                    }
                                }
                            } else if !should_be_buffed && is_currently_buffed {
                                // Revert buff
                                if let (Some(stats_to_modify), Some(original_stats)) = (
                                    fighter_stats_map.get_mut(ft),
                                    base_fighter_stats_map.get(ft),
                                ) {
                                    *stats_to_modify = *original_stats;
                                    buffed_fighters.remove(ft);
                                    println!("Reverted field trait from {:?}", ft);

                                    // If this is the active fighter, update their stats immediately.
                                    if fighter.fighter_type == *ft {
                                        fighter.stats = *stats_to_modify;
                                        fighter.max_hp = stats_to_modify.defense.hp;
                                        fighter.current_hp = fighter.current_hp.min(fighter.max_hp);
                                        fighter.melee_damage = stats_to_modify.attack.melee_damage;
                                        fighter.ranged_damage =
                                            stats_to_modify.attack.ranged_damage;
                                        fighter.run_speed = stats_to_modify.speed.run_speed;									
                                        fighter_hp_map.insert(*ft, fighter.current_hp);                                    
										} else {
											// For non-selected fighters, cap their HP to the new (lower) max
											let new_max_hp = stats_to_modify.defense.hp;
											if let Some(current_hp) = fighter_hp_map.get(ft).copied() {
												fighter_hp_map.insert(*ft, current_hp.min(new_max_hp));
											}
										}                                
									}
								}
							}	
                        // --- END FIELD TRAIT APPLICATION ---

                        for p in &mut particles {
                            p.pos.x += p.vel.x * dt;
                            p.pos.y += p.vel.y * dt;
                            p.lifetime -= dt;
                        }
                        particles.retain(|p| p.lifetime > 0.0);
                    }

                    if frontal_strike_timer > 0.0 {
                        frontal_strike_timer -= dt;
                    }

                    if let Some(notification) = &mut task_reward_notification {
                        notification.lifetime -= dt;
                        if notification.lifetime <= 0.0 {
                            task_reward_notification = None;
                        }
                    }

                    chatbox.update(dt, enter_key_held);

                    if shift_held && fighter.fighter_type != FighterType::Raptor {
                        if fighter.fighter_type == FighterType::Racer && fighter.boost {
                            // boost mode active, do not enter Ranged mode
                        } else {
                            if fighter.combat_mode == CombatMode::CloseCombat && !shift_override_active
                            {
                                fighter.combat_mode = CombatMode::Ranged;
                                shift_override_active = true;
                                audio_manager.play_sound_effect("aim").ok();
                            }
						}
                    } else if shift_held && fighter.fighter_type == FighterType::Raptor {
                        // raptor Shift-Flight Logic (Toggle ON)
                        if fighter.state == RacerState::OnFoot
                            && !shift_override_active
                            && fighter.fuel > 0.0
							&& !block_system.active
							&& !block_system.rmb_held
                        {
                            fighter.state = RacerState::OnBike;
                            shift_override_active = true;

                            // Update textures for flight mode
                            let tex_set = &raptor_textures;
                            update_current_textures(
                                &fighter,
                                tex_set,
                                &mut current_idle_texture,
                                &mut current_fwd_texture,
                                &mut current_backpedal_texture,
                                &mut current_block_texture,
                                &mut current_block_break_texture,
                                &mut current_ranged_texture,
                                &mut current_ranged_marker_texture,
                                &mut current_ranged_blur_texture,
                                &mut current_rush_texture,
                                &mut current_strike_textures,
								shift_held,
                            );
							// Update to appropriate texture based on current state
							if strike_animation_timer <= 0.0 {
								if block_system.active || block_system.rmb_held {
									current_racer_texture = current_block_texture;
								} else {
									current_racer_texture = current_idle_texture;
								}
							}
                        }
                    } else {
                        if shift_override_active {
                            if fighter.fighter_type == FighterType::Raptor {
                                // raptor Shift-Flight Logic (Toggle OFF)
                                if fighter.state == RacerState::OnBike {
                                    fighter.state = RacerState::OnFoot;

                                    // Update textures for foot mode
                                    let tex_set = &raptor_textures;
                                    update_current_textures(
                                        &fighter,
                                        tex_set,
                                        &mut current_idle_texture,
                                        &mut current_fwd_texture,
                                        &mut current_backpedal_texture,
                                        &mut current_block_texture,
                                        &mut current_block_break_texture,
                                        &mut current_ranged_texture,
                                        &mut current_ranged_marker_texture,
                                        &mut current_ranged_blur_texture,
                                        &mut current_rush_texture,
                                        &mut current_strike_textures,
										shift_held,
                                    );
									if strike_animation_timer <= 0.0 {
										current_racer_texture = current_idle_texture;
									}
                                }
                            } else {
                                // Standard Ranged Mode logic
                                fighter.combat_mode = CombatMode::CloseCombat;
                            }
                            shift_override_active = false;
                        }
                    }

                    for effect in &mut active_visual_effects {
                        effect.lifetime -= dt;
                    }
                    active_visual_effects.retain(|e| e.lifetime > 0.0);

                    for effect in &mut active_kinetic_strike_effects {
                        effect.lifetime -= dt;
                    }
                    active_kinetic_strike_effects.retain(|e| e.lifetime > 0.0);
					
                    for rush_line in &mut kinetic_rush_lines {
                        rush_line.lifetime -= dt;
                    }
                    kinetic_rush_lines.retain(|l| l.lifetime > 0.0);					

                    if bunker_entry_choice == BunkerEntryChoice::AwaitingInput {
                        let mut is_in_range = false;
                        if let Some(&(bunker_x, bunker_y)) = fort_silo_bunkers.first() {
                            let dx = fighter.x - bunker_x;
                            let dy = fighter.y - bunker_y;
                            let distance_sq = dx * dx + dy * dy;
                            if distance_sq < RAPTOR_NEST_INTERACTION_DISTANCE.powi(2) {
                                is_in_range = true;
                            }
                        }
                        if !is_in_range {
                            bunker_entry_choice = BunkerEntryChoice::None;
                        }
                    }

                    // End wave progress if not in a bunker.
                    let is_in_bunker = if let Some(area) = &current_area {
                        area.area_type == AreaType::Bunker
                    } else {
                        false
                    };
                    if !is_in_bunker && wave_manager.is_active() {
                        wave_manager.reset();
                    }

                    if wave_manager.is_active() && !is_paused {
                        let was_active = wave_manager.is_active();

                        if wave_manager.update(dt) {
                            // Time to spawn an enemy for the wave
                            if cpu_entities.len() < 10 {
                                // Determine spawn variant based on current floor
                                let current_floor = if let Some(area) = &current_area {
                                    area.floor
                                } else {
                                    1 // Default if no area context (shouldn't happen during active wave)
                                };
                                
                                let spawn_table = WaveManager::get_spawn_table_for_floor(current_floor);
                                let variant = WaveManager::pick_random_variant(&spawn_table);
                                
                                let x = safe_gen_range(BUNKER_ORIGIN_X, BUNKER_ORIGIN_X + BUNKER_WIDTH, "wave spawn x");
                                let y = safe_gen_range(BUNKER_ORIGIN_Y, BUNKER_ORIGIN_Y + BUNKER_HEIGHT, "wave spawn y");
                                
                                // Factory function logic
                                let mut new_cpu = match variant {
                                    CpuVariant::NightReaver => CpuEntity::new_night_reaver(x, y),
                                    CpuVariant::LightReaver => CpuEntity::new_light_reaver(x, y),
                                    CpuVariant::VoidTempest => {
                                        // VoidTempest uses a different constructor signature, assume base stats for now or define a position-based one
                                        // The original constructor takes (line_y, hp, speed)
                                        // We'll adapt it or create a temporary instance. 
                                        // Better: Update CpuEntity to have a consistent constructor or handle special cases.
                                        // For now, using existing pattern for VoidTempest which usually overrides position.
                                        // Let's use `new_void_tempest` but override x/y immediately.
                                        let mut vt = CpuEntity::new_void_tempest(line_y, 250.0, 150.0);
                                        vt.x = x;
                                        vt.y = y;
                                        vt
                                    },
                                    CpuVariant::RazorFiend => CpuEntity::new_razor_fiend(x, y),
                                    // Fallback for types not usually in bunker waves but safe to handle
                                    _ => CpuEntity::new_night_reaver(x, y), 
                                };

                                wave_manager.apply_modifiers_to_new_cpu(&mut new_cpu);
                                cpu_entities.push(new_cpu);
                                wave_manager.notify_enemy_spawned(); // Notify manager a spawn occurred
                            }
                        }

                        // Check for transition to FRENZY and apply buff
                        if wave_manager.state == crate::mechanics::wave::WaveState::Frenzy
                            && !wave_manager.enrage_buff_applied
                        {
                            chatbox.add_interaction(vec![(
                                "ENEMY FEROCITY INCREASED",
                                MessageType::Notification,
                            )]);
                            for cpu in cpu_entities.iter_mut() {
                                cpu.damage_value *= 2.0;
                                cpu.speed *= 2.0;
                            }
                            wave_manager.enrage_buff_applied = true;
                        }

                        // Check if the encounter just ended in this frame
                        if was_active && !wave_manager.is_active() {
                            if let Some(area) = current_area.as_mut() {
                                area.waves_active = false;
                                // Mark this floor's waves as completed
                                completed_bunker_waves.insert(area.floor);
                                println!(
                                    "[WAVE SYSTEM] Bunker floor {} marked as complete.",
                                    area.floor
                                );
                            }
                            chatbox.add_interaction(vec![(
                                "ALL WAVES CLEARED. EXITS UNLOCKED.",
                                MessageType::Notification,
                            )]);
                            // Check if all bunker waves have been cleared for the first time
                            if !bunker_waves_fully_completed {
                                let required_floors: HashSet<i32> =
                                    [1, 0, -1, -2].iter().cloned().collect();
                                if required_floors.is_subset(&completed_bunker_waves) {
                                    bunker_waves_fully_completed = true;
                                    //println!("[WAVE SYSTEM] All bunker wave encounters completed for the first time. Lockdowns will now be disabled on future visits.");
                                    chatbox.add_interaction(vec![(
                                        "BUNKER SECURED. ALL FLOORS ACCESSIBLE.",
                                        MessageType::Notification,
                                    )]);
                                }
                            }
                        }
                    }

                    // NEW: Update group icon animation timers
                    let mut finished_anims = Vec::new();
                    for (ft, timer) in group_animation_timers.iter_mut() {
                        *timer -= dt;
                        if *timer <= 0.0 {
                            finished_anims.push(*ft);
                        }
                    }
                    for ft in finished_anims {
                        group_animation_timers.remove(&ft);
                    }

                    combo_system.update(dt);
                    // Handle fighter-specific state progression when timer expires
                    if combo_system.timer <= 0.0 && combo_system.is_in_rest_period {
                        combo_system.progress_state_for_fighter(fighter.fighter_type);
                    }

                    // Handle fighter-specific combo state progression
                    if combo_system.timer > 0.0 {
                        // Check if we need to handle fighter-specific progression
                        // This will be called when the timer expires and state should progress
                    }

                    // NEW: Combined rapid fire logic for all fighters
                    if lmb_held {
                        if fighter.is_reloading {
                            // Prevent action if reloading
                        } else {
                            let (wmx, wmy) = screen_to_world(&camera, mouse_x, mouse_y);
                            let dx = wmx - fixed_crater.x;
                            let dy = wmy - fixed_crater.y;
                            let hr = fixed_crater.radius;
                            let vr = fixed_crater.radius * 0.75;
                            let dsq = (dx * dx) / (hr * hr) + (dy * dy) / (vr * vr);

                            let perform_melee = match fighter.combat_mode {
                                CombatMode::CloseCombat => true,
                                CombatMode::Ranged => false,
                                CombatMode::Balanced => dsq <= 1.0,
                            };

                            if perform_melee {
                                // Melee logic for ALL fighters
                                melee_rapid_fire_timer -= dt;
                                if melee_rapid_fire_timer <= 0.0 {
                                    handle_melee_strike(
                                        &mut fighter,
                                        &camera,
                                        mouse_x,
                                        mouse_y,
                                        &mut fixed_crater,
                                        &mut combo_system,
                                        &mut strike,
                                        &audio_manager,
										&mut chatbox,
                                        &current_strike_textures,
                                        &mut current_racer_texture,
                                        &mut strike_frame,
                                        &mut strike_animation_timer,
                                        &mut movement_active,
                                        &mut backpedal_active,
                                        &mut frontal_strike_timer,
                                        &mut frontal_strike_angle,
                                        &mut frontal_strike_color,
                                        &mut frontal_strike_is_special,
                                        &mut combo_finisher_slash_count,
                                        &mut cpu_entities,
                                        &mut damage_texts,
										is_paused,
                                    );
                                    melee_rapid_fire_timer = MELEE_RAPID_FIRE_RATE;
                                }
                            } else {
                                // Ranged logic, currently only for Soldier
                                if fighter.fighter_type == FighterType::Soldier {
                                    soldier_rapid_fire_timer -= dt;
                                    if soldier_rapid_fire_timer <= 0.0 {
                                        if fighter.ammo > 0 {
                                            audio_manager.play_sound_effect("firearm").ok();

                                            let facing_left = wmx < fighter.x;
                                            let (offset_x, offset_y) = match fighter.state {
                                                RacerState::OnFoot => {
                                                    let x = if facing_left { -35.0 } else { 35.0 };
                                                    (x, -15.0)
                                                }
                                                RacerState::OnBike => {
                                                    let x = if facing_left { -60.0 } else { 60.0 };
                                                    (x, -40.0)
                                                }
                                            };
                                            let start_x = fighter.x + offset_x;
                                            let start_y = fighter.y + offset_y;

                                            shoot.trigger(start_x, start_y, wmx, wmy);

                                            current_racer_texture = current_ranged_texture;
                                            strike_animation_timer = 0.25;
                                            movement_active = false;
                                            backpedal_active = false;
                                            soldier_rapid_fire_timer = SOLDIER_RAPID_FIRE_RATE;
                                            fighter.ammo -= 1;
                                            if fighter.ammo == 0 {
                                                fighter.trigger_reload(&audio_manager);
                                                lmb_held = false; // Force input release on reload
                                            }
                                        } else {
                                            // Should generally be caught by ammo check but safe fallback
                                            fighter.trigger_reload(&audio_manager);
                                            lmb_held = false;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    block_system.update(dt, game_time);
                    strike.update(dt);
                    shoot.update(
                        dt,
                        &mut cpu_entities,
                        &mut fighter,
                        &mut damage_texts,
                        is_paused,
                    );

                    if fighter.state == RacerState::OnBike && !is_paused {
                        // Only deplete fuel for RACER and SOLDIER
                        if fighter.fighter_type != FighterType::Raptor && fighter.fuel > 0.0 {
                            fighter.fuel -= FUEL_DEPLETION_RATE * dt;
                        }
                        if fighter.fuel <= 0.0 {
                            fighter.fuel = 0.0;
                            println!("Fuel depleted! Forcing dismount.");
                            fighter.state = RacerState::OnFoot;
                            // Only respawn a physical bike for non-raptors
                            if fighter.fighter_type != FighterType::Raptor {
                                sbrx_bike.respawn(fighter.x, fighter.y);
                            }
                            let tex_set = match fighter.fighter_type {
                                FighterType::Racer => &racer_textures,
                                FighterType::Soldier => &soldier_textures,
                                FighterType::Raptor => &raptor_textures,
                            };
                            update_current_textures(
                                &fighter,
                                tex_set,
                                &mut current_idle_texture,
                                &mut current_fwd_texture,
                                &mut current_backpedal_texture,
                                &mut current_block_texture,
                                &mut current_block_break_texture,
                                &mut current_ranged_texture,
                                &mut current_ranged_marker_texture,
                                &mut current_ranged_blur_texture,
                                &mut current_rush_texture,
                                &mut current_strike_textures,
								shift_held,
                            );
                            current_racer_texture = current_idle_texture;
                        }
                    }

                    let is_stun_locked_for_anim = block_system.is_stun_locked() || fighter.stun_timer > 0.0;
                    if is_stun_locked_for_anim {
                        if !block_break_animation_active {
                            block_break_animation_active = true;
                            movement_active = false;
                            movement_timer = 0.0;
                            backpedal_active = false;
                            backpedal_timer = 0.0;
                            strike_animation_timer = 0.0;
                            strike_frame = 0;
                            rush_active = false;
                            rush_timer = 0.0;
                        }
                        current_racer_texture = current_block_break_texture;
                    } else {
                        if block_break_animation_active {
                            block_break_animation_active = false;
                            if !is_high_priority_animation_active(
                                rush_active,
                                strike_animation_timer,
                                block_system.active,
                                block_system.rmb_held,
                            ) && !movement_active
                                && !backpedal_active
                            {
                                current_racer_texture = current_idle_texture;
                            }
                        }
                        let (current_min_x, current_max_x, current_min_y, current_max_y) =
                            if let Some(ref area_state) = current_area {
                                let (width, height, origin_x, origin_y) = match area_state.area_type
                                {
                                    AreaType::RaptorNest => {
                                        (AREA_WIDTH, AREA_HEIGHT, AREA_ORIGIN_X, AREA_ORIGIN_Y)
                                    }
                                    AreaType::Bunker => (
                                        BUNKER_WIDTH,
                                        BUNKER_HEIGHT,
                                        BUNKER_ORIGIN_X,
                                        BUNKER_ORIGIN_Y,
                                    ),
                                };
                                (origin_x, origin_x + width, origin_y, origin_y + height)
                            } else {
                                let min_y_world = if fighter.state == RacerState::OnBike {
                                    line_y
                                } else {
                                    MIN_Y
                                };
                                (MIN_X, MAX_X, min_y_world, MAX_Y)
                            };
							
						// Detect Rut Zone once for both Bike and OnFoot logic
						let in_rut_zone = collision_barrier_manager.check_rut(
							&sbrx_map_system.current_field_id,
							fighter.x,
							fighter.y,
						);							
							
                        if fighter.state == RacerState::OnBike {
                            let bike_speed = match fighter.fighter_type {
                                FighterType::Racer => BIKE_SPEED,
                                FighterType::Raptor => BIKE_SPEED * 0.75,
                                FighterType::Soldier => BIKE_SPEED * 0.5,
                            };
                            // Apply speed reduction for combat actions AND blocking
                            let mut final_bike_speed = if fighter.combat_action_slowdown_timer > 0.0
                                || block_system.active
                            {
                                bike_speed * 0.75 // 25% speed reduction for combat/blocking
                            } else {
                                bike_speed
                            };
							
                            // Apply 50% speed reduction when bike sliding (backpedal on bike)
                            if backpedal_active {
                                final_bike_speed *= 0.5;
                            }
							
                            if fighter.fighter_type == FighterType::Racer && fighter.boost && shift_held {
                                final_bike_speed *= 1.25 // 1.03; // Increase bike speed by 3%
                            }
							
                            // Apply 2.0x speed buff while ATOMIC-STATE is active (consistent with OnFoot)
                            if fighter.invincible_timer > 1.0 {
                                final_bike_speed *= 2.0;
                            }							

                            // Apply Rut Zone speed reduction (50%) to Bike
                            if in_rut_zone {
                                final_bike_speed *= 0.5;
                            }	

							// Apply Rut Zone speed reduction (50%) to Bike
							if in_rut_zone {
								final_bike_speed *= 0.85;
							}							
							
                            let move_distance = final_bike_speed * dt;
                            let mut moved = false;
                            if key_w_pressed || key_s_pressed || key_a_pressed || key_d_pressed {
                                moved = true;
                            }
                            if !is_paused && !block_system.is_stun_locked() {
                                // raptor flight mode: prevent movement while blocking (same as OnFoot)
                                let raptor_block_prevents_movement = 
                                    fighter.fighter_type == FighterType::Raptor && block_system.active;								
								
                                let mut target_x = fighter.x;
                                let mut target_y = fighter.y;
                                if key_w_pressed && !raptor_block_prevents_movement {
                                    target_y -= move_distance;
                                }
                                if key_s_pressed && !raptor_block_prevents_movement {
                                    target_y += move_distance;
                                }
                                if key_a_pressed && !raptor_block_prevents_movement {
                                    target_x -= move_distance;
                                }
                                if key_d_pressed && !raptor_block_prevents_movement {
                                    target_x += move_distance;
                                }
                                fighter.x = target_x.clamp(current_min_x, current_max_x);
                                fighter.y = target_y.clamp(current_min_y, current_max_y);
                            }
                            if moved
                                && !is_high_priority_animation_active(
                                    rush_active,
                                    strike_animation_timer,
                                    block_system.active,
                                    block_system.rmb_held,
                                )
                            {
                                let (world_mouse_x, _) = screen_to_world(&camera, mouse_x, mouse_y);
                                let moving_horizontally = key_a_pressed || key_d_pressed;
                                let moving_left = key_a_pressed && !key_d_pressed;
                                let moving_towards_mouse = if moving_horizontally {
                                    if world_mouse_x < fighter.x {
                                        moving_left
                                    } else {
                                        !moving_left
                                    }
                                } else {
                                    true
                                };
                                if moving_towards_mouse {
                                    let anim_vec = match fighter.fighter_type {
                                        FighterType::Racer => {
											if fighter.boost && shift_held {
												racer_textures.bike_accelerate_boost.as_ref()
													.unwrap_or(&racer_textures.bike_accelerate)
											} else {
												&racer_textures.bike_accelerate
											}
                                        },
                                        FighterType::Soldier => &soldier_textures.bike_accelerate,
                                        FighterType::Raptor => &raptor_textures.bike_accelerate,
                                    };
                                    if !anim_vec.is_empty() {
                                        bike_accelerate_anim_timer += dt;
                                        let total_duration =
                                            BIKE_ACCELERATE_FRAME_DURATION * anim_vec.len() as f64;
                                        if bike_accelerate_anim_timer >= total_duration {
                                            bike_accelerate_anim_timer -= total_duration;
                                        }
                                        let frame_index = (bike_accelerate_anim_timer
                                            / BIKE_ACCELERATE_FRAME_DURATION)
                                            as usize
                                            % anim_vec.len();
                                        current_racer_texture = &anim_vec[frame_index];
                                    } else {
                                        current_racer_texture = current_fwd_texture;
                                    }
                                    movement_active = true;
                                    movement_timer = 0.25;
                                    backpedal_active = false;
                                    current_movement_direction = MovementDirection::Forward;
                                } else {
                                    current_racer_texture = current_backpedal_texture;
                                    backpedal_active = true;
                                    backpedal_timer = 0.25;
                                    movement_active = false;
                                    current_movement_direction = MovementDirection::Backward;
                                    bike_accelerate_anim_timer = 0.0;
                                }
                            } else if !moved
                                && !is_high_priority_animation_active(
                                    rush_active,
                                    strike_animation_timer,
                                    block_system.active,
                                    block_system.rmb_held,
                                )
                            {
                                if !(movement_active || backpedal_active) {
                                    current_racer_texture = current_idle_texture;
                                }
                                bike_accelerate_anim_timer = 0.0;
                            }
                        }
                        if movement_active {
                            movement_timer -= dt;
                            if movement_timer <= 0.0 {
                                movement_active = false;
                                if !is_high_priority_animation_active(
                                    rush_active,
                                    strike_animation_timer,
                                    block_system.active,
                                    block_system.rmb_held,
                                ) && !backpedal_active
                                {
                                    current_racer_texture = current_idle_texture;
                                    current_movement_direction = MovementDirection::None;
                                }
                            }
                        }
                        if backpedal_active {
                            backpedal_timer -= dt;
                            if backpedal_timer <= 0.0 {
                                backpedal_active = false;
                                if !is_high_priority_animation_active(
                                    rush_active,
                                    strike_animation_timer,
                                    block_system.active,
                                    block_system.rmb_held,
                                ) && !movement_active
                                {
                                    current_racer_texture = current_idle_texture;
                                    current_movement_direction = MovementDirection::None;
                                }
                            }
                        }
                        if strike_animation_timer > 0.0 {
                            strike_animation_timer -= dt;
                            if strike_animation_timer <= 0.0 {
                                if !block_system.active
                                    && !rush_active
                                    && !movement_active
                                    && !backpedal_active
                                {
                                    current_racer_texture = current_idle_texture;
                                }
                            }
                        }
                        if rush_active {
                            rush_timer -= dt;
                            if rush_timer <= 0.0 {
                                rush_active = false;
                                if !block_system.active && !movement_active && !backpedal_active {
                                    current_racer_texture = current_idle_texture;
                                }
                            }
                        }
                        if rush_cooldown > 0.0 {
                            rush_cooldown -= dt;
                        }
                        if !block_break_animation_active
                            && !movement_active
                            && !backpedal_active
                            && !block_system.active
                            && !block_system.rmb_held
                            && strike_animation_timer <= 0.0
                            && !rush_active
                        {
                            if (fighter.state == RacerState::OnFoot
                                && !key_w_pressed
                                && !key_s_pressed
                                && !key_a_pressed
                                && !key_d_pressed)
                                || (fighter.state == RacerState::OnBike
                                    && !key_w_pressed
                                    && !key_s_pressed
                                    && !key_a_pressed
                                    && !key_d_pressed)
                            {
                                if current_racer_texture != current_idle_texture {
                                    current_racer_texture = current_idle_texture;
                                }
                            }
                        }
                    }
                    let tasks_completed = task_system.update();
					task_system.update_timer(dt);
                    if tasks_completed > 0 {
                        let points_awarded = 10 * tasks_completed;
                        task_reward_notification = Some(TaskRewardNotification {
                            text: format!("+{}", points_awarded),
                            lifetime: 1.5,
                        });
                        award_kill_score(
                            &mut fighter,
                            points_awarded,
                            &mut chatbox,
                            &mut lvl_up_state,
                            &format!("completing {} task(s)", tasks_completed),
                        );
                    }
					
					

                    // Deactivate CPUs on the racetrack if the final task is active
                    // --- RUT ZONE CHECK ---
                    let in_rut_zone = collision_barrier_manager.check_rut(
                        &sbrx_map_system.current_field_id,
                        fighter.x,
                        fighter.y,
                    );					
					
                    let is_racetrack_finale_task_active = task_system
                        .has_task("PROCEED TO THE STARTING LINE")
                        && !task_system.is_task_complete("PROCEED TO THE STARTING LINE");

                    if is_racetrack_finale_task_active
                        && sbrx_map_system.current_field_id == SbrxFieldId(0, 0)
                    {
                        if !racetrack_active {
                            // First frame this condition is met, clear enemies
                            println!(
                                "[FINALE] Racetrack peaceful mode activated. Clearing enemies."
                            );
                            cpu_entities.clear();
                            racetrack_active = true;
                        }
                    } else {
                        // If we leave the field or the task is completed/removed, deactivate peaceful mode
                        if racetrack_active {
                            //println!("[FINALE] Racetrack peaceful mode deactivated.");
                            racetrack_active = false;
                            // Deactivate arena mode only if player leaves the field
                            if endless_arena_mode_active
                                && sbrx_map_system.current_field_id != SbrxFieldId(0, 0)
                            {
                                endless_arena_mode_active = false;
                                endless_arena_timer = 0.0;
                                endless_arena_stage = 1;
                                arena_kill_count = 0;
                                last_arena_milestone = 0; // Reset milestone on field exit								
                                //println!("[ARENA MODE] Player left the racetrack. Deactivating endless arena.");
                            }
                        }
                    }

                    // Force switch to Racer if on the racetrack during the finale
                    if racetrack_active && fighter.fighter_type != FighterType::Racer {
                        //println!("[FINALE] Forcing switch to RACER for the race.");
                        // This logic is adapted from the F1 key press handler
                        fighter_hp_map.insert(fighter.fighter_type, fighter.current_hp);
                        let new_radius = fighter.switch_fighter_type(FighterType::Racer);
                        fixed_crater.radius = new_radius;						
						fighter.stats = fighter_stats_map
							.get(&FighterType::Racer)
							.copied()
							.unwrap_or(combat::stats::RACER_LVL1_STATS);							
                        fighter.max_hp = fighter.stats.defense.hp;
                        fighter.melee_damage = fighter.stats.attack.melee_damage;
                        fighter.ranged_damage = fighter.stats.attack.ranged_damage;
                        fighter.run_speed = fighter.stats.speed.run_speed;
                        fighter.current_hp = fighter.max_hp; // Set to full health for the race
                        fighter_hp_map.insert(FighterType::Racer, fighter.max_hp); // Update map
                        combo_system.is_combo3_stun_disabled = false;
                        fighter.combat_mode = CombatMode::CloseCombat;
                        shift_override_active = false;

                        update_current_textures(
                            &fighter,
                            &racer_textures,
                            &mut current_idle_texture,
                            &mut current_fwd_texture,
                            &mut current_backpedal_texture,
                            &mut current_block_texture,
                            &mut current_block_break_texture,
                            &mut current_ranged_texture,
                            &mut current_ranged_marker_texture,
                            &mut current_ranged_blur_texture,
                            &mut current_rush_texture,
                            &mut current_strike_textures,
							false,
                        );
                        if !block_break_animation_active {
                            current_racer_texture = current_idle_texture;
                        }
                        lvl_up_state = LvlUpState::None;
                    }

                    // Spawn VoidTempest when all survivors are found
                    if task_system.survivors_found >= 10
                        && !void_tempest_spawned_for_survivors
                        && sbrx_map_system.current_field_id == SbrxFieldId(-2, 5)
                    {
                        let void_tempest_exists = cpu_entities
                            .iter()
                            .any(|e| e.variant == CpuVariant::VoidTempest);
                        if !void_tempest_exists && CPU_ENABLED {
                            let (base_hp, base_speed) = cpu_entities
                                .iter()
                                .find(|e| e.variant == CpuVariant::GiantMantis && !e.is_dead())
                                .map_or((250.0, 150.0), |m| (m.max_hp, m.speed));
                            cpu_entities
                                .push(CpuEntity::new_void_tempest(line_y, base_hp, base_speed));
                            void_tempest_spawned_for_survivors = true;
                            chatbox.add_interaction(vec![(
                                "WARNING: STRONG ENCOUNTER",
                                MessageType::Warning,
                            )]);							
                            println!("VoidTempest spawned after all survivors found.");
                        }
                    }

                    if boundary_warning_cooldown > 0.0 {
                        boundary_warning_cooldown -= dt;
                    }

                    if is_paused {
                        continue;
                    }

                    // Racetrack Soldier interaction check (Disabled in Arena Mode)
                    if !crate::config::ARENA_MODE && racetrack_active && !racetrack_soldier_dialogue_triggered {
                        let soldier_x = 300.0;
                        let soldier_y = 650.0;
                        let dx = fighter.x - soldier_x;
                        let dy = fighter.y - soldier_y;
                        if (dx * dx + dy * dy).sqrt() < INFO_POST_INTERACTION_DISTANCE {
                            show_racetrack_soldier_prompt = true;
                        } else {
                            show_racetrack_soldier_prompt = false;
                        }
                    } else {
                        show_racetrack_soldier_prompt = false;
                    }
/*
                    // Draw interaction prompt over soldier if conditions are met
                    if racetrack_active && !racetrack_soldier_dialogue_triggered {
                        let _indicator_text = "[!]";
                        let _font_size = 24;
                        // ... code to draw the [!] ...
                    }
*/
                    show_fort_silo_survivor_prompt = false;
                    if !is_paused
                        && sbrx_map_system.current_field_id == SbrxFieldId(-25, 25)
                        && !fort_silo_survivor.interaction_triggered
                    {
                        let dx = fighter.x - fort_silo_survivor.x;
                        let dy = fighter.y - fort_silo_survivor.y;
                        if (dx * dx + dy * dy).sqrt() < INFO_POST_INTERACTION_DISTANCE {
                            show_fort_silo_survivor_prompt = true;
                        }
                    }

                    // Grand Commander interaction check
                    show_grand_commander_prompt = false;
                    if let Some(area_state) = &current_area {
                        if area_state.area_type == AreaType::Bunker && area_state.floor == -3 {
                            if razor_fiend_defeated_flag && !grand_commander_dialogue_triggered {
                                let gc_x = BUNKER_ORIGIN_X + BUNKER_WIDTH / 2.0;
                                let gc_y = BUNKER_ORIGIN_Y + BUNKER_HEIGHT / 2.0;
                                let dx = fighter.x - gc_x;
                                let dy = fighter.y - gc_y;
                                if (dx * dx + dy * dy).sqrt() < INFO_POST_INTERACTION_DISTANCE {
                                    show_grand_commander_prompt = true;
                                }
                            }
                        }
                    }

                    // Survivor interaction check
                    show_survivor_interaction_prompt = false;
                    nearby_survivor_index = None;
                    if sbrx_map_system.current_field_id == SbrxFieldId(-2, 5) {
                        if let Some(field_survivors) =
                            survivors.get_mut(&sbrx_map_system.current_field_id)
                        {
                            for (i, survivor) in field_survivors.iter().enumerate() {
                                if !survivor.is_rescued
                                    && (!FOG_OF_WAR_ENABLED
                                        || fog_of_war.is_position_visible(
                                            sbrx_map_system.current_field_id,
                                            survivor.x,
                                            survivor.y,
                                        ))
                                {
                                    let dx = fighter.x - survivor.x;
                                    let dy = fighter.y - survivor.y;
                                    if (dx * dx + dy * dy).sqrt() < 150.0 {
                                        // Interaction distance
                                        show_survivor_interaction_prompt = true;
                                        nearby_survivor_index = Some(i);
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    game_time += dt;
                    show_soldier_interaction_prompt = false;
                    if soldier_visible && sbrx_map_system.current_field_id == SbrxFieldId(0, 0) {
                        let dx = fighter.x - random_image_x;
                        let dy = fighter.y - random_image_y;
                        if (dx * dx + dy * dy).sqrt() < soldier_interaction_distance {
                            show_soldier_interaction_prompt = true;
                        }
                    }
                    show_raptor_interaction_prompt = false;
                    if let Some(area_state) = &current_area {
                        // Only show raptor interaction in raptor nest, not in bunker
                        if area_state.area_type == AreaType::RaptorNest {
                            if raptor_is_trapped_in_nest
                                && task_system.is_task_complete("CLEAR RAPTOR NEST: FIELD[X1 Y0]")
                            {
                                let raptor_x = AREA_ORIGIN_X + (AREA_WIDTH / 2.0) + 50.0;
                                let raptor_y = AREA_ORIGIN_Y + (AREA_HEIGHT / 2.0) - 30.0;
                                let dx = fighter.x - raptor_x;
                                let dy = fighter.y - raptor_y;
                                if (dx * dx + dy * dy).sqrt() < RAPTOR_INTERACTION_DISTANCE {
                                    show_raptor_interaction_prompt = true;
                                }
                            }
                        }
                    }
                    show_info_post_prompt = false;
                    if !crate::config::ARENA_MODE 
                        && !racetrack_active
                        && sbrx_map_system.current_field_id == SbrxFieldId(0, 0)
                        && !racetrack_info_post_interacted
                    {
                        let dx = fighter.x - info_post_position.0;
                        let dy = fighter.y - info_post_position.1;
                        if (dx * dx + dy * dy).sqrt() < INFO_POST_INTERACTION_DISTANCE {
                            show_info_post_prompt = true;
                        }
                    }

                    show_finale_info_post_prompt = false;
                    if racetrack_active
                        && sbrx_map_system.current_field_id == SbrxFieldId(0, 0)
                        && !finale_info_post_interacted
                    {
                        let dx = fighter.x - info_post_position.0;
                        let dy = fighter.y - info_post_position.1;
                        if (dx * dx + dy * dy).sqrt() < INFO_POST_INTERACTION_DISTANCE {
                            show_finale_info_post_prompt = true;
                        }
                    }

                    if FOG_OF_WAR_ENABLED {
                        fog_of_war.update_player_visibility(
                            sbrx_map_system.current_field_id,
                            fighter.x,
                            fighter.y,
                            game_time,
                        );
                    }
					
                    // Mark Rocketbay task as found when entering these fields
                    if sbrx_map_system.current_field_id == SbrxFieldId(-2, 5) 
                        || sbrx_map_system.current_field_id == SbrxFieldId(-25, 25) {
                        task_system.mark_rocketbay_found();
                    }					
					
                    match ambient_track_state {
                        AmbientTrackState::Background => {
                            if let Some(track_name) = handle_ambient_playlist(&audio_manager, &mut current_bgm_sink, &mut ambient_playlist_index) {
                                track_notification = Some(TrackNotification {
                                    track_name,
                                    lifetime: 3.0, // Display for 3 seconds
                                });
                            }
                        }
                        AmbientTrackState::Crickets => {
                            if current_bgm_sink.is_some() { if let Some(s) = current_bgm_sink.take() { s.stop(); } }
                            if crickets_sound_sink.is_none() {
                                if let Ok(s) = audio_manager.play_sfx_loop("crickets") { crickets_sound_sink = Some(s); }
                            }
                        }
                        AmbientTrackState::Muted => {
                            if let Some(s) = current_bgm_sink.take() { s.stop(); }
                            if let Some(s) = crickets_sound_sink.take() { s.stop(); }
                        }
                    }				
                    let (current_min_x, current_max_x, current_min_y, current_max_y) =
                        if let Some(ref area_state) = current_area {
                            let (width, height, origin_x, origin_y) = match area_state.area_type {
                                AreaType::RaptorNest => {
                                    (AREA_WIDTH, AREA_HEIGHT, AREA_ORIGIN_X, AREA_ORIGIN_Y)
                                }
                                AreaType::Bunker => (
                                    BUNKER_WIDTH,
                                    BUNKER_HEIGHT,
                                    BUNKER_ORIGIN_X,
                                    BUNKER_ORIGIN_Y,
                                ),
                            };
                            (origin_x, origin_x + width, origin_y, origin_y + height)
                        } else {
                            let min_y_world = if fighter.state == RacerState::OnBike {
                                line_y
                            } else {
                                MIN_Y
                            };
                            (MIN_X, MAX_X, min_y_world, MAX_Y)
                        };
                    if fighter.state == RacerState::OnFoot
                        && !block_system.active
                        && !rush_active
                        && strike_animation_timer <= 0.0
                        && fighter.knockback_duration <= 0.0
                        && !block_system.is_stun_locked()
                    {
                        let mut dx = 0.0;
                        let mut dy = 0.0;
                        if key_w_pressed {
                            dy -= 1.0;
                        }
                        if key_s_pressed {
                            dy += 1.0;
                        }
                        if key_a_pressed {
                            dx -= 1.0;
                        }
                        if key_d_pressed {
                            dx += 1.0;
                        }
                        let current_move_direction = (dx, dy);
                        if current_move_direction != (0.0, 0.0) {
                            if current_move_direction == last_move_direction {
                                continuous_move_timer += dt;
                            } else {
                                continuous_move_timer = 0.0;
                                is_in_continuous_move = false;
                            }
                            if continuous_move_timer > ON_FOOT_HOLD_DURATION {
                                is_in_continuous_move = true;
                            }
                            if is_in_continuous_move {
                                let (world_mouse_x, _) = screen_to_world(&camera, mouse_x, mouse_y);
                                let moving_horizontally = dx != 0.0;
                                let moving_left = dx < 0.0;
                                let is_moving_forward = if moving_horizontally {
                                    if moving_left {
                                        world_mouse_x < fighter.x
                                    } else {
                                        world_mouse_x > fighter.x
                                    }
                                } else {
                                    true
                                };
                                if is_moving_forward {
                                    current_racer_texture = current_fwd_texture;
                                } else {
                                    current_racer_texture = current_backpedal_texture;
                                }
                                let mut move_vec = Vec2d::new(dx, dy);
                                let mag = (move_vec.x.powi(2) + move_vec.y.powi(2)).sqrt();
                                if mag > 0.0 {
                                    move_vec.x /= mag;
                                    move_vec.y /= mag;
                                }
                                let boost_mult = if fighter.fighter_type == FighterType::Racer && fighter.boost && shift_held { 1.5 } else { 1.0 };
								let rut_mult = if in_rut_zone { 0.5 } else { 1.0 };
                                let backpedal_mult = if !is_moving_forward { 0.5 } else { 1.0 };
								let atomic_mult = if fighter.invincible_timer > 1.0 { 2.0 } else { 1.0 }; // 25% speed buff during ATOMIC-STATE
								let speed_mult = boost_mult * backpedal_mult * rut_mult * atomic_mult;
                                fighter.x += move_vec.x * fighter.run_speed * speed_mult * dt;
                                fighter.y += move_vec.y * fighter.run_speed * speed_mult * dt;
                                fighter.x = fighter.x.clamp(current_min_x, current_max_x);
                                fighter.y = fighter.y.clamp(current_min_y, current_max_y);
                            }
                        } else {
                            continuous_move_timer = 0.0;
                            is_in_continuous_move = false;
                        }
                        last_move_direction = current_move_direction;
                    } else {
                        continuous_move_timer = 0.0;
                        is_in_continuous_move = false;
                        last_move_direction = (0.0, 0.0);
                    }
                    for text in &mut damage_texts {
                        text.lifetime -= dt;
                        text.y -= 30.0 * dt;
                    }
                    damage_texts.retain(|text| text.lifetime > 0.0);
                    if current_area.is_none() {
                        if (racetrack_active || endless_arena_mode_active)
                            && sbrx_map_system.current_field_id == SbrxFieldId(0, 0)
                        {
                            // Player is on the racetrack during the finale. Lock them in.
                            let hit_boundary = fighter.x >= config::boundaries::MAX_X
                                || fighter.x <= config::boundaries::MIN_X
                                || fighter.y >= config::boundaries::MAX_Y
                                || fighter.y <= config::boundaries::MIN_Y;

                            if hit_boundary {
                                // Clamp position
                                fighter.x = fighter.x.clamp(
                                    config::boundaries::MIN_X + 10.0,
                                    config::boundaries::MAX_X - 10.0,
                                );
                                fighter.y = fighter.y.clamp(
                                    config::boundaries::MIN_Y + 10.0,
                                    config::boundaries::MAX_Y - 10.0,
                                );

                                // Show warning message
                                if boundary_warning_cooldown <= 0.0 {
                                    chatbox.add_interaction(vec![(
                                        "PROCEED TO THE STARTING LINE",
                                        MessageType::Notification,
                                    )]);
                                    boundary_warning_cooldown = BOUNDARY_WARNING_COOLDOWN_TIME;
                                }
                            }
                        } else {
                            // Original field transition logic for all other cases
                            let can_transition = sbrx_map_system.current_field_id
                                != SbrxFieldId(0, 0)
                                || soldier_has_joined
                                || racetrack_active;

                            let allowed_fields =
                                get_allowed_fields(soldier_has_joined, raptor_has_joined);
                            let has_field_restrictions = !allowed_fields.is_empty();

                            let is_saucer_defeated = task_system.flying_saucer_defeated;
                            // Block ground access to Fort Silo until flying saucer is defeated
                            let fort_silo_ground_restriction = !is_saucer_defeated
                                && (fighter.state == RacerState::OnFoot
                                    || fighter.state == RacerState::OnBike);

                            if can_transition {
                                let mut dx_field_transition = 0;
                                let mut dy_field_transition = 0;

                                // Rocketbay (x-2, y5) Lockdown Check
                                let is_in_rocketbay =
                                    sbrx_map_system.current_field_id == SbrxFieldId(-2, 5);
                                let void_tempest_active = cpu_entities
                                    .iter()
                                    .any(|e| e.variant == CpuVariant::VoidTempest && !e.is_dead());
                                let rocketbay_lockdown = is_in_rocketbay && void_tempest_active;

                                if fighter.x >= config::boundaries::MAX_X {
                                    let target_field = SbrxFieldId(
                                        sbrx_map_system.current_field_id.0 + 1,
                                        sbrx_map_system.current_field_id.1,
                                    );
                                    if rocketbay_lockdown {
                                        fighter.x = config::boundaries::MAX_X - 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                                "DEFEAT THE VOID TEMPEST TO ESCAPE",
                                                MessageType::Notification,
                                            )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    } else if fort_silo_ground_restriction
                                        && target_field == SbrxFieldId(-25, 25)
                                    {
                                        fighter.x = config::boundaries::MAX_X - 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                            "A GRAVITATIONAL FORCE PREVENTS YOU FROM \nENTERING BY FOOT.",
                                            MessageType::Notification,
                                        )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    } else if racetrack_active
                                        || !has_field_restrictions
                                        || allowed_fields.contains(&target_field)
                                    {
                                        fighter.x = config::boundaries::MIN_X + 1.0;
                                        dx_field_transition = 1;
                                    } else {
                                        fighter.x = config::boundaries::MAX_X - 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                                "FIELD RESTRICTED",
                                                MessageType::Notification,
                                            )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    }
                                } else if fighter.x <= config::boundaries::MIN_X {
                                    let target_field = SbrxFieldId(
                                        sbrx_map_system.current_field_id.0 - 1,
                                        sbrx_map_system.current_field_id.1,
                                    );
                                    if rocketbay_lockdown {
                                        fighter.x = config::boundaries::MIN_X + 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                                "DEFEAT THE VOID TEMPEST TO ESCAPE",
                                                MessageType::Notification,
                                            )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    } else if fort_silo_ground_restriction
                                        && target_field == SbrxFieldId(-25, 25)
                                    {
                                        fighter.x = config::boundaries::MIN_X + 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                            "A GRAVITATIONAL FORCE PREVENTS YOU FROM \nENTERING BY FOOT.",
                                            MessageType::Notification,
                                        )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    } else if racetrack_active
                                        || !has_field_restrictions
                                        || allowed_fields.contains(&target_field)
                                    {
                                        fighter.x = config::boundaries::MAX_X - 1.0;
                                        dx_field_transition = -1;
                                    } else {
                                        fighter.x = config::boundaries::MIN_X + 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                                "FIELD RESTRICTED",
                                                MessageType::Notification,
                                            )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    }
                                }
                                if fighter.y >= config::boundaries::MAX_Y {
                                    let target_field = SbrxFieldId(
                                        sbrx_map_system.current_field_id.0,
                                        sbrx_map_system.current_field_id.1 - 1,
                                    );
                                    if rocketbay_lockdown {
                                        fighter.y = config::boundaries::MAX_Y - 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                                "DEFEAT THE VOID TEMPEST TO ESCAPE",
                                                MessageType::Notification,
                                            )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    } else if fort_silo_ground_restriction
                                        && target_field == SbrxFieldId(-25, 25)
                                    {
                                        fighter.y = config::boundaries::MAX_Y - 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                            "A GRAVITATIONAL FORCE PREVENTS YOU FROM \nENTERING BY FOOT.",
                                            MessageType::Notification,
                                        )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    } else if racetrack_active
                                        || !has_field_restrictions
                                        || allowed_fields.contains(&target_field)
                                    {
                                        fighter.y = config::boundaries::MIN_Y + 1.0;
                                        dy_field_transition = -1;
                                    } else {
                                        fighter.y = config::boundaries::MAX_Y - 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                                "FIELD RESTRICTED",
                                                MessageType::Notification,
                                            )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    }
                                } else if fighter.y <= config::boundaries::MIN_Y {
                                    let target_field = SbrxFieldId(
                                        sbrx_map_system.current_field_id.0,
                                        sbrx_map_system.current_field_id.1 + 1,
                                    );
                                    if rocketbay_lockdown {
                                        fighter.y = config::boundaries::MIN_Y + 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                                "DEFEAT THE VOID TEMPEST TO ESCAPE",
                                                MessageType::Notification,
                                            )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    } else if fort_silo_ground_restriction
                                        && target_field == SbrxFieldId(-25, 25)
                                    {
                                        fighter.y = config::boundaries::MIN_Y + 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                            "A GRAVITATIONAL FORCE PREVENTS YOU FROM 
											ENTERING BY FOOT.",
                                            MessageType::Notification,
                                        )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    } else if racetrack_active
                                        || !has_field_restrictions
                                        || allowed_fields.contains(&target_field)
                                    {
                                        fighter.y = config::boundaries::MAX_Y - 1.0;
                                        dy_field_transition = 1;
                                    } else {
                                        fighter.y = config::boundaries::MIN_Y + 10.0;
                                        if boundary_warning_cooldown <= 0.0 {
                                            chatbox.add_interaction(vec![(
                                                "FIELD RESTRICTED",
                                                MessageType::Notification,
                                            )]);
                                            boundary_warning_cooldown =
                                                BOUNDARY_WARNING_COOLDOWN_TIME;
                                        }
                                    }
                                }
                                if dx_field_transition != 0 || dy_field_transition != 0 {
                                    sbrx_map_system.transition_field_by_delta(
                                        dx_field_transition,
                                        dy_field_transition,
                                    );

                                    // --- BUG FIX: Clear entities from previous field to allow new spawns ---
                                    // This prevents the entity cap from being filled by enemies from the previous field,
                                    // ensuring field-specific spawns (like Night Reavers) can occur.
                                    cpu_entities.clear();

                                    last_field_entry_point = (fighter.x, fighter.y);

                                    // Spawn T-Rex if active and entering its field
                                    if t_rex_is_active
                                        && sbrx_map_system.current_field_id == SbrxFieldId(1, 0)
                                        && CPU_ENABLED
                                    {
                                        let t_rex_exists = cpu_entities
                                            .iter()
                                            .any(|e| e.variant == CpuVariant::TRex && !e.is_dead());
                                        if !t_rex_exists && cpu_entities.len() < 10 {
                                            let t_rex_x =
                                                safe_gen_range(MIN_X, MAX_X, "T-Rex field entry x");
                                            let t_rex_y =
                                                safe_gen_range(MIN_X, MAX_Y, "T-Rex field entry y");
                                            cpu_entities
                                                .push(CpuEntity::new_t_rex(t_rex_x, t_rex_y));
                                            println!("Respawned T-Rex when entering field x1 y0");
											chatbox.add_interaction(vec![(
												"WARNING: STRONG ENCOUNTER",
												MessageType::Warning,
											)]);											
                                        }
                                    }

                                    if sbrx_map_system.current_field_id == SbrxFieldId(-2, 5) {
                                        if !task_system.has_task("FIND 10 SURVIVORS") {
                                            task_system.add_task("FIND 10 SURVIVORS");
                                        }

                                        let has_night_reavers = cpu_entities
                                            .iter()
                                            .any(|e| e.variant == CpuVariant::NightReaver);
                                        if !has_night_reavers && CPU_ENABLED {
                                            println!(
                                                "Spawning Night Reavers in ROCKETBAY field x-2 y5"
                                            );
                                            for _ in 0..4 {
                                                if cpu_entities.len() < 10 {
                                                    let reaver_x = safe_gen_range(
                                                        MIN_X,
                                                        MAX_X,
                                                        "NightReaver x",
                                                    );
                                                    let reaver_y = safe_gen_range(
                                                        MIN_Y,
                                                        MAX_Y,
                                                        "NightReaver y",
                                                    );
                                                    cpu_entities.push(CpuEntity::new_night_reaver(
                                                        reaver_x, reaver_y,
                                                    ));
                                                }
                                            }
                                        }

                                        if !rocketbay_dialogue_triggered {
                                            // TASK 1: Force revive downed fighters for this dialogue
                                            if !downed_fighters.is_empty() {
                                                //println!("[DIALOGUE REVIVAL] Reviving all downed fighters for interaction.");
                                                let fighters_to_revive = downed_fighters.clone();
                                                downed_fighters.clear(); // Clear the list
                                                revival_kill_score = 0; // Reset counter

                                                let mut revival_messages = Vec::new();

                                                for fighter_to_revive in fighters_to_revive {
                                                    let max_hp = match fighter_to_revive {
                                                        FighterType::Racer => {
                                                            RACER_LVL1_STATS.defense.hp
                                                        }
                                                        FighterType::Soldier => {
                                                            SOLDIER_LVL1_STATS.defense.hp
                                                        }
                                                        FighterType::Raptor => {
                                                            RAPTOR_LVL1_STATS.defense.hp
                                                        }
                                                    };
                                                    // Revive with 25% health to be consistent with kill-based revival
                                                    let revived_hp = max_hp * 0.25;
                                                    fighter_hp_map
                                                        .insert(fighter_to_revive, revived_hp);

                                                    let message = format!(
                                                        "{} IS BACK IN THE FIGHT!",
                                                        match fighter_to_revive {
                                                            FighterType::Racer => "RACER",
                                                            FighterType::Soldier => "SOLDIER",
                                                            FighterType::Raptor => "RAPTOR",
                                                        }
                                                    );
                                                    revival_messages.push(message);
                                                }

                                                // Now add all messages
                                                for msg in revival_messages {
                                                    chatbox.add_interaction(vec![(
                                                        &msg,
                                                        MessageType::Notification,
                                                    )]);
                                                }
                                            }

                                            chatbox.add_interaction(vec![
												("THE RAPTOR GROWLS", MessageType::Notification),  
												("-SOLDIER-", MessageType::Info), 
												("IT'S TOO QUIET. BE ON YOUR GUARD.", MessageType::Dialogue), 
												("-RACER-", MessageType::Info), 
												("I SEE SOMEONE ON THE GROUND OVER THERE.", MessageType::Dialogue)
											]);
										
                                            rocketbay_dialogue_triggered = true;

                                            // Spawn survivors if not already spawned for this field
                                            // (moved outside rocketbay_dialogue_triggered check so it works with or without fog)
                                            survivors
                                                .entry(sbrx_map_system.current_field_id)
                                                .or_insert_with(|| {
                                                    let mut new_survivors = Vec::new();
                                                    let num_survivors_to_spawn = 10;
                                                    for _ in 0..num_survivors_to_spawn {
                                                        new_survivors.push(Survivor {
                                                            x: safe_gen_range(
                                                                MIN_X,
                                                                MAX_X,
                                                                "survivor x",
                                                            ),
                                                            y: safe_gen_range(
                                                                MIN_Y,
                                                                MAX_Y,
                                                                "survivor y",
                                                            ),
                                                            fighter_type: FighterType::Soldier, // Only soldier for now
                                                            is_rescued: false,
                                                        });
                                                    }
                                                    new_survivors
                                                });
                                        }
                                    }

                                    // Spawn Light Reavers and Night Reavers when entering Fort Silo field via field transition
									if sbrx_map_system.current_field_id == SbrxFieldId(-25, 25) 
										&& !task_system.is_task_complete("SPEAK TO THE GRAND COMMANDER") 
									{
                                        let has_light_reavers = cpu_entities
                                            .iter()
                                            .any(|e| e.variant == CpuVariant::LightReaver);
                                        let has_night_reavers = cpu_entities
                                            .iter()
                                            .any(|e| e.variant == CpuVariant::NightReaver);
                                        if (!has_light_reavers || !has_night_reavers) && CPU_ENABLED {
                                            println!("Spawning Reavers in Fort Silo field x-25 y25 (via field transition)");
                                            for _ in 0..3 {
                                                if cpu_entities.len() < 10 {
                                                    let reaver_x = safe_gen_range(
                                                        MIN_X,
                                                        MAX_X,
                                                        "LightReaver x",
                                                    );
                                                    let reaver_y = safe_gen_range(
                                                        MIN_Y,
                                                        MAX_Y,
                                                        "LightReaver y",
                                                    );
                                                    cpu_entities.push(CpuEntity::new_light_reaver(
                                                        reaver_x, reaver_y,
                                                    ));
                                                }
                                            }
                                            for _ in 0..3 {
                                                if cpu_entities.len() < 10 {
                                                    let reaver_x = safe_gen_range(
                                                        MIN_X,
                                                        MAX_X,
                                                        "NightReaver x",
                                                    );
                                                    let reaver_y = safe_gen_range(
                                                        MIN_Y,
                                                        MAX_Y,
                                                        "NightReaver y",
                                                    );
                                                    cpu_entities.push(CpuEntity::new_night_reaver(
                                                        reaver_x, reaver_y,
                                                    ));
                                                }
                                            }											
                                        }
                                    }

                                    // --- NEW: Generate ground assets for the new field ---
                                    let current_field_id = sbrx_map_system.current_field_id;
                                    if !ground_asset_manager.is_exclusion_zone(&current_field_id)
                                        && !placed_ground_assets.contains_key(&current_field_id)
                                    {
                                        println!(
                                            "Generating ground assets for field {:?}",
                                            current_field_id
                                        );
                                        let mut assets_for_field = Vec::new();
                                        let num_assets_to_spawn =
                                            rand::rng().random_range(30..=60);

                                        for _ in 0..num_assets_to_spawn {
                                            if let Some(asset_to_spawn) =
                                                ground_asset_manager.get_random_asset()
                                            {
                                                let asset_x = safe_gen_range(MIN_X, MAX_X, "ground asset x");
                                                let asset_y = safe_gen_range(line_y + 150.0, MAX_Y, "ground asset y");
 
                                                // --- FLATLINE_field.x1y0 Raptor Nest Exclusion Zone ---
                                                if current_field_id == SbrxFieldId(1, 0) {
                                                    let center_x = 2500.0;
                                                    let center_y = 1895.0;
                                                    let half_w = 1415.0 / 2.0;
                                                    let half_h = 500.0 / 2.0;
 
                                                    // If asset spawns inside the raptor nest box, skip this iteration
                                                    if asset_x >= center_x - half_w && asset_x <= center_x + half_w &&
                                                       asset_y >= center_y - half_h && asset_y <= center_y + half_h {
                                                        continue; 
                                                    }
                                                }												
												
                                                let new_asset = PlacedAsset {
                                                    texture_name: asset_to_spawn.name.to_string(),
                                                    x: asset_x,
                                                    y: asset_y,
                                                };
                                                assets_for_field.push(new_asset);
                                            }
                                        }
                                        // Sort by y-coordinate for correct draw order (painter's algorithm)
                                        assets_for_field.sort_by(|a, b| {
                                            a.y.partial_cmp(&b.y)
                                                .unwrap_or(std::cmp::Ordering::Equal)
                                        });
                                        placed_ground_assets
                                            .insert(current_field_id, assets_for_field);
                                    }

                                    raptor_nests.clear();
                                    if FOG_OF_WAR_ENABLED
                                        && fog_of_war
                                            .is_fog_enabled(sbrx_map_system.current_field_id)
                                    {
                                        let blood_idol_spawn_chance =
                                            if has_blood_idol_fog_spawned_once {
                                                0.03
                                            } else {
                                                0.25
                                            };
                                        if CPU_ENABLED
                                            && safe_gen_range(
                                                0.0,
                                                1.0,
                                                "BLOOD IDOL fog spawn chance",
                                            ) < blood_idol_spawn_chance
                                        {
                                            if !has_blood_idol_fog_spawned_once {
                                                println!("25% fog-of-war spawn triggered! Spawning BLOOD IDOL in field {:?}", sbrx_map_system.current_field_id);
                                                has_blood_idol_fog_spawned_once = true;
                                            } else {
                                                println!("3% fog-of-war spawn triggered! Spawning BLOOD IDOL in field {:?}", sbrx_map_system.current_field_id);
                                            }
                                            let (base_hp, base_speed) = cpu_entities
                                                .iter()
                                                .find(|e| {
                                                    e.variant == CpuVariant::GiantMantis
                                                        && !e.is_dead()
                                                })
                                                .map_or((250.0, 150.0), |m| (m.max_hp, m.speed));
                                            cpu_entities.push(CpuEntity::new_blood_idol(
                                                line_y, base_hp, base_speed,
                                            ));
                                        }
                                    }
                                    check_and_display_demonic_presence(
                                        &sbrx_map_system.current_field_id,
                                        &cpu_entities,
                                        &mut chatbox,
                                        &fog_of_war,
                                    );
                                }
                            } else {
                                let hit_boundary = fighter.x >= config::boundaries::MAX_X
                                    || fighter.x <= config::boundaries::MIN_X
                                    || fighter.y >= config::boundaries::MAX_Y
                                    || fighter.y <= config::boundaries::MIN_Y;
                                if hit_boundary && boundary_warning_cooldown <= 0.0 {
                                    fighter.x = fighter.x.clamp(
                                        config::boundaries::MIN_X + 10.0,
                                        config::boundaries::MAX_X - 10.0,
                                    );
                                    fighter.y = fighter.y.clamp(
                                        config::boundaries::MIN_Y + 10.0,
                                        config::boundaries::MAX_Y - 10.0,
                                    );
                                    chatbox.add_interaction(vec![(
                                        "SOMEONE NEAR THE TRACK CALLS FOR HELP",
                                        MessageType::Notification,
                                    )]);
                                    boundary_warning_cooldown = BOUNDARY_WARNING_COOLDOWN_TIME;
                                }
                            }
                        }
                    }
                    if current_area.is_none()
                        && sbrx_map_system.current_field_id == SbrxFieldId(1, 0)
                        && raptor_nests.is_empty()
                    {
                        println!("Spawning a raptor nest in FLATLINE_field.x1y0");
                        let nest = RaptorNest::new();
                        if cpu_entities.len() < 10 {
                            cpu_entities.push(CpuEntity::new_raptor(nest.x - 75.0, nest.y));
                        }
                        if cpu_entities.len() < 10 {
                            cpu_entities.push(CpuEntity::new_raptor(nest.x + 75.0, nest.y));
                        }
                        raptor_nests.push(nest);
                    }
                    // Spawn bunker entrance (only once)
                    if current_area.is_none()
                        && sbrx_map_system.current_field_id == SbrxFieldId(-25, 25)
                        && fort_silo_bunkers.is_empty()
                    {
                        println!("Spawning bunker entrance at Fort Silo field x-25 y25");
                        // Position the bunker entrance at the fort silo building location
                        // Based on the image, placing it near the bottom-right of the field
                        let bunker_x = 3500.0; // Adjusted for better placement
                        let bunker_y = 2800.0; // was 3000
                        fort_silo_bunkers.push((bunker_x, bunker_y));
                    }
                    if current_area.is_none()
                        && sbrx_map_system.current_field_id == SbrxFieldId(-25, 25)
                        && !task_system.is_task_complete("SPEAK TO THE GRAND COMMANDER")
                    {
                        let has_light_reavers = cpu_entities
                            .iter()
                            .any(|e| e.variant == CpuVariant::LightReaver);
                        let has_night_reavers = cpu_entities
                            .iter()
                            .any(|e| e.variant == CpuVariant::NightReaver);
                        if (!has_light_reavers || !has_night_reavers) && CPU_ENABLED {
                            println!("Spawning Reavers in Fort Silo field x-25 y25");
                            for _ in 0..3 {
                                if cpu_entities.len() < 10 {
                                    let reaver_x = safe_gen_range(MIN_X, MAX_X, "LightReaver x");
                                    let reaver_y = safe_gen_range(MIN_Y, MAX_Y, "LightReaver y");
                                    cpu_entities
                                        .push(CpuEntity::new_light_reaver(reaver_x, reaver_y));
                                }
                            }
                            for _ in 0..3 {
                                if cpu_entities.len() < 10 {
                                    let reaver_x = safe_gen_range(MIN_X, MAX_X, "NightReaver x");
                                    let reaver_y = safe_gen_range(MIN_Y, MAX_Y, "NightReaver y");
                                    cpu_entities
                                        .push(CpuEntity::new_night_reaver(reaver_x, reaver_y));
                                }
                            }
                        }
                    }
                    fighter.update(dt, line_y);
                    fixed_crater.x = fighter.x;
                    fixed_crater.y = fighter.y;
                    stars.iter_mut().for_each(|s| s.update(dt));

                    for cpu in &mut cpu_entities {
                        if racetrack_active && !endless_arena_mode_active {
                            continue; // Skip AI logic for all CPUs
                        }
                        let result = cpu.update(fighter.x, fighter.y, dt, line_y, &audio_manager);

                        // Handle skill effects returned from the update
                        if let Some(damage) = result.damage_to_player {
                            // Check if blocked first
                            if block_system.active
                                && !block_system.block_broken
                                && !block_system.block_fatigue
                            {
                                audio_manager.play_sound_effect("block").ok();
                                // Consume block point for skill block? Or just negate damage.
                                // Let's negate damage for now to fix the bug.
                            } else if fighter.invincible_timer <= 0.0 {
                                fighter.current_hp -= damage;
                                damage_texts.push(DamageText {
                                    text: format!("{:.0}", damage),
                                    x: fighter.x,
                                    y: fighter.y - 70.0,
                                    color: [1.0, 0.0, 0.0, 1.0], // Red for skill damage
                                    lifetime: 0.5,
                                });

                                // Dismount if hit on bike
                                if fighter.state == RacerState::OnBike {
                                    println!("Dismounted by enemy skill damage!");
                                    fighter.state = RacerState::OnFoot;

                                    if fighter.fighter_type != FighterType::Raptor {
                                        sbrx_bike.respawn(fighter.x, fighter.y);
                                    }

                                    let tex_set = match fighter.fighter_type {
                                        FighterType::Racer => &racer_textures,
                                        FighterType::Soldier => &soldier_textures,
                                        FighterType::Raptor => &raptor_textures,
                                    };
                                    update_current_textures(
                                        &fighter,
                                        tex_set,
                                        &mut current_idle_texture,
                                        &mut current_fwd_texture,
                                        &mut current_backpedal_texture,
                                        &mut current_block_texture,
                                        &mut current_block_break_texture,
                                        &mut current_ranged_texture,
                                        &mut current_ranged_marker_texture,
                                        &mut current_ranged_blur_texture,
                                        &mut current_rush_texture,
                                        &mut current_strike_textures,
										shift_held,
                                    );
                                    current_racer_texture = current_idle_texture;
                                }
                            }
                        }

                        if let Some(effect) = result.visual_effect {
                            match effect {
                                VisualEffect::FlickerStrike {
                                    from_x,
                                    from_y,
                                    to_x,
                                    to_y,
                                } => {
                                    // Trigger a strike visual at the destination
                                    strike.trigger(to_x, to_y);
                                    strike.timer = 0.15; // A quick flash

                                    active_visual_effects.push(FlickerStrikeEffectInstance {
                                        x: from_x,
                                        y: from_y,
                                        lifetime: 0.2, // Short duration for the effect
                                        max_lifetime: 0.2,
                                    });
                                }
                                VisualEffect::ShootPulseOrb {
                                    start_x,
                                    start_y,
                                    target_x,
                                    target_y,
                                } => {
                                    pulse_orbs
                                        .push(PulseOrb::new(start_x, start_y, target_x, target_y));
                                    audio_manager.play_sound_effect("firearm").ok();
                                }
                            }
                        }
                    }

                    // Update Pulse Orbs
                    for orb in &mut pulse_orbs {
                        orb.update(dt);

                        // Check collision with player
                        let dx = orb.x - fighter.x;
                        let dy = orb.y - fighter.y;
                        let dist_sq = dx * dx + dy * dy;
                        let collision_radius = orb.radius + 25.0; // Approx player radius

                        if orb.active && dist_sq < collision_radius * collision_radius {
                            // Check for Block
                            let projectile_blocked = block_system.process_projectile_block(
                                &mut fighter,
                                &audio_manager,
                                game_time,
                            );

                            if projectile_blocked {
                                orb.active = false;
                            } else if fighter.invincible_timer <= 0.0 {
                                let damage = 20.0; // pulse_orb damage
                                fighter.current_hp -= damage;
                                audio_manager.play_sound_effect("hit").ok();
                                damage_texts.push(DamageText {
                                    text: format!("{:.0}", damage),
                                    x: fighter.x,
                                    y: fighter.y - 70.0,
                                    color: [1.0, 0.0, 0.0, 1.0],
                                    lifetime: 0.5,
                                });

                                orb.active = false;

                                // Pulse Orb Death Logic
                                if fighter.current_hp <= 0.0 {
                                    lmb_held = false;
                                    melee_rapid_fire_timer = 0.0;
                                    soldier_rapid_fire_timer = 0.0;
									
                                    if let Some(sink) = bike_accelerate_sound_sink.take() { sink.stop(); }
                                    if let Some(sink) = bike_idle_sound_sink.take() { sink.stop(); }									

                                    fighter_hp_map.insert(fighter.fighter_type, 0.0);

                                    if fighter.state == RacerState::OnBike {
                                        fighter.state = RacerState::OnFoot;
                                        if fighter.fighter_type != FighterType::Raptor {
                                            sbrx_bike.respawn(fighter.x, fighter.y);
                                        }
                                    }

                                    block_break_animation_active = false;
                                    block_system = BlockSystem::new(20);

                                    let mut group_members = vec![FighterType::Racer];
                                    if soldier_has_joined {
                                        group_members.push(FighterType::Soldier);
                                    }
                                    if raptor_has_joined {
                                        group_members.push(FighterType::Raptor);
                                    }

                                    let has_survivors = group_members.iter().any(|ft| {
                                        !downed_fighters.contains(ft) && *ft != fighter.fighter_type
                                    });

                                    let death_type = DeathType::NightReaver; // Pulse Orb is a Night Reaver skill

                                    if group_members.len() > 1 && has_survivors {
                                        game_state = GameState::DeathScreenGroup {
                                            death_type,
                                            downed_fighter_type: fighter.fighter_type,
                                        };
                                        death_screen_cooldown = DEATH_SCREEN_COOLDOWN_TIME;
                                        if !downed_fighters.contains(&fighter.fighter_type) {
                                            downed_fighters.push(fighter.fighter_type);
                                        }
                                        revival_kill_score = 0;
                                    } else {
                                        if !downed_fighters.contains(&fighter.fighter_type) {
                                            downed_fighters.push(fighter.fighter_type);
                                        }
                                        game_state = GameState::DeathScreen(death_type);
                                        death_screen_cooldown = DEATH_SCREEN_COOLDOWN_TIME;
                                    }
                                    audio_manager.play_sound_effect("death").ok();
                                }
                            }
                        }
                    }
                    pulse_orbs.retain(|o| o.active);

                    let is_on_bike = fighter.state == RacerState::OnBike;
                    let is_moving_on_bike_input =
                        key_w_pressed || key_s_pressed || key_a_pressed || key_d_pressed;
                    let is_stunned = block_system.is_stun_locked();
                    let is_raptor_mounted =
                        fighter.fighter_type == FighterType::Raptor && is_on_bike;
                    let should_play_bike_accelerate =
                        is_on_bike && is_moving_on_bike_input && !is_stunned && !is_raptor_mounted;
                    let should_play_bike_idle =
                        is_on_bike && !is_moving_on_bike_input && !is_stunned && !is_raptor_mounted;
                    if should_play_bike_accelerate {
                        if bike_accelerate_sound_sink.is_none() {
                            if let Some(idle_sink) = bike_idle_sound_sink.take() {
                                idle_sink.stop();
                            }
                            match audio_manager.play_sfx_loop("bike_accelerate") {
                                Ok(sink) => bike_accelerate_sound_sink = Some(sink),
                                Err(e_msg) => {
                                    eprintln!("Failed to play bike_accelerate: {}", e_msg)
                                }
                            }
                        }
                    } else {
                        if let Some(accel_sink) = bike_accelerate_sound_sink.take() {
                            accel_sink.stop();
                        }
                    }
                    if should_play_bike_idle {
                        if bike_idle_sound_sink.is_none() {
                            if let Some(accel_sink) = bike_accelerate_sound_sink.take() {
                                accel_sink.stop();
                            }
                            match audio_manager.play_sfx_loop("bike_idle") {
                                Ok(sink) => bike_idle_sound_sink = Some(sink),
                                Err(e_msg) => eprintln!("Failed to play bike_idle: {}", e_msg),
                            }
                        }
                    } else {
                        if let Some(idle_sink) = bike_idle_sound_sink.take() {
                            idle_sink.stop();
                        }
                    }
                    spheres.retain_mut(|s| {
                        s.update(dt);
                        if s.check_explosion_collision(fighter.x, fighter.y)
                            && matches!(game_state, GameState::Playing)
                        {
                            game_state = GameState::DeathScreen(DeathType::Meteorite);
                            death_screen_cooldown = DEATH_SCREEN_COOLDOWN_TIME;
                            audio_manager.play_sound_effect("death").ok();
                            lmb_held = false; // Stop rapid fire on death
                            melee_rapid_fire_timer = 0.0;
                            soldier_rapid_fire_timer = 0.0;
                        }
                        s.is_visible()
                    });
                    if !PERFORMANCE_MODE {
                        spawn_timer += dt;
                        if spawn_timer >= next_spawn {
                            spheres.push(Box::new(MovingSphere::new(
                                safe_gen_range(MIN_X + 50.0, MAX_X - 50.0, "Sphere x"), // meteor_areaX
                                -10.0,
                                safe_gen_range(1250.0, 300.0, "Sphere speed"),
                                safe_gen_range(25.0, 50.0, "Sphere size"),
                                safe_gen_range(line_y + 20.0, screen_height, "Sphere crash_y"), // meteor_areaY
                            )));
                            spawn_timer = 0.0;
                            next_spawn = safe_gen_range(0.3, 1.5, "Next spawn time");
                        }
                    }
                    if sbrx_map_system.current_field_id == fighter_jet_current_sbrx_location {
                        if fighter_jet_instance.is_none() {
                            fighter_jet_instance = Some(FighterJet {
                                x: fighter_jet_world_x,
                                y: fighter_jet_world_y,
                            });
                            println!("fighter_jet object created/visible for sbrx plane '{}', field {:?} at ({:.2}, {:.2})", sbrx_map_system.current_plane_name, fighter_jet_current_sbrx_location, fighter_jet_world_x, fighter_jet_world_y);
                        } else {
                            if let Some(sj_mut) = fighter_jet_instance.as_mut() {
                                sj_mut.x = fighter_jet_world_x;
                                sj_mut.y = fighter_jet_world_y;
                            }
                        }
                    } else {
                        if fighter_jet_instance.is_some() {
                            println!("Player not in fighter_jet's current sbrx field ({:?}). Despawning fighter_jet visual.", fighter_jet_current_sbrx_location);
                            fighter_jet_instance = None;
                        }
                    }
                    show_fighter_jet_prompt = false;
                    if let Some(ref sj) = fighter_jet_instance {
                        if sbrx_map_system.current_field_id == fighter_jet_current_sbrx_location {
                            let dx = fighter.x - sj.x;
                            let dy = fighter.y - sj.y;
                            let distance_to_fighter_jet = (dx * dx + dy * dy).sqrt();
                            if distance_to_fighter_jet <= FIGHTER_JET_INTERACTION_DISTANCE {
                                let find_survivors_task_active =
                                    task_system.has_task("FIND 10 SURVIVORS");
                                let find_survivors_task_complete =
                                    task_system.is_task_complete("FIND 10 SURVIVORS");

                                if (!find_survivors_task_active || find_survivors_task_complete)
                                    && !racetrack_active
                                {
                                    show_fighter_jet_prompt = true;
                                }
                            }
                        }
                    }
                    show_raptor_nest_prompt = false;
                    if current_area.is_none()
                        && sbrx_map_system.current_field_id == SbrxFieldId(1, 0)
                    {
                        if let Some(nest) = raptor_nests.first() {
                            let dx = fighter.x - nest.x;
                            let dy = fighter.y - nest.y;
                            let distance_sq = dx * dx + dy * dy;
                            if distance_sq < RAPTOR_NEST_INTERACTION_DISTANCE.powi(2) {
                                show_raptor_nest_prompt = true;
                            }
                        }
                    }

                    show_bunker_prompt = false;
                    if current_area.is_none()
                        && sbrx_map_system.current_field_id == SbrxFieldId(-25, 25)
                    {
                        if let Some(&(bunker_x, bunker_y)) = fort_silo_bunkers.first() {
                            let dx = fighter.x - bunker_x;
                            let dy = fighter.y - bunker_y;
                            let distance_sq = dx * dx + dy * dy;
                            if distance_sq < RAPTOR_NEST_INTERACTION_DISTANCE.powi(2) {
                                show_bunker_prompt = true;
                            }
                        }
                    }

                    // Check for raptor nest exit prompt
                    show_raptor_nest_exit_prompt = false;
                    show_bunker_exit_prompt = false;
                    show_bunker_floor_transition_prompt = false;
                    target_floor_from_prompt = None;
                    if let Some(area_state) = &current_area {
                        if let Some(target_floor) =
                            area_state.get_player_floor_transition(fighter.x, fighter.y)
                        {
                            if area_state.area_type == AreaType::Bunker {
                                show_bunker_floor_transition_prompt = true;
                                target_floor_from_prompt = Some(target_floor);
                            }
                        }
                        if area_state.is_player_at_world_exit(fighter.x, fighter.y) {
                            match area_state.area_type {
                                AreaType::RaptorNest => show_raptor_nest_exit_prompt = true,
                                AreaType::Bunker => show_bunker_exit_prompt = true,
                            }
                        }
                    }

                    let field_id_has_changed_for_spawn_check = last_field_id_for_rattlesnake_spawn
                        != Some(sbrx_map_system.current_field_id);
                    if field_id_has_changed_for_spawn_check {
                        // Do not spawn default rattlesnakes in Rocketbay
                        if sbrx_map_system.current_field_id != SbrxFieldId(0, 0)
                            && sbrx_map_system.current_field_id != SbrxFieldId(-2, 5)
                            && sbrx_map_system.current_field_id != SbrxFieldId(-25, 25)							
                            && !wave_manager.is_active()
                        {
                            println!("sbrx Field changed to non-(0,0) field {:?}. Spawning 3 rattlesnakes.", sbrx_map_system.current_field_id);
                            for _ in 0..3 {
                                if CPU_ENABLED {
                                    if cpu_entities.len() < 10 {
                                        cpu_entities.push(CpuEntity::new_rattlesnake(line_y));
                                    }
                                }
                            }
                        }
                    }
                    last_field_id_for_rattlesnake_spawn = Some(sbrx_map_system.current_field_id);
                    if !rattlesnakes_spawned_in_field0_score3
                        && sbrx_map_system.current_field_id == SbrxFieldId(0, 0)
                        && fighter.score >= 3
                        && !wave_manager.is_active()
                    {
                        if CPU_ENABLED && !wave_manager.is_active() && !racetrack_active {
                            println!("Spawning 3 rattlesnakes due to score 3 in sbrx field (0,0).");
                            for _ in 0..3 {
                                if cpu_entities.len() < 10 {
                                    cpu_entities.push(CpuEntity::new_rattlesnake(line_y));
                                }
                            }
                            rattlesnakes_spawned_in_field0_score3 = true;
                        }
                    }
                    if CPU_ENABLED && !racetrack_active {
                        let current_score = fighter.score;
                        let mut score_tier_to_check = 5;
                        while score_tier_to_check <= current_score {
                            if !spawned_giant_rattlesnake_scores.contains(&score_tier_to_check) {
                                // Do not spawn score-based Giant Rattlesnakes in certain areas.
                                if current_area.is_none()
                                    && sbrx_map_system.current_field_id != SbrxFieldId(-2, 5)
									&& sbrx_map_system.current_field_id != SbrxFieldId(-25, 25)
                                {
                                    println!(
                                        "Spawning Giant Rattlesnake for score tier {}.",
                                        score_tier_to_check
                                    );
                                    if cpu_entities.len() < 10 {
                                        cpu_entities.push(CpuEntity::new_giant_rattlesnake(line_y));
                                    }
                                }
                                spawned_giant_rattlesnake_scores.insert(score_tier_to_check);
                            }
                            score_tier_to_check += 5;
                        }
                        let next_blood_idol_spawn_score = 0;
						/*
                        if current_score >= 0 {
                            if next_blood_idol_spawn_score == 0 {
                                next_blood_idol_spawn_score = 0;
                            }
                            while next_blood_idol_spawn_score <= current_score
                                && spawned_blood_idol_scores.contains(&next_blood_idol_spawn_score)
                            {
                            }
                        }
						*/
                        if next_blood_idol_spawn_score > 0
                            && next_blood_idol_spawn_score <= current_score
                        {
                            let blood_idol_active = cpu_entities
                                .iter()
                                .any(|e| e.variant == CpuVariant::BloodIdol && !e.is_dead());
                            let mantis_exists_alive = cpu_entities
                                .iter()
                                .any(|e| e.variant == CpuVariant::GiantMantis && !e.is_dead());
                            if !blood_idol_active && mantis_exists_alive {
                                println!(
                                    "BLOOD IDOL spawning for score {}.",
                                    next_blood_idol_spawn_score
                                );
                                let (base_hp, base_speed) = cpu_entities
                                    .iter()
                                    .find(|e| e.variant == CpuVariant::GiantMantis && !e.is_dead())
                                    .map_or((250.0, 150.0), |m| (m.max_hp, m.speed));
                                let spirit = CpuEntity::new_blood_idol(line_y, base_hp, base_speed);
                                if cpu_entities.len() < 10 {
                                    cpu_entities.push(spirit);
                                }
                                spawned_blood_idol_scores.insert(next_blood_idol_spawn_score);
                            }
                        }
                    }
                    if CPU_ENABLED
                        && matches!(game_state, GameState::Playing)
                        && (!racetrack_active || endless_arena_mode_active)
                    {
                        let mut player_died_this_frame = false;

                        for cpu_entity in &mut cpu_entities {
                            if !player_died_this_frame
                                && cpu_entity.check_collision(
                                    fighter.x,
                                    fighter.y,
                                    block_system.active,
                                    &audio_manager,
                                )
                            {
                                let mut attack_negated = block_system.process_attack(
                                    &mut fighter,
                                    cpu_entity,
                                    &audio_manager,
                                    game_time,
                                );

                                // --- Passive Defense Check (Auto-Block / Auto-Dodge) ---
                                // Condition: Attack wasn't manually blocked, player isn't invincible, and NOT vulnerable/broken
                                if !attack_negated && fighter.invincible_timer <= 0.0 && !block_system.block_broken {
                                    let mut rng = rand::rng();
                                    let roll: f64 = rng.random();
                                    
                                    // Visual Priority: Only show passive anims if player isn't busy with inputs
                                    let can_show_passive_anim = !is_high_priority_animation_active(
                                        rush_active, strike_animation_timer, block_system.active, block_system.rmb_held
                                    ) && !movement_active && !backpedal_active;

                                    if roll < fighter.stats.defense.auto_dodge {
                                        attack_negated = true;
                                        audio_manager.play_sound_effect("boost").ok();
                                        damage_texts.push(DamageText {
                                            text: "DODGE".to_string(),
                                            x: fighter.x, y: fighter.y - 90.0,
                                            color: [0.0, 0.8, 1.0, 1.0], lifetime: 0.5,
                                        });
                                        if can_show_passive_anim {
                                            current_racer_texture = current_backpedal_texture;
                                            strike_animation_timer = 0.25
                                        }
                                    } else if roll < fighter.stats.defense.auto_dodge + fighter.stats.defense.auto_block {
                                        attack_negated = true;
                                        audio_manager.play_sound_effect("block").ok();
                                        damage_texts.push(DamageText {
                                            text: "BLOCK".to_string(),
                                            x: fighter.x, y: fighter.y - 90.0,
                                            color: [1.0, 1.0, 1.0, 1.0], lifetime: 0.5,
                                        });
                                        if can_show_passive_anim {
                                            current_racer_texture = current_block_texture;
                                            strike_animation_timer = 0.25
                                        }
                                    }
                                }

                                if !attack_negated {
                                    if cpu_entity.damage_display_cooldown <= 0.0 {
                                        if fighter.invincible_timer <= 0.0 {
                                            let damage_chunk = cpu_entity.damage_value;
                                            let combo_dr_multiplier =
                                                combo_system.get_damage_intake_multiplier();
                                            let final_damage = damage_chunk
                                                * block_system.get_damage_multiplier()
                                                * combo_dr_multiplier;
                                            fighter.current_hp -= final_damage;
                                            damage_texts.push(DamageText {
                                                text: format!("{:.0}", damage_chunk),
                                                x: fighter.x,
                                                y: fighter.y - 70.0,
                                                color: [1.0, 1.0, 1.0, 1.0],
                                                lifetime: 0.25,
                                            });
                                            cpu_entity.damage_display_cooldown = 0.125;

                                            // Dismount if hit on bike
                                            if fighter.state == RacerState::OnBike {
                                                println!("Dismounted by enemy damage!");
                                                fighter.state = RacerState::OnFoot;

                                                if fighter.fighter_type != FighterType::Raptor {
                                                    sbrx_bike.respawn(fighter.x, fighter.y);
                                                }

                                                let tex_set = match fighter.fighter_type {
                                                    FighterType::Racer => &racer_textures,
                                                    FighterType::Soldier => &soldier_textures,
                                                    FighterType::Raptor => &raptor_textures,
                                                };
                                                update_current_textures(
                                                    &fighter,
                                                    tex_set,
                                                    &mut current_idle_texture,
                                                    &mut current_fwd_texture,
                                                    &mut current_backpedal_texture,
                                                    &mut current_block_texture,
                                                    &mut current_block_break_texture,
                                                    &mut current_ranged_texture,
                                                    &mut current_ranged_marker_texture,
                                                    &mut current_ranged_blur_texture,
                                                    &mut current_rush_texture,
                                                    &mut current_strike_textures,
													false, // Force standard textures on dismount event
                                                );
                                                current_racer_texture = current_idle_texture;
                                                chatbox.add_interaction(vec![(
                                                    "DISMOUNTED",
                                                    MessageType::Warning,
                                                )]);
                                            }

                                            if fighter.current_hp <= 0.0 {
                                                player_died_this_frame = true;
                                                lmb_held = false; // Stop rapid fire on death
                                                melee_rapid_fire_timer = 0.0;
                                                soldier_rapid_fire_timer = 0.0;
												
                                                if let Some(sink) = bike_accelerate_sound_sink.take() { sink.stop(); }
                                                if let Some(sink) = bike_idle_sound_sink.take() { sink.stop(); }										
												
                                                let death_type_for_cpu = match cpu_entity.variant {
                                                    CpuVariant::GiantMantis => {
                                                        DeathType::GiantMantis
                                                    }
                                                    CpuVariant::BloodIdol => DeathType::BloodIdol,
                                                    CpuVariant::Rattlesnake => {
                                                        DeathType::Rattlesnake
                                                    }
                                                    CpuVariant::GiantRattlesnake => {
                                                        DeathType::GiantRattlesnake
                                                    }
                                                    CpuVariant::Raptor => DeathType::Raptor,
                                                    CpuVariant::TRex => DeathType::TRex,
                                                    CpuVariant::VoidTempest => {
                                                        DeathType::VoidTempest
                                                    }
                                                    CpuVariant::LightReaver => {
                                                        DeathType::LightReaver
                                                    }
                                                    CpuVariant::NightReaver => {
                                                        DeathType::NightReaver
                                                    }
                                                    CpuVariant::RazorFiend => DeathType::RazorFiend,
                                                };

                                                fighter_hp_map.insert(fighter.fighter_type, 0.0);

                                                if fighter.state == RacerState::OnBike {
                                                    fighter.state = RacerState::OnFoot;
                                                    if fighter.fighter_type != FighterType::Raptor {
                                                        sbrx_bike.respawn(fighter.x, fighter.y);
                                                    }
                                                }

                                                block_break_animation_active = false;
                                                block_system = BlockSystem::new(20);

                                                let mut group_members = vec![FighterType::Racer];
                                                if soldier_has_joined {
                                                    group_members.push(FighterType::Soldier);
                                                }
                                                if raptor_has_joined {
                                                    group_members.push(FighterType::Raptor);
                                                }

                                                let has_survivors =
                                                    group_members.iter().any(|ft| {
                                                        !downed_fighters.contains(ft)
                                                            && *ft != fighter.fighter_type
                                                    });

                                                if group_members.len() > 1 && has_survivors {
                                                    game_state = GameState::DeathScreenGroup {
                                                        death_type: death_type_for_cpu,
                                                        downed_fighter_type: fighter.fighter_type,
                                                    };
                                                    death_screen_cooldown =
                                                        DEATH_SCREEN_COOLDOWN_TIME;
                                                    if !downed_fighters
                                                        .contains(&fighter.fighter_type)
                                                    {
                                                        downed_fighters.push(fighter.fighter_type);
                                                    }
                                                    revival_kill_score = 0;
                                                } else {
                                                    if !downed_fighters
                                                        .contains(&fighter.fighter_type)
                                                    {
                                                        downed_fighters.push(fighter.fighter_type);
                                                    }
                                                    game_state =
                                                        GameState::DeathScreen(death_type_for_cpu);
                                                    death_screen_cooldown =
                                                        DEATH_SCREEN_COOLDOWN_TIME;
                                                }
                                                audio_manager.play_sound_effect("death").ok();
                                            }
                                        } else {
                                            cpu_entity.damage_display_cooldown = 0.125;
                                        }
                                    }
                                }
                            }
                        }

                        // --- Phase 2: Identify and process dead enemies ---
                        let mut dead_indices = Vec::new();
                        for (i, cpu) in cpu_entities.iter().enumerate() {
                            if cpu.is_dead() {
                                dead_indices.push(i);
                            }
                        }

                        if !dead_indices.is_empty() {
                            let mut raptor_died_in_area = false;
                            let mut t_rex_was_defeated_this_frame = false;
                            let mut cpus_to_remove = Vec::new();
                            let mut cpus_to_respawn = Vec::new();

                            for &index in &dead_indices {
                                if let Some(cpu_entity) = cpu_entities.get(index) {
                                    spawn_particles(
                                        &mut particles,
                                        cpu_entity.x,
                                        cpu_entity.y,
                                        PARTICLE_COUNT_CPU,
                                    );

                                    if wave_manager.is_active() {
                                        wave_manager.notify_enemy_defeated();
                                    }
                                    task_system.increment_kill_count(cpu_entity.variant);																
									
                                    let score_value = match cpu_entity.variant {
                                        CpuVariant::GiantMantis => 3,
                                        CpuVariant::Rattlesnake => 1,
                                        CpuVariant::GiantRattlesnake => 3,
                                        CpuVariant::Raptor => 2,
                                        CpuVariant::TRex => 5,
                                        CpuVariant::BloodIdol => 3,
                                        CpuVariant::VoidTempest => 4,
                                        CpuVariant::LightReaver => 2,
                                        CpuVariant::NightReaver => 2,
                                        CpuVariant::RazorFiend => 10,
                                    };
									
                                    if endless_arena_mode_active {
                                        arena_kill_count += score_value; 
                                        // Milestones for every 25 points
                                        let current_milestone = arena_kill_count / 25;
                                        if current_milestone > last_arena_milestone && current_milestone > 0 {
                                            last_arena_milestone = current_milestone;
                                                                                       
                                            let damage_bonus = 1.25;
                                            fighter.melee_damage = fighter.stats.attack.melee_damage + damage_bonus;
                                            fighter.ranged_damage = fighter.stats.attack.ranged_damage + damage_bonus;
											//fighter.run_speed *= 1.25;
                                            
                                            // Grant 3 seconds of invincibility
                                            fighter.invincible_timer = 5.0;
 
                                            chatbox.add_interaction(vec![
                                                (&format!("KILL BONUS: ATOMIC-STATE"), MessageType::Warning),
                                            ]);
                                            audio_manager.play_sound_effect("boost").ok();
                                        }										
                                    }									
                                    
                                    fighter.score = (fighter.score + score_value).min(999);

                                    let current_kills = fighter
                                        .kill_counters
                                        .entry(fighter.fighter_type)
                                        .or_insert(0);
                                    *current_kills += score_value;
                                    let current_level =
                                        fighter.levels.entry(fighter.fighter_type).or_insert(1);
                                    let stat_points = fighter
                                        .stat_points_to_spend
                                        .entry(fighter.fighter_type)
                                        .or_insert(0);
                                    let levels_gained = mechanics::lvl_up::check_for_level_up(
                                        current_kills,
                                        current_level,
                                    );
                                    if levels_gained > 0 {
                                        *stat_points += levels_gained;
                                        lvl_up_state = LvlUpState::PendingTab {
                                            fighter_type: fighter.fighter_type,
                                        };
                                        let fighter_name = match fighter.fighter_type {
                                            FighterType::Racer => "RACER",
                                            FighterType::Soldier => "SOLDIER",
                                            FighterType::Raptor => "RAPTOR",
                                        };
                                        chatbox.add_interaction(vec![(
                                            &format!(
                                                "!! [TAB] TO LVL UP [{}] +[{}] !!",
                                                fighter_name, *stat_points
                                            ),
                                            MessageType::Warning,
                                        )]);
										audio_manager.play_sound_effect("death").ok();
                                    }

                                    if !downed_fighters.is_empty() {
                                        revival_kill_score += 1;
                                    }
                                    fighter.current_hp =
                                        (fighter.current_hp + 25.0).min(fighter.max_hp);
                                    fighter.fuel = (fighter.fuel + FUEL_REPLENISH_AMOUNT)
                                        .min(fighter.max_fuel);

                                    match cpu_entity.variant {
                                        CpuVariant::Raptor => {
                                            cpus_to_remove.push(index);
                                            if current_area.is_some() {
                                                raptor_died_in_area = true;
                                            }
                                        }
                                        CpuVariant::TRex => {
                                            cpus_to_remove.push(index);
                                            t_rex_was_defeated_this_frame = true;
                                        }
                                        CpuVariant::BloodIdol
                                        | CpuVariant::VoidTempest
                                        | CpuVariant::LightReaver
                                        | CpuVariant::NightReaver
                                        | CpuVariant::RazorFiend => {
                                            cpus_to_remove.push(index);
                                            if cpu_entity.variant == CpuVariant::RazorFiend {
                                                razor_fiend_defeated_flag = true;
                                            }
                                        }
                                        _ => {
                                            if endless_arena_mode_active {
                                                cpus_to_remove.push(index);
                                            // In Rocketbay, remove default enemies instead of respawning them.
                                            // The continuous spawner will add Night Reavers.
                                            } else if sbrx_map_system.current_field_id
                                                == SbrxFieldId(-2, 5)
                                            {
                                                cpus_to_remove.push(index);
                                            } else {
                                                cpus_to_respawn.push(index);
                                            }
                                        }
                                    }
                                }
                            }

                            // --- Phase 3: Mutate the cpu_entities list safely ---
                            for &index in &cpus_to_respawn {
                                if let Some(cpu) = cpu_entities.get_mut(index) {
                                    cpu.respawn(line_y);
                                }
                            }

                            cpus_to_remove.sort_unstable();
                            for &index in cpus_to_remove.iter().rev() {
                                if index < cpu_entities.len() {
                                    cpu_entities.remove(index);
                                }
                            }

                            if raptor_died_in_area {
                                let live_raptors = cpu_entities
                                    .iter()
                                    .filter(|c| c.variant == CpuVariant::Raptor && !c.is_dead())
                                    .count();
                                if live_raptors == 0 {
                                    if !task_system.raptor_nest_cleared {
                                        task_system.mark_raptor_nest_cleared();
                                        println!("All raptors in nest cleared for the first time! raptor can be rescued.");
                                        raptor_is_trapped_in_nest = true;
                                    }
                                    println!(
                                        "Raptor Nest cleared. A T-Rex will be waiting outside."
                                    );
                                    t_rex_spawn_pending = true;
                                }
                            }

                            if t_rex_was_defeated_this_frame {
                                println!(
                                    "T-Rex defeated. Restoring default CPU spawn (Giant Mantis)."
                                );
                                t_rex_is_active = false;
                                if CPU_ENABLED && cpu_entities.len() < 10 {
                                    cpu_entities.push(CpuEntity::new_giant_mantis(line_y));
                                }
                            }
                        }
                    }
                    //  //	//	//	//
                    // NEW: Check for fighter revival
                    if revival_kill_score >= 10 {
                        if !downed_fighters.is_empty() {
                            let fighter_to_revive = downed_fighters.remove(0);
                            println!("Reviving fighter: {:?}", fighter_to_revive);

                            revival_kill_score = 0; // Reset for the next revival

                            let max_hp = match fighter_to_revive {
                                FighterType::Racer => RACER_LVL1_STATS.defense.hp,
                                FighterType::Soldier => SOLDIER_LVL1_STATS.defense.hp,
                                FighterType::Raptor => RAPTOR_LVL1_STATS.defense.hp,
                            };

                            // Revive with 25% health
                            let revived_hp = max_hp * 0.25;
                            fighter_hp_map.insert(fighter_to_revive, revived_hp);

                            // Post a message
                            chatbox.add_interaction(vec![(
                                &format!(
                                    "{} IS BACK IN THE FIGHT!",
                                    match fighter_to_revive {
                                        FighterType::Racer => "RACER",
                                        FighterType::Soldier => "SOLDIER",
                                        FighterType::Raptor => "RAPTOR",
                                    }
                                ),
                                MessageType::Notification,
                            )]);
                        }
                    }

                    if block_system.needs_dismount {
                        if fighter.state == RacerState::OnBike {
                            println!("Dismounting from block break!");
                            fighter.state = RacerState::OnFoot;
                            sbrx_bike.respawn(fighter.x, fighter.y);
                            let tex_set = match fighter.fighter_type {
                                FighterType::Racer => &racer_textures,
                                FighterType::Soldier => &soldier_textures,
                                FighterType::Raptor => &raptor_textures,
                            };
                            update_current_textures(
                                &fighter,
                                tex_set,
                                &mut current_idle_texture,
                                &mut current_fwd_texture,
                                &mut current_backpedal_texture,
                                &mut current_block_texture,
                                &mut current_block_break_texture,
                                &mut current_ranged_texture,
                                &mut current_ranged_marker_texture,
                                &mut current_ranged_blur_texture,
                                &mut current_rush_texture,
                                &mut current_strike_textures,
								shift_held,
                            );
                            current_racer_texture = if block_break_animation_active {
                                current_block_break_texture
                            } else {
                                current_idle_texture
                            };
                        }
                        block_system.needs_dismount = false;
                    }

                    // --- Special Field Spawning Logic ---
                    if !is_paused && CPU_ENABLED && current_area.is_none() {
                        if sbrx_map_system.current_field_id == SbrxFieldId(-2, 5) {
                            // ROCKETBAY: Continuously spawn Night Reavers if count is low.
                            let void_tempest_exists = cpu_entities
                                .iter()
                                .any(|e| e.variant == CpuVariant::VoidTempest);
                            let night_reaver_count = cpu_entities
                                .iter()
                                .filter(|e| e.variant == CpuVariant::NightReaver)
                                .count();

                            // Keep a population of up to 4 Night Reavers, but don't spawn if Void Tempest is active.
                            if !void_tempest_exists
                                && night_reaver_count < 4
                                && cpu_entities.len() < 10
                            {
                                night_reaver_spawn_timer += dt;
                                if night_reaver_spawn_timer >= next_night_reaver_spawn {
                                    println!("Spawning Night Reaver in Rocketbay.");
                                    let reaver_x = safe_gen_range(MIN_X, MAX_X, "NightReaver x");
                                    let reaver_y = safe_gen_range(MIN_Y, MAX_Y, "NightReaver y");
                                    cpu_entities
                                        .push(CpuEntity::new_night_reaver(reaver_x, reaver_y));
                                    night_reaver_spawn_timer = 0.0;
                                    next_night_reaver_spawn =
                                        safe_gen_range(3.0, 6.0, "Next Night Reaver spawn time");
                                }
                            }
                        }
                    }

                    camera.update(fighter.x, fighter.y);
                }
				
// --- JUMP ZONE CHECK (1 -> 2 -> 3) ---
				// This is moved OUTSIDE the invincibility check to allow Leg 2 to trigger immediately after Leg 1
				if !is_paused && current_area.is_none() {
					if let Some(hit) = collision_barrier_manager.check_jump(
						&sbrx_map_system.current_field_id,
						fighter.x,
						fighter.y,
					) {
						match hit.zone_type {
							JumpZoneType::Launch => {
								// LEG 1: RAPID LAUNCH ('1' -> '2')
								if fighter.knockback_duration <= 0.0 {
									let dx = hit.target_x - fighter.x;
									let dy = hit.target_y - fighter.y;
									let dist = (dx * dx + dy * dy).sqrt();
									if dist > 10.0 {
										let norm_dx = dx / dist;
										let norm_dy = dy / dist;
										let travel_time = 0.25; 
										let required_speed = dist / travel_time;
										fighter.knockback_velocity = Vec2d::new(norm_dx * required_speed, norm_dy * required_speed);
										fighter.knockback_duration = travel_time;
										fighter.invincible_timer = 2.0;
										in_jump_sequence = true;  // Start jump sequence
									}
								}
							}
							JumpZoneType::Air => {
								// LEG 2: FLUID MID-AIR REDIRECT ('2' -> '3')
								// Only triggers if player is in a jump sequence from '1'
								if in_jump_sequence {
									let dx = (hit.target_x + 50.0) - fighter.x;
									let dy = (hit.target_y + 50.0) - fighter.y;  // Add 50 units to land lower
									let dist = (dx * dx + dy * dy).sqrt();
									
									if dist > 50.0 {
										let norm_dx = dx / dist;
										let norm_dy = dy / dist;
										let travel_time = 0.25; 
										let required_speed = dist / travel_time;
										fighter.knockback_velocity = Vec2d::new(norm_dx * required_speed, norm_dy * required_speed);
										fighter.knockback_duration = travel_time;
										fighter.invincible_timer = 2.0; 
									}
								}
								// If not in jump sequence, do nothing (player walks/rides over)
							}
							JumpZoneType::Landing => {
								// TOUCHDOWN: Reset physics and end high-immunity
								if in_jump_sequence {
									fighter.knockback_velocity = Vec2d::new(0.0, 0.0);
									fighter.knockback_duration = 0.0;
									if fighter.invincible_timer > 0.3 {
										fighter.invincible_timer = 0.3;
									}
									in_jump_sequence = false;  // End jump sequence
								}
							}
						}
					}
				}
				
				// --- CHAIN ZONE CHECK ('0' -> '0' -> '0' ...) ---
				if let Some(hit) = collision_barrier_manager.check_chain(
					&sbrx_map_system.current_field_id,
					fighter.x,
					fighter.y,
				) {
					if hit.is_final {
						// Final zone in chain - stop and end immunity
						fighter.knockback_velocity = Vec2d::new(0.0, 0.0);
						fighter.knockback_duration = 0.0;
						if fighter.invincible_timer > 0.3 {
							fighter.invincible_timer = 0.3;
						}
					} else {
						// Launch to next '0' in chain
						let dx = hit.target_x - fighter.x;
						let dy = hit.target_y - fighter.y;
						let dist = (dx * dx + dy * dy).sqrt();
						if dist > 0.0 {
							let norm_dx = dx / dist;
							let norm_dy = dy / dist;
							let travel_time = 0.4;
							let required_speed = dist / travel_time;
							fighter.knockback_velocity = Vec2d::new(norm_dx * required_speed, norm_dy * required_speed);
							fighter.knockback_duration = travel_time;
							fighter.invincible_timer = 1.5;
						}
					}
				}				
 
 				// --- GROUND ASSET COLLISIONS ---
 				if !is_paused && fighter.invincible_timer <= 0.0 && current_area.is_none() {
					// --- COLLISION BARRIER CHECK ---
 					if let Some((barrier_x, barrier_y)) = collision_barrier_manager.check_collision(
 						&sbrx_map_system.current_field_id,
 						fighter.x,
 						fighter.y,
 						45.0, // collision threshold
 					) {
 						audio_manager.play_sound_effect("death").ok();
 						let dx = fighter.x - barrier_x;
 						let dy = fighter.y - barrier_y;
 						let angle = dy.atan2(dx);
 						
 						if fighter.state == RacerState::OnFoot {
 							// [TRIP] - on foot collision with barrier
 							fighter.current_hp -= 25.0;
 							let force = 400.0;
 							fighter.knockback_velocity = Vec2d::new(angle.cos() * force, angle.sin() * force);
 							fighter.knockback_duration = 0.2;
 						} else {
 							// [CRASH] - vehicle collision with barrier
 							fighter.current_hp -= 100.0;
 							fighter.stun_timer = 1.0;
 							let force = 800.0;
 							fighter.knockback_velocity = Vec2d::new(angle.cos() * force, angle.sin() * force);
 							fighter.knockback_duration = 0.4;
							
							// BUG FIX: Deactivate rapid-fire melee state on crash
							lmb_held = false;
							melee_rapid_fire_timer = 0.0;							
 						
 							// Dismount
 							fighter.state = RacerState::OnFoot;
 							sbrx_bike.is_crashed = true;
 
 							if fighter.fighter_type != FighterType::Raptor {
 								sbrx_bike.visible = true;
 								sbrx_bike.x = fighter.x;
 								sbrx_bike.y = fighter.y;
 								let bike_angle = angle + 0.5;
 								sbrx_bike.knockback_velocity = Vec2d::new(bike_angle.cos() * 600.0, bike_angle.sin() * 600.0);
 								sbrx_bike.knockback_duration = 0.5;
 							}
 
 							let tex_set = match fighter.fighter_type {
 								FighterType::Racer => &racer_textures,
 								FighterType::Soldier => &soldier_textures,
 								FighterType::Raptor => &raptor_textures,
 							};
 							update_current_textures(&fighter, tex_set, &mut current_idle_texture, &mut current_fwd_texture, &mut current_backpedal_texture, &mut current_block_texture, &mut current_block_break_texture, &mut current_ranged_texture, &mut current_ranged_marker_texture, &mut current_ranged_blur_texture, &mut current_rush_texture, &mut current_strike_textures, shift_held);
 							current_racer_texture = current_block_break_texture;
 							chatbox.add_interaction(vec![("COLLISION", MessageType::Warning)]);
 						}
 
 						if fighter.current_hp <= 0.0 {
 							fighter_hp_map.insert(fighter.fighter_type, 0.0);
 							if let Some(sink) = bike_accelerate_sound_sink.take() { sink.stop(); }
 							if let Some(sink) = bike_idle_sound_sink.take() { sink.stop(); }
 							let mut group_members = vec![FighterType::Racer];
 							if soldier_has_joined { group_members.push(FighterType::Soldier); }
 							if raptor_has_joined { group_members.push(FighterType::Raptor); }
 							let has_survivors = group_members.iter().any(|ft| !downed_fighters.contains(ft) && *ft != fighter.fighter_type);
 
 							if group_members.len() > 1 && has_survivors {
 								game_state = GameState::DeathScreenGroup { death_type: DeathType::Crashed, downed_fighter_type: fighter.fighter_type };
 							} else {
 								game_state = GameState::DeathScreen(DeathType::Crashed);
 							}
 							if !downed_fighters.contains(&fighter.fighter_type) { downed_fighters.push(fighter.fighter_type); }
 							death_screen_cooldown = DEATH_SCREEN_COOLDOWN_TIME;
 						}
 						fighter.invincible_timer = 1.0;
 					}
					
 					// --- RUT ZONE CHECK (speed reduction applied in movement code) ---
 					let _in_rut_zone = collision_barrier_manager.check_rut(
 						&sbrx_map_system.current_field_id,
 						fighter.x,
 						fighter.y,
 					);					
					
					if let Some(assets) = placed_ground_assets.get(&sbrx_map_system.current_field_id) {
						for asset in assets {
							let dx = fighter.x - asset.x;
							let dy = fighter.y - asset.y;
							let dist_sq = dx * dx + dy * dy;
							let collision_radius = 45.0; // Standard hitbox for assets

							if dist_sq < collision_radius * collision_radius {
								audio_manager.play_sound_effect("death").ok();
								let angle = dy.atan2(dx);
								
								if fighter.state == RacerState::OnFoot {
									// [TRIP]
									fighter.current_hp -= 25.0;
									let force = 400.0;
									fighter.knockback_velocity = Vec2d::new(angle.cos() * force, angle.sin() * force);
									fighter.knockback_duration = 0.2;
								} else {
									// [CRASH]
									fighter.current_hp -= 100.0;
									fighter.stun_timer = 1.0;
									let force = 800.0;
									fighter.knockback_velocity = Vec2d::new(angle.cos() * force, angle.sin() * force);
									fighter.knockback_duration = 0.4;
									
									// BUG FIX: Deactivate rapid-fire melee state on crash
									lmb_held = false;
									melee_rapid_fire_timer = 0.0;										
								
									// Dismount
									fighter.state = RacerState::OnFoot;
									sbrx_bike.is_crashed = true; // Swaps texture to crashed for all synced views

									// Relocate and knockback vehicle ONLY for RACER or SOLDIER
									if fighter.fighter_type != FighterType::Raptor {
										sbrx_bike.visible = true;
										sbrx_bike.x = fighter.x;
										sbrx_bike.y = fighter.y;

										let bike_angle = angle + 0.5; // different trajectory from player
										sbrx_bike.knockback_velocity = Vec2d::new(bike_angle.cos() * 600.0, bike_angle.sin() * 600.0);
										sbrx_bike.knockback_duration = 0.5;
									} 
									// If raptor, sbrx_bike stays at its last left location 
									// but is_crashed flag (set above) ensures it renders as crashed.

									let tex_set = match fighter.fighter_type {
										FighterType::Racer => &racer_textures,
										FighterType::Soldier => &soldier_textures,
										FighterType::Raptor => &raptor_textures,
									};
									update_current_textures(&fighter, tex_set, &mut current_idle_texture, &mut current_fwd_texture, &mut current_backpedal_texture, &mut current_block_texture, &mut current_block_break_texture, &mut current_ranged_texture, &mut current_ranged_marker_texture, &mut current_ranged_blur_texture, &mut current_rush_texture, &mut current_strike_textures, shift_held);
                                    current_racer_texture = current_block_break_texture; // Display crash animation immediately
								}

								if fighter.current_hp <= 0.0 {
									fighter_hp_map.insert(fighter.fighter_type, 0.0);
								    if let Some(sink) = bike_accelerate_sound_sink.take() { sink.stop(); }
								    if let Some(sink) = bike_idle_sound_sink.take() { sink.stop(); }									
									let mut group_members = vec![FighterType::Racer];
									if soldier_has_joined { group_members.push(FighterType::Soldier); }
									if raptor_has_joined { group_members.push(FighterType::Raptor); }
									let has_survivors = group_members.iter().any(|ft| !downed_fighters.contains(ft) && *ft != fighter.fighter_type);

									if group_members.len() > 1 && has_survivors {
										game_state = GameState::DeathScreenGroup { death_type: DeathType::Crashed, downed_fighter_type: fighter.fighter_type };
									} else {
										game_state = GameState::DeathScreen(DeathType::Crashed);
									}
									if !downed_fighters.contains(&fighter.fighter_type) { downed_fighters.push(fighter.fighter_type); }
									death_screen_cooldown = DEATH_SCREEN_COOLDOWN_TIME;
								}
								fighter.invincible_timer = 1.0;
								break;
							}
						}
					}
				}				

                if let Some(_) = e.render_args() {
                    window.draw_2d(&e, |c, g, device| {
                        let tc = camera.transform(c);
                        let oc = c;
						
						// Render track notification (bottom right)
						if let Some(ref notif) = track_notification {
							let text = format!("track: {}", notif.track_name);
							let font_size = 18;
							let text_width = glyphs.width(font_size, &text).unwrap_or(150.0);
							let padding = 8.0;
							let box_width = text_width + padding * 2.0;
							let box_height = font_size as f64 + padding * 2.0;
							let box_x = screen_width - box_width - 20.0;
							let box_y = screen_height - box_height - 20.0;
							
							// Dark gray background
							rectangle(
								[0.2, 0.2, 0.2, 0.8],
								[box_x, box_y, box_width, box_height],
								c.transform,
								g,
							);
							
							// Green text
							piston_window::text::Text::new_color([0.0, 1.0, 0.0, 1.0], font_size)
								.draw(
									&text,
									&mut glyphs,
									&c.draw_state,
									c.transform.trans(box_x + padding, box_y + padding + font_size as f64 - 4.0),
									g,
								)
								.ok();
						}						

                        if let Some(area_state) = &current_area {
                            let (origin_x, origin_y, width, height, ground_color) =
                                match area_state.area_type {
                                    AreaType::RaptorNest => (
                                        AREA_ORIGIN_X,
                                        AREA_ORIGIN_Y,
                                        AREA_WIDTH,
                                        AREA_HEIGHT, 
                                        [0.4, 0.25, 0.13, 1.0],
                                    ),
                                    AreaType::Bunker => {                                        
                                        (
                                            BUNKER_ORIGIN_X,
                                            BUNKER_ORIGIN_Y,
                                            BUNKER_WIDTH,
                                            BUNKER_HEIGHT,
                                            [0.40, 0.40, 0.40, 1.0], 
                                        )
                                    }
                                };
                            clear([0.0, 0.0, 0.0, 1.0], g);
                            rectangle(
                                ground_color,
                                [origin_x, origin_y, width, height],
                                tc.transform,
                                g,
                            );

                            if area_state.area_type == AreaType::Bunker && area_state.floor == -3 {
                                // Determine which texture to use
                                let gc_tex = if grand_commander_dialogue_triggered {
                                    &grand_commander_texture
                                } else {
                                    &grand_commander_down_texture
                                };

                                let tex_w = gc_tex.get_width() as f64;
                                let tex_h = gc_tex.get_height() as f64;
                                let gc_x = BUNKER_ORIGIN_X + BUNKER_WIDTH / 2.0;
                                let gc_y = BUNKER_ORIGIN_Y + BUNKER_HEIGHT / 2.0;

                                let img_x = gc_x - tex_w / 2.0;
                                let img_y = gc_y - tex_h / 2.0;
                                image(gc_tex, tc.transform.trans(img_x, img_y), g);

                                // --- TASK 2 FIX: Always show prompt after fiend is defeated ---
                                if razor_fiend_defeated_flag && !grand_commander_dialogue_triggered
                                {
                                    let indicator_text = "[!]";
                                    let font_size = 32;
                                    let text_color = [1.0, 0.0, 0.0, 1.0];
                                    let text_width =
                                        glyphs.width(font_size, indicator_text).unwrap_or(0.0);
                                    let text_x = gc_x - text_width / 2.0;
                                    let text_y = img_y - 5.0; // Above the image
                                    text::Text::new_color(text_color, font_size)
                                        .draw(
                                            indicator_text,
                                            &mut glyphs,
                                            &tc.draw_state,
                                            tc.transform.trans(text_x, text_y),
                                            g,
                                        )
                                        .ok();
                                }
                            }

                            if area_state.area_type == AreaType::RaptorNest {
                                let remains_width = remains_texture.get_width() as f64;
                                let remains_height = remains_texture.get_height() as f64;
                                let remains_x = origin_x + (width - remains_width) / 2.0;
                                let remains_y = origin_y + (height - remains_height) / 2.0;
                                image(
                                    &remains_texture,
                                    tc.transform.trans(remains_x, remains_y),
                                    g,
                                );
                            }

                            // Only show raptor graphic in raptor nest
                            if area_state.area_type == AreaType::RaptorNest {
                                if show_raptor_in_nest_graphic {
                                    let raptor_texture_width =
                                        raptor_block_break_nest_texture.get_width() as f64;
                                    let raptor_texture_height =
                                        raptor_block_break_nest_texture.get_height() as f64;
                                    let raptor_x =
                                        origin_x + (width - raptor_texture_width) / 2.0 + 50.0;
                                    let raptor_y =
                                        origin_y + (height - raptor_texture_height) / 2.0 - 30.0;
                                    image(
                                        &raptor_block_break_nest_texture,
                                        tc.transform.trans(raptor_x, raptor_y),
                                        g,
                                    );
                                    if raptor_is_trapped_in_nest {
                                        let indicator_text = "[!]";
                                        let font_size = 32;
                                        let text_color = [1.0, 0.0, 0.0, 1.0];
                                        let text_width =
                                            glyphs.width(font_size, indicator_text).unwrap_or(0.0);
                                        let text_x = raptor_x + (raptor_texture_width / 2.0)
                                            - (text_width / 2.0);
                                        let text_y = raptor_y - 5.0;
                                        text::Text::new_color(text_color, font_size)
                                            .draw(
                                                indicator_text,
                                                &mut glyphs,
                                                &tc.draw_state,
                                                tc.transform.trans(text_x, text_y),
                                                g,
                                            )
                                            .ok();
                                    }
                                }
                            }
                            let exit = &area_state.exit_to_world;
                            rectangle(
                                [0.0, 1.0, 0.0, 1.0],
                                [exit.x, exit.y, exit.width, exit.height],
                                tc.transform,
                                g,
                            );

                            // Draw text for world exit
                            let exit_text = if area_state.area_type == AreaType::Bunker {
                                "TO FORT SILO"
                            } else {
                                "EXIT"
                            };
                            let exit_font_size = 20;
                            let exit_text_color = [0.0, 1.0, 0.0, 1.0];
                            let exit_text_width =
                                glyphs.width(exit_font_size, exit_text).unwrap_or(100.0);
                            let text_x = exit.x + (exit.width / 2.0) - (exit_text_width / 2.0);
                            let text_y = exit.y - 10.0; // Above the marker
                            text::Text::new_color(exit_text_color, exit_font_size)
                                .draw(
                                    exit_text,
                                    &mut glyphs,
                                    &tc.draw_state,
                                    tc.transform.trans(text_x, text_y),
                                    g,
                                )
                                .ok();

                            // Draw floor transitions
                            for transition in &area_state.floor_transitions {
                                let trans_rect = &transition.rect;
                                rectangle(
                                    [1.0, 0.5, 0.0, 1.0], // Orange
                                    [
                                        trans_rect.x,
                                        trans_rect.y,
                                        trans_rect.width,
                                        trans_rect.height,
                                    ],
                                    tc.transform,
                                    g,
                                );
                                let text_to_draw =
                                    format!("TO BUNKER[{}]", transition.target_floor);
                                let font_size = 20;
                                let text_color = [1.0, 0.5, 0.0, 1.0];

                                let text_width =
                                    glyphs.width(font_size, &text_to_draw).unwrap_or(100.0);
                                let (text_x, text_y) = if trans_rect.x
                                    > BUNKER_ORIGIN_X + BUNKER_WIDTH - 20.0
                                {
                                    // Right edge
                                    (
                                        trans_rect.x - text_width - 10.0,
                                        trans_rect.y + font_size as f64,
                                    )
                                } else if trans_rect.y < BUNKER_ORIGIN_Y + 20.0 {
                                    // Top edge
                                    (
                                        trans_rect.x,
                                        trans_rect.y + trans_rect.height + font_size as f64 + 5.0,
                                    )
                                } else {
                                    // Default
                                    (trans_rect.x, trans_rect.y - 10.0)
                                };

                                text::Text::new_color(text_color, font_size)
                                    .draw(
                                        &text_to_draw,
                                        &mut glyphs,
                                        &tc.draw_state,
                                        tc.transform.trans(text_x, text_y),
                                        g,
                                    )
                                    .ok();
                            }
                        } else {
                            let (current_ground_color, current_sky_color) =
                                sbrx_map_system.get_current_field_colors();
                            clear(current_ground_color, g);

                            // --- NEW: Render Ground Texture ---
                            if !excluded_ground_texture_fields
                                .contains(&sbrx_map_system.current_field_id)
                            {
                                let texture_index = *field_ground_texture_indices
                                    .entry(sbrx_map_system.current_field_id)
                                    .or_insert_with(|| rand::rng().random_range(0..4));
                                let ground_texture = &ground_textures[texture_index];

                                let tex_width = ground_texture.get_width() as f64;
                                let tex_height = ground_texture.get_height() as f64;

                                if tex_width > 0.0 && tex_height > 0.0 {
                                    // Calculate the visible world coordinates to ensure the entire screen is covered
                                    let (world_tl_x, world_tl_y) =
                                        screen_to_world(&camera, 0.0, 0.0);
                                    let (world_br_x, world_br_y) =
                                        screen_to_world(&camera, WIDTH, HEIGHT);

                                    // Align the starting coordinates to the texture grid to prevent texture "swimming"
                                    let start_x = (world_tl_x / tex_width).floor() * tex_width;
                                    let start_y = (world_tl_y / tex_height).floor() * tex_height;

                                    // Tile the texture to cover the entire visible area.
                                    // The sky rectangle is drawn over the top part, creating the horizon.
                                    let mut y = start_y;
                                    while y < world_br_y {
                                        let mut x = start_x;
                                        while x < world_br_x {
                                            image(ground_texture, tc.transform.trans(x, y), g);
                                            x += tex_width;
                                        }
                                        y += tex_height;
                                    }
                                }
                            }

                            piston_window::rectangle(
                                current_sky_color,
                                [-5000.0, -5000.0, sky_width + 10000.0, line_y + 5000.0],
                                tc.transform,
                                g,
                            );
                            stars.iter().for_each(|s| s.draw(tc, g));

                            // --- NEW: Render Placed Ground Assets ---
                            if current_area.is_none() {
                                if let Some(assets_to_render) =
                                    placed_ground_assets.get(&sbrx_map_system.current_field_id)
                                {
                                for asset in assets_to_render {
                                    if let Some(texture) =
                                        ground_asset_textures.get(&asset.texture_name)
                                    {
                                        let tex_w = texture.get_width() as f64;
                                        let tex_h = texture.get_height() as f64;
                                        // Anchor the image to its bottom center for proper placement
                                        let img_x = asset.x - tex_w / 2.0;
                                        let img_y = asset.y - tex_h;

                                        // Only render if it should be visible
                                        if !FOG_OF_WAR_ENABLED
                                            || fog_of_war.should_render_entity(
                                                sbrx_map_system.current_field_id,
                                                asset.x,
                                                asset.y,
                                            )
                                        {
                                            image(texture, tc.transform.trans(img_x, img_y), g);
                                        }
                                    }
                                }
								}
                            }
							
							

                            if sbrx_map_system.current_field_id == SbrxFieldId(0, 0) {
                                fuel_pump.fuel_interaction(&mut fighter, &fuel_pump_texture);
                            }

                            if sbrx_map_system.current_field_id == SbrxFieldId(0, 0) {
                                image(&track_texture, tc.transform.trans(track.x, track.y), g);
								
                                // TOGGLEABLE DEBUG: Draw collision barriers
                                if show_collision_debug > 0 {
                                    if let Some(barriers) = collision_barrier_manager.get_barriers(&sbrx_map_system.current_field_id) {
                                        for line in &barriers.lines {
                                            piston_window::line(
                                                [1.0, 0.0, 0.0, 0.5], // Semi-transparent red
                                                2.0,
                                                [line.x1, line.y1, line.x2, line.y2],
                                                tc.transform,
                                                g,
                                            );
                                        }

                                        if show_collision_debug == 2 {
                                            // Draw jump zones by type
                                            for zone in &barriers.jump_zones {
                                                let color = match zone.zone_type {
                                                    JumpZoneType::Launch => [0.0, 1.0, 0.0, 0.3],  // Green
                                                    JumpZoneType::Air => [0.0, 0.0, 0.0, 0.0],     // transparent
                                                    JumpZoneType::Landing => [0.0, 0.0, 0.0, 0.0], // transparent
                                                };
                                                rectangle(
                                                    color,
                                                    [zone.x, zone.y, zone.width, zone.height],
                                                    tc.transform,
                                                    g,
                                                );
                                            }
                                            // Draw chain zones (cyan) - shade indicates order
                                            let chain_count = barriers.chain_zones.len();
                                            for chain in &barriers.chain_zones {
                                               let alpha = if chain.is_final {
                                                   0.0  // Brightest for final
                                               } else if chain_count > 1 {
                                                   // Gradient: first is dimmest, increases toward final
                                                   0.3 + (chain.chain_index as f32 / chain_count as f32) * 0.4
                                               } else {
                                                   0.3
                                               };
                                               let color = [0.0, 1.0, 0.0, alpha];
                                               rectangle(
                                                   color,
                                                   [chain.x, chain.y, chain.width, chain.height],
                                                   tc.transform,
                                                   g,
                                               );
                                            }								
                                            // Draw rut zones (orange)
                                            for rut in &barriers.rut_zones {
                                                rectangle(
                                                    [1.0, 0.5, 0.0, 0.3], // Semi-transparent orange
                                                    [rut.x, rut.y, rut.width, rut.height],
                                                    tc.transform,
                                                    g,
                                                );
                                            }
                                        }
                                    }
                                }	

                                fuel_pump.draw(tc, g, &fuel_pump_texture);

                                // TEMPORARY RACE SETUP ASSETS AT START
                                /*
                                                                    // --- Render Race Setup Assets ---
                                                                    // RacerLineup.png - Positioned near the bottom-center of the world, like a starting line.
                                                                    let lineup_x = 150.0;
                                                                    let lineup_y = 3100.0;
                                                                    let lineup_w = racer_lineup_texture.get_width() as f64;
                                                                    let lineup_h = racer_lineup_texture.get_height() as f64;
                                                                    image(&racer_lineup_texture, tc.transform.trans(lineup_x - lineup_w / 2.0, lineup_y - lineup_h), g);

                                                                    // RaceSpectators.png - Positioned on the left side of the track.
                                                                    let spec1_x = 1.0;
                                                                    let spec1_y = 1500.0;
                                                                    let spec1_w = race_spectators_texture.get_width() as f64;
                                                                    let spec1_h = race_spectators_texture.get_height() as f64;
                                                                    image(&race_spectators_texture, tc.transform.trans(spec1_x - spec1_w / 2.0, spec1_y - spec1_h), g);

                                                                    // RaceSpectators2.png - Positioned on the right side of the track.
                                                                    let spec2_x = 5000.0;
                                                                    let spec2_y = 1500.0;
                                                                    let spec2_w = race_spectators2_texture.get_width() as f64;
                                                                    let spec2_h = race_spectators2_texture.get_height() as f64;
                                                                    image(&race_spectators2_texture, tc.transform.trans(spec2_x - spec2_w / 2.0, spec2_y - spec2_h), g);

                                                                    // --- Render Soldier and raptor at Racetrack ---
                                                                    // Position the Soldier
                                                                    let soldier_x = 150.0;
                                                                    let soldier_y = 650.0;
                                                                    let soldier_w = soldier_textures.idle.get_width() as f64;
                                                                    let soldier_h = soldier_textures.idle.get_height() as f64;
                                                                    image(&soldier_textures.idle, tc.transform.trans(soldier_x - soldier_w / 2.0, soldier_y - soldier_h), g);

                                                                    // Position the raptor
                                                                    let raptor_x = 50.0;
                                                                    let raptor_y = 700.0;
                                                                    let raptor_w = raptor_textures.block_break.get_width() as f64;
                                                                    let raptor_h = raptor_textures.block_break.get_height() as f64;
                                                                    image(&raptor_textures.block_break, tc.transform.trans(raptor_x - raptor_w / 2.0, raptor_y - raptor_h), g);

                                */
                                // TEMPORARY RACE SETUP ASSETS AT START

                                if task_system.is_task_complete("LAND THE FIGHTERJET ON THE RACETRACK")
                                {
                                    // --- Render Race Setup Assets ---
                                    // RacerLineup.png - Positioned near the bottom-center of the world, like a starting line.
                                    let lineup_x = 150.0;
                                    let lineup_y = 3100.0;
                                    let lineup_w = racer_lineup_texture.get_width() as f64;
                                    let lineup_h = racer_lineup_texture.get_height() as f64;
                                    image(
                                        &racer_lineup_texture,
                                        tc.transform
                                            .trans(lineup_x - lineup_w / 2.0, lineup_y - lineup_h),
                                        g,
                                    );

                                    // RaceSpectators.png - Positioned on the left side of the track.
                                    let spec1_x = 1.0;
                                    let spec1_y = 1500.0;
                                    let spec1_w = race_spectators_texture.get_width() as f64;
                                    let spec1_h = race_spectators_texture.get_height() as f64;
                                    image(
                                        &race_spectators_texture,
                                        tc.transform
                                            .trans(spec1_x - spec1_w / 2.0, spec1_y - spec1_h),
                                        g,
                                    );

                                    // RaceSpectators2.png - Positioned on the right side of the track.
                                    let spec2_x = 5000.0;
                                    let spec2_y = 1500.0;
                                    let spec2_w = race_spectators2_texture.get_width() as f64;
                                    let spec2_h = race_spectators2_texture.get_height() as f64;
                                    image(
                                        &race_spectators2_texture,
                                        tc.transform
                                            .trans(spec2_x - spec2_w / 2.0, spec2_y - spec2_h),
                                        g,
                                    );

                                    // --- Render Soldier and raptor at Racetrack ---
                                    if show_racetrack_soldier_raptor_assets {
                                        // Position the Soldier
                                        let soldier_x = 300.0;
                                        let soldier_y = 650.0;
                                        let soldier_w = soldier_textures.idle.get_width() as f64;
                                        let soldier_h = soldier_textures.idle.get_height() as f64;
                                        image(
                                            &soldier_textures.idle,
                                            tc.transform.trans(
                                                soldier_x - soldier_w / 2.0,
                                                soldier_y - soldier_h / 2.0,
                                            ),
                                            g,
                                        );
 
                                        // Position the raptor on racetrack
                                        let raptor_x = 275.0;
                                        let raptor_y = 700.0;
                                        let raptor_w = raptor_textures.idle.get_width() as f64;
                                        let raptor_h = raptor_textures.idle.get_height() as f64;
                                        image(
                                            &raptor_textures.idle,
                                            tc.transform.trans(
                                                raptor_x - raptor_w / 2.0,
                                                raptor_y - raptor_h / 2.0,
                                            ),
                                            g,
                                        );
 
                                        // Draw interaction prompt over soldier if conditions are met
                                        if show_racetrack_soldier_prompt {
                                            let indicator_text = "[!]";
                                            let font_size = 24;
                                            let text_color = [1.0, 0.0, 0.0, 1.0];
                                            let text_width =
                                                glyphs.width(font_size, indicator_text).unwrap_or(0.0);
                                            let text_x = soldier_x - text_width / 2.0;
                                            let text_y = soldier_y - (soldier_h / 2.0) - 5.0; // Position above the sprite
                                            text::Text::new_color(text_color, font_size)
                                                .draw(
                                                    indicator_text,
                                                    &mut glyphs,
                                                    &tc.draw_state,
                                                    tc.transform.trans(text_x, text_y),
                                                    g,
                                                )
                                                .ok();
                                        }
                                    }
                                }

                                // Hide Info Post and its prompt in Arena Mode
                                if !crate::config::ARENA_MODE {
                                    let info_post_w = info_post_texture.get_width() as f64;
                                    let info_post_h = info_post_texture.get_height() as f64;
                                    let img_x = info_post_position.0 - info_post_w / 2.0;
                                    let img_y = info_post_position.1 - info_post_h / 2.0;
                                    image(&info_post_texture, tc.transform.trans(img_x, img_y), g);

                                    if (!racetrack_active && !racetrack_info_post_interacted)
                                        || (racetrack_active && !finale_info_post_interacted)
                                    {
                                        let indicator_text = "[!]";
                                        let font_size = 32;
                                        let text_color = [1.0, 0.0, 0.0, 1.0];
                                        let text_width =
                                            glyphs.width(font_size, indicator_text).unwrap_or(0.0);
                                        let text_x = info_post_position.0 - text_width / 2.0;
                                        let text_y = img_y - 5.0;
                                        text::Text::new_color(text_color, font_size)
                                            .draw(
                                                indicator_text,
                                                &mut glyphs,
                                                &tc.draw_state,
                                                tc.transform.trans(text_x, text_y),
                                                g,
                                            )
                                            .ok();
                                     }
                                 }
                            }
                            if sbrx_map_system.current_field_id == SbrxFieldId(-25, 25) {
                                let fs_tex = if task_system.is_task_complete("SPEAK TO THE GRAND COMMANDER") {
                                    &fort_silo2_texture
                                } else {
                                    &fort_silo_texture
                                };
                                image(fs_tex, tc.transform.trans(track.x, track.y), g);

                                // Draw survivor if not interacted with yet
                                if !fort_silo_survivor.interaction_triggered {
                                    let survivor_texture = &soldier_textures.block_break;
                                    let tex_w = survivor_texture.get_width() as f64;
                                    let tex_h = survivor_texture.get_height() as f64;
                                    let img_x = fort_silo_survivor.x - tex_w / 2.0;
                                    let img_y = fort_silo_survivor.y - tex_h / 2.0;
                                    image(survivor_texture, tc.transform.trans(img_x, img_y), g);

                                    // Draw prompt
                                    let indicator_text = "[!]";
                                    let font_size = 24;
                                    let text_color = [1.0, 0.0, 0.0, 1.0];
                                    let text_width =
                                        glyphs.width(font_size, indicator_text).unwrap_or(0.0);
                                    let text_x = fort_silo_survivor.x - text_width / 2.0;
                                    let text_y = fort_silo_survivor.y - (tex_h / 2.0) - 5.0;
                                    text::Text::new_color(text_color, font_size)
                                        .draw(
                                            indicator_text,
                                            &mut glyphs,
                                            &tc.draw_state,
                                            tc.transform.trans(text_x, text_y),
                                            g,
                                        )
                                        .ok();
                                }

                                // Draw bunker entrance indicator
                                if let Some(&(bunker_x, bunker_y)) = fort_silo_bunkers.first() {
                                    // Draw a visual indicator for the bunker entrance
                                    let indicator_text = "[!]";
                                    let font_size = 32;
                                    let text_color = [1.0, 0.0, 0.0, 1.0];
                                    let text_width =
                                        glyphs.width(font_size, indicator_text).unwrap_or(0.0);
                                    let text_x = bunker_x - text_width / 2.0;
                                    let text_y = bunker_y - 20.0;
                                    text::Text::new_color(text_color, font_size)
                                        .draw(
                                            indicator_text,
                                            &mut glyphs,
                                            &tc.draw_state,
                                            tc.transform.trans(text_x, text_y),
                                            g,
                                        )
                                        .ok();
                                }
                            }
                            if !crate::config::ARENA_MODE && sbrx_map_system.current_field_id == SbrxFieldId(0, 0) {
                                if !FOG_OF_WAR_ENABLED
                                    || fog_of_war.should_render_entity(
                                        sbrx_map_system.current_field_id,
                                        random_image_x,
                                        random_image_y,
                                    )
                                {
                                    if soldier_visible {
                                        let tex_w = soldier_textures.block_break.get_width() as f64;
                                        let tex_h =
                                            soldier_textures.block_break.get_height() as f64;
                                        let img_x = random_image_x - tex_w / 2.0;
                                        let img_y = random_image_y - tex_h / 2.0;
                                        image(
                                            &soldier_textures.block_break,
                                            tc.transform.trans(img_x, img_y),
                                            g,
                                        );
                                        let indicator_text = "[!]";
                                        let font_size = 24;
                                        let text_color = [1.0, 0.0, 0.0, 1.0];
                                        let text_width =
                                            glyphs.width(font_size, indicator_text).unwrap_or(0.0);
                                        let text_x = random_image_x - text_width / 2.0;
                                        let text_y = random_image_y - (tex_h / 2.0) - 5.0;
                                        text::Text::new_color(text_color, font_size)
                                            .draw(
                                                indicator_text,
                                                &mut glyphs,
                                                &tc.draw_state,
                                                tc.transform.trans(text_x, text_y),
                                                g,
                                            )
                                            .ok();
                                    }
                                }
                            }
                            if sbrx_map_system.current_field_id == SbrxFieldId(-2, 5) {
                                image(&rocketbay_texture, tc.transform.trans(track.x, track.y), g);
                            }
							/*
                            line(
                                [0.0, 1.0, 0.0, 0.8], // green horizon_line color
                                0.1, // line thickness
                                [-1000.0, line_y, MAX_X + 1000.0, line_y],
                                tc.transform,
                                g,
                            );
							*/
                            spheres
                                .iter()
                                .filter(|s| s.has_exploded && s.crater_timer > 0.0 && s.y > line_y)
                                .for_each(|s| {
                                    draw_crater(
                                        s.x,
                                        s.y,
                                        s.crater_radius,
                                        [0.3, 0.3, 0.3, 1.0],
                                        tc,
                                        g,
                                    )
                                });
                            spheres
                                .iter()
                                .filter(|s| {
                                    !s.exploding && !s.has_exploded && s.y > line_y - s.size * 2.0
                                })
                                .for_each(|s| {
                                    let h_diff = s.crash_y - s.y;
                                    let o_f = (1.0 - h_diff / s.crash_y).max(0.0).min(1.0);
                                    let o = (0.6 * o_f).max(0.0);
                                    ellipse(
                                        [0.0, 0.0, 0.0, o as f32],
                                        [
                                            s.x - s.size * 0.6,
                                            s.crash_y - s.size * 0.6 * 0.5,
                                            s.size * 1.2,
                                            s.size * 1.2 * 0.5,
                                        ],
                                        tc.transform,
                                        g,
                                    );
                                });
                        }

                        // RENDER PARTICLES
                        for p in &particles {
                            let alpha_f64 = p.lifetime / p.max_lifetime;
                            let color_alpha = p.color[3] * alpha_f64 as f32;
                            let color = [p.color[0], p.color[1], p.color[2], color_alpha];
                            let size = p.size;
                            ellipse(
                                color,
                                [p.pos.x - size / 2.0, p.pos.y - size / 2.0, size, size],
                                tc.transform,
                                g,
                            );
                        }

                        if let Some(ref sj) = fighter_jet_instance {
                            // Don't render fighter_jet if player is in any area (bunker, raptor nest, etc.)
                            if current_area.is_none()
                                && sbrx_map_system.current_field_id == fighter_jet_current_sbrx_location
                                && (!FOG_OF_WAR_ENABLED
                                    || fog_of_war.should_render_entity(
                                        sbrx_map_system.current_field_id,
                                        sj.x,
                                        sj.y,
                                    ))
                            {
                                sj.draw(tc, g, &fighter_jet_texture);
                            }
                        }

                        if sbrx_map_system.current_field_id == SbrxFieldId(-2, 5) {
                            if let Some(field_survivors) =
                                survivors.get(&sbrx_map_system.current_field_id)
                            {
                                for survivor in field_survivors {
                                    if !survivor.is_rescued {
                                        if !FOG_OF_WAR_ENABLED
                                            || fog_of_war.is_position_visible(
                                                sbrx_map_system.current_field_id,
                                                survivor.x,
                                                survivor.y,
                                            )
                                        {
                                            let texture = match survivor.fighter_type {
                                                FighterType::Soldier => {
                                                    &soldier_textures.block_break
                                                }
                                                FighterType::Racer => &racer_textures.block_break,
                                                FighterType::Raptor => &raptor_textures.block_break,
                                            };
                                            let tex_w = texture.get_width() as f64;
                                            let tex_h = texture.get_height() as f64;
                                            let img_x = survivor.x - tex_w / 2.0;
                                            let img_y = survivor.y - tex_h / 2.0;
                                            image(texture, tc.transform.trans(img_x, img_y), g);
                                        }
                                    }
                                }
                            }
                        }

                        for site in &crashed_fighter_jet_sites {
                            if current_area.is_none()
                                && site.sbrx_field_id == sbrx_map_system.current_field_id
                                && (!FOG_OF_WAR_ENABLED
                                    || fog_of_war.should_render_entity(
                                        sbrx_map_system.current_field_id,
                                        site.world_x,
                                        site.world_y,
                                    ))
                            {
                                let tex_w = crashed_fighter_jet_texture.get_width() as f64;
                                let tex_h = crashed_fighter_jet_texture.get_height() as f64;
                                let img_x = site.world_x - tex_w / 2.0;
                                let img_y = site.world_y - tex_h / 2.0;
                                image(
                                    &crashed_fighter_jet_texture,
                                    tc.transform.trans(img_x, img_y),
                                    g,
                                );
                            }
                        }
                        if current_area.is_none() {
                            for nest in &raptor_nests {
                                if !FOG_OF_WAR_ENABLED
                                    || fog_of_war.should_render_entity(
                                        sbrx_map_system.current_field_id,
                                        nest.x,
                                        nest.y,
                                    )
                                {
                                    nest.draw(tc, g, &raptor_nest_texture);
                                }
                            }
                        }
                        for cpu_entity in &cpu_entities {
                            if !FOG_OF_WAR_ENABLED
                                || fog_of_war.should_render_entity(
                                    sbrx_map_system.current_field_id,
                                    cpu_entity.x,
                                    cpu_entity.y,
                                )
                            {
                                let textures_to_use = match cpu_entity.variant {
                                    CpuVariant::GiantMantis => &mantis_cpu_textures,
                                    CpuVariant::BloodIdol => &blood_idol_cpu_textures,
                                    CpuVariant::Rattlesnake => &rattlesnake_cpu_textures,
                                    CpuVariant::GiantRattlesnake => &giant_rattlesnake_cpu_textures,
                                    CpuVariant::Raptor => &raptor_cpu_textures,
                                    CpuVariant::TRex => &t_rex_cpu_textures,
                                    CpuVariant::VoidTempest => &void_tempest_cpu_textures,
                                    CpuVariant::LightReaver => &light_reaver_cpu_textures,
                                    CpuVariant::NightReaver => &night_reaver_cpu_textures,
                                    CpuVariant::RazorFiend => &razor_fiend_cpu_textures,
                                };
                                cpu_entity.draw(tc, g, textures_to_use);
                            }
                        }

                        for effect in &active_visual_effects {
                            let alpha = (effect.lifetime / effect.max_lifetime) as f32;
                            let img = Image::new_color([1.0, 1.0, 1.0, alpha]);

                            let tex_w = flicker_strike_effect_texture.get_width() as f64;
                            let tex_h = flicker_strike_effect_texture.get_height() as f64;

                            let img_x = effect.x - tex_w / 2.0;
                            let img_y = effect.y - tex_h / 2.0;

                            img.draw(
                                &flicker_strike_effect_texture,
                                &tc.draw_state,
                                tc.transform.trans(img_x, img_y),
                                g,
                            );
                        }

                        for orb in &pulse_orbs {
                            orb.draw(tc, g, &pulse_orb_texture);
                        }

                        for effect in &active_kinetic_strike_effects {
                            let alpha = (effect.lifetime / effect.max_lifetime) as f32;
                            let img = Image::new_color([1.0, 1.0, 1.0, alpha]);

                            let texture = &kinetic_strike_textures[effect.texture_index];
                            let tex_w = texture.get_width() as f64;
                            let tex_h = texture.get_height() as f64;

                            let img_x = effect.x - tex_w / 2.0;
                            let img_y = effect.y - tex_h / 2.0;

                            img.draw(texture, &tc.draw_state, tc.transform.trans(img_x, img_y), g);
                        }
						
                        // Render KINETIC_RUSH speed lines
                        for rush_line in &kinetic_rush_lines {
                            let alpha = (rush_line.lifetime / rush_line.max_lifetime) as f32;
                            let sx = rush_line.start_x;
                            let sy = rush_line.start_y;
                            let ex = rush_line.end_x;
                            let ey = rush_line.end_y;
                            let ldx = ex - sx;
                            let ldy = ey - sy;
                            let bls = (ldx * ldx + ldy * ldy).sqrt();
                            
                            if bls > 0.0 {
                                let mut rng = rand::rng();
                                let ndx = ldx / bls;
                                let ndy = ldy / bls;
								
/*								
(1.0, 3.0, [1.0, 1.0, 1.0, 0.9], 2.0),   // Main white line [KINETIC_RUSH SPEED LINES]
  │    │     │    │    │    │     │
  │    │     │    │    │    │     └── w (width): Line thickness in pixels (2.0 = thickest)
  │    │     │    │    │    │
  │    │     │    │    │    └── A (alpha): Opacity (0.9 = 90% visible)
  │    │     │    │    │
  │    │     │    │    └── B (blue): Blue color component (0.0-1.0)
  │    │     │    │
  │    │     │    └── G (green): Green color component (0.0-1.0)
  │    │     │
  │    │     └── R (red): Red color component (0.0-1.0)
  │    │
  │    └── ofr (offset range): Random offset in pixels (-3.0 to +3.0)
  │                            Smaller = lines stay closer together
  │                            Larger = lines spread out more (jittery effect)
  │
  └── lm (length multiplier): How far the line extends (1.0 = full distance)
                              0.7 would make the line 70% of the full length								
*/                                

                                // Multiple offset lines like SOLDIER ranged attack
                                for (lm, ofr, base_col, w) in [
                                    (1.0, 25.0, [1.0, 1.0, 1.0, 0.9], 2.0),   // Main white line
                                    (1.0, 50.0, [0.8, 0.8, 1.0, 0.7], 1.5),   // Secondary
                                    (1.0, 75.0, [0.7, 0.7, 0.9, 0.5], 1.0),   // Tertiary
                                ] {
                                    let ox = rng.random_range(-ofr..ofr);
                                    let oy = rng.random_range(-ofr..ofr);
                                    let efx = sx + ndx * bls * lm + ox;
                                    let efy = sy + ndy * bls * lm + oy;
                                    let col = [base_col[0], base_col[1], base_col[2], base_col[3] * alpha];
                                    line(col, w, [sx + ox, sy + oy, efx, efy], tc.transform, g);
                                }
                            }
                        }						

                        // CRATER STRIKE RADIUS: deactivated
                        /*
                                                if !is_special_strike_active {
                                                      draw_crater( //melee crater
                                                        fixed_crater.x,
                                                        fixed_crater.y,
                                                        fixed_crater.radius,
                                                        [1.0, 1.0, 1.0, 1.0], // melee strike radius color white
                                                        tc,
                                                        g,
                                                    );
                                                }
                        */
                        // NEW: Draw frontal strike visual effect
                        if frontal_strike_timer > 0.0 {
                            let mut color = frontal_strike_color;
                            // Fade out alpha
                            color[3] =
                                frontal_strike_color[3] * (frontal_strike_timer / 0.1) as f32;

                            let mut points = Vec::new();
                            points.push([fighter.x, fighter.y]); // Center of the arc is the player

                            let num_segments = 20; // Number of triangles to approximate the arc
                            let angle_start = frontal_strike_angle - std::f64::consts::FRAC_PI_2; // -90 degrees

                            for i in 0..=num_segments {
                                let current_angle = angle_start
                                    + (std::f64::consts::PI * (i as f64 / num_segments as f64));
                                let px = fighter.x + fixed_crater.radius * current_angle.cos();
                                let py = fighter.y + fixed_crater.radius * current_angle.sin();
                                points.push([px, py]);
                            }

                            polygon(color, &points, tc.transform, g);
                        }

                        // Draw bike/vehicles AFTER entities but BEFORE player
                        if sbrx_bike.visible
                            && (!FOG_OF_WAR_ENABLED
                                || fog_of_war.should_render_entity(
                                    sbrx_map_system.current_field_id,
                                    sbrx_bike.x,
                                    sbrx_bike.y,
                                ))
                        {
                            // Don't render bike in area instances
                            let should_render_bike = if let Some(ref _area_state) = current_area {
                                false // Never render bike in any area
                            } else {
                                true // Render normally in open world
                            };

                            if should_render_bike {
                                let vehicle_texture_to_draw = match (fighter.fighter_type, sbrx_bike.is_crashed) {
                                    (FighterType::Soldier, false) => Some(&sbrx_quad_texture),
                                    (FighterType::Soldier, true) => Some(&sbrx_quad_crashed_texture),
                                    (_, false) => Some(&sbrx_bike_texture),
                                    (_, true) => Some(&sbrx_bike_crashed_texture),
                                };

                                if let Some(vehicle_texture) = vehicle_texture_to_draw {
                                    image(
                                        vehicle_texture,
                                        tc.transform.trans(
                                            sbrx_bike.x - vehicle_texture.get_width() as f64 / 2.0,
                                            sbrx_bike.y - vehicle_texture.get_height() as f64 / 2.0,
                                        ),
                                        g,
                                    );
                                }

                                if !FOG_OF_WAR_ENABLED
                                    || fog_of_war.is_position_visible(
                                        sbrx_map_system.current_field_id,
                                        sbrx_bike.x,
                                        sbrx_bike.y,
                                    )
                                {
                                    fighter.draw_bike_interaction_indicator(
                                        oc,
                                        g,
                                        sbrx_bike.x,
                                        sbrx_bike.y,
                                        sbrx_bike.visible,
                                    );
                                }
                            }
                        }

                        spheres.iter().for_each(|s| {
                            if !s.exploding && !s.has_exploded {
                                ellipse(
                                    [0.0, 0.0, 0.0, 1.0],
                                    [
                                        s.x - s.size / 2.0 - 1.0,
                                        s.y - s.size / 2.0 - 1.0,
                                        s.size + 2.0,
                                        s.size + 2.0,
                                    ],
                                    tc.transform,
                                    g,
                                );
                                ellipse(
                                    [0.0, 1.0, 0.0, 1.0],
                                    [s.x - s.size / 2.0, s.y - s.size / 2.0, s.size, s.size],
                                    tc.transform,
                                    g,
                                );
                            } else if s.exploding {
                                let es = s.size * (1.0 - s.explosion_timer / 0.5) * 2.0;
                                ellipse(
                                    [1.0, 1.0, 1.0, 1.0],
                                    [s.x - es / 2.0, s.y - es / 2.0, es, es],
                                    tc.transform,
                                    g,
                                );
                            }
                        });
                        if !is_paused {
                            let (wmxr, _) = screen_to_world(&camera, mouse_x, mouse_y);
                            let flip = wmxr < fighter.x;
                            let sw = current_racer_texture.get_width() as f64;
                            let sh = current_racer_texture.get_height() as f64;
                            let idx = fighter.x - sw / 2.0;
                            let idy = fighter.y - sh / 2.0;
							
                            // Display Atomic State visual beneath player if Invincible 
                            // Logic: Prevent during stun recovery (stun_timer/knockback) and block-break.
                            // Added check: Only show if timer > 1.0s to prevent showing during the 1s post-collision penalty.
                            if fighter.invincible_timer > 1.0 
                                && fighter.stun_timer <= 0.0 
                                && fighter.knockback_duration <= 0.0 
                                && !block_system.is_stun_locked() 
                            {
                                let tex_w = atomic_state_texture.get_width() as f64;
                                let tex_h = atomic_state_texture.get_height() as f64;
                                // Center the effect on the player's position
                                image(&atomic_state_texture, tc.transform.trans(fighter.x - tex_w / 2.0, fighter.y - tex_h / 2.0), g);
                            }							
							
                            let mut dtf = tc.transform.trans(idx, idy);
                            if flip {
                                dtf = tc.transform.trans(idx + sw, idy).scale(-1.0, 1.0);
                            }
                            image(current_racer_texture, dtf, g);
                            fighter.draw_health_bar(tc, g);
							
                            // Draw Shift Function Indicator (Boost/Ranged)
                            if fighter.fighter_type == FighterType::Racer && fighter.boost_indicator_timer > 0.0 {
                                let indicator_tex = if fighter.boost {
                                    &set_boost_texture
                                } else {
                                    &set_ranged_texture
                                };
                                
                                // Position over HP bar (approx 80px above player as per draw_health_bar)
                                // draw_health_bar puts bar at fighter.y - 80.0
                                // We'll put this slightly above that
                                let ind_w = indicator_tex.get_width() as f64;
                                let _ind_h = indicator_tex.get_height() as f64;
                                let ind_x = fighter.x - ind_w / 2.0;
                                let ind_y = fighter.y - 125.0; // Above HP bar
                                
                                image(indicator_tex, tc.transform.trans(ind_x, ind_y), g);
                            }							
                        }
                        // Show fatigue icon as soon as stun ends (during vulnerability) and through the fatigue period.
                        if block_system.block_fatigue
                            || (block_system.block_broken && block_system.stun_lock_timer <= 0.0)
                        {
                            let radius = 60.0;
                            let (offset_x, offset_y) = (radius * 0.707, -radius * 0.707);

                            let tex_w = block_fatigue_texture.get_width() as f64;
                            let tex_h = block_fatigue_texture.get_height() as f64;

                            let img_center_x = fighter.x + offset_x;
                            let img_center_y = fighter.y + offset_y;

                            let img_x = img_center_x - tex_w / 2.0;
                            let img_y = img_center_y - tex_h / 2.0;

                            image(&block_fatigue_texture, tc.transform.trans(img_x, img_y), g);
                        } else if block_system.kinetic_intake_count > 0 {
                            let ki_count = block_system.kinetic_intake_count;
                            let ki_text = format!("{}", ki_count);
                            let ki_font_size = 24;
                            let ki_color = match ki_count {
                                1..=10 => [0.0, 1.0, 0.0, 1.0],  // GREEN
                                11..=17 => [1.0, 1.0, 0.0, 1.0], // YELLOW
                                18..=20 => [1.0, 0.0, 0.0, 1.0], // RED
                                _ => [1.0, 1.0, 1.0, 1.0],       // Fallback
                            };
                            let text_width = glyphs.width(ki_font_size, &ki_text).unwrap_or(0.0);
                            let radius = 60.0;
                            let positions = [(radius * 0.707, -radius * 0.707)];
                            for (offset_x, offset_y) in &positions {
                                let text_x = fighter.x + offset_x - text_width / 2.0;
                                let text_y = fighter.y + offset_y + ki_font_size as f64 / 2.0;
                                let padding = 1.0;
                                let backdrop_x = text_x - padding;
                                let backdrop_y = text_y - ki_font_size as f64 - padding;
                                let backdrop_width = text_width + (padding * 2.0);
                                let backdrop_height = ki_font_size as f64 + (padding * 2.0);
                                let backdrop_color = [0.2, 0.2, 0.2, 0.7];
                                rectangle(
                                    backdrop_color,
                                    [backdrop_x, backdrop_y, backdrop_width, backdrop_height],
                                    tc.transform,
                                    g,
                                );
                                text::Text::new_color(ki_color, ki_font_size)
                                    .draw(
                                        &ki_text,
                                        &mut glyphs,
                                        &tc.draw_state,
                                        tc.transform.trans(text_x, text_y),
                                        g,
                                    )
                                    .ok();
                            }
                        }
                        if FOG_OF_WAR_ENABLED {
                            fog_of_war.render_fog_overlay(sbrx_map_system.current_field_id, tc, g);
                        }
                        task_system.draw(oc, g, &mut glyphs);
                        fighter.draw_score(tc, oc, g);

                        if let Some(notification) = &task_reward_notification {
                            let font_size = 18;
                            let alpha = (notification.lifetime / 1.5).min(1.0) as f32;
                            let color = [0.0, 1.0, 0.0, alpha]; // Lime green, fading out

                            // Constants from fighter.rs's draw_score to align the text
                            let score_base_x = 20.0;
                            let score_base_y = 300.0;
                            let score_segment_height = 30.0;
                            let score_display_width = (20.0 + 7.5) * 3.0; // segment_width + spacing * digits
                            let score_end_x = score_base_x + score_display_width;

                            let text_x = score_end_x + 10.0;
                            let text_y_baseline = score_base_y + score_segment_height; // Align with bottom of score numbers

                            text::Text::new_color(color, font_size)
                                .draw(
                                    &notification.text,
                                    &mut glyphs,
                                    &oc.draw_state,
                                    oc.transform.trans(text_x, text_y_baseline),
                                    g,
                                )
                                .ok();
                        }
						
                        // --- TRACK NOTIFICATION (bottom right) ---
                        if let Some(ref notif) = track_notification {
                            let text = format!("track: {}", notif.track_name);
                            let font_size = 18;
                            let text_width = glyphs.width(font_size, &text).unwrap_or(150.0);
                            let padding = 8.0;
                            let box_width = text_width + padding * 2.0;
                            let box_height = font_size as f64 + padding * 2.0;
                            let box_x = screen_width - box_width - 20.0;
                            let box_y = screen_height - box_height - 20.0;
                            
                            // Dark gray background padding
                            rectangle(
                                [0.2, 0.2, 0.2, 0.8],
                                [box_x, box_y, box_width, box_height],
                                oc.transform,
                                g,
                            );
                            
                            // Green text
                            text::Text::new_color([0.0, 1.0, 0.0, 1.0], font_size)
                                .draw(
                                    &text,
                                    &mut glyphs,
                                    &oc.draw_state,
                                    oc.transform.trans(box_x + padding, box_y + padding + font_size as f64 - 2.0),
                                    g,
                                )
                                .ok();
                        }						

                        // --- WAVE UI ---
                        if let Some(wave_text) = wave_manager.get_ui_text() {
                            let font_size = 22;
                            let text_color = [1.0, 0.2, 0.2, 1.0]; // Red
                            let text_width = glyphs.width(font_size, &wave_text).unwrap_or(0.0);
                            let text_x = (screen_width - text_width) / 2.0;
                            let text_y = 50.0;
                            text::Text::new_color(text_color, font_size)
                                .draw(
                                    &wave_text,
                                    &mut glyphs,
                                    &oc.draw_state,
                                    oc.transform.trans(text_x, text_y),
                                    g,
                                )
                                .ok();
                        }

                        // --- ENDLESS ARENA TIMER ---
                        if endless_arena_mode_active {
                            let minutes = (endless_arena_timer / 60.0).floor() as u32;
                            let seconds = (endless_arena_timer % 60.0).floor() as u32;
                            let centiseconds =
                                ((endless_arena_timer.fract()) * 100.0).floor() as u32;
                            let timer_text = format!(
                                "ARENA TIME: {:02}:{:02} . {:02}",
                                minutes, seconds, centiseconds
                            );
							let kills_text = format!("ARENA SCORE: {:04}", arena_kill_count);

                            let font_size = 22;
                            let text_color = [1.0, 1.0, 1.0, 1.0]; // White
                            let text_width = glyphs.width(font_size, &timer_text).unwrap_or(0.0);
                            let kills_width = glyphs.width(font_size, &kills_text).unwrap_or(0.0);							
                            let text_x = (screen_width - text_width) / 2.0;
							let kills_x = (screen_width - kills_width) / 2.0;
                            let text_y = 50.0;
							let kills_y = text_y + 30.0;

                            text::Text::new_color(text_color, font_size)
                                .draw(
                                    &timer_text,
                                    &mut glyphs,
                                    &oc.draw_state,
                                    oc.transform.trans(text_x, text_y),
                                    g,
                                )
                                .ok();
								
                            text::Text::new_color([0.0, 1.0, 0.0, 1.0], font_size) // Green for kills
                                .draw(
                                    &kills_text,
                                    &mut glyphs,
                                    &oc.draw_state,
                                    oc.transform.trans(kills_x, kills_y),
                                    g,
                                )
                                .ok();								
                        }

                        block_system.draw_ui(oc, g);
                        fighter.draw_fuel_meter(oc, g, &mut glyphs);
                        fighter.draw_ammo_gauge(oc, g, &mut glyphs);

                        // Calculate level modifier for the current fighter for rendering
                        let mut total_level_mod_for_render = 0;
                        let active_traits_for_render = field_trait_manager
                            .get_active_traits_for_field(&sbrx_map_system.current_field_id);
                        for trait_instance in &active_traits_for_render {
                            let mut applies = false;
                            if let TraitTarget::Fighter(target_ft) = trait_instance.target {
                                if target_ft == fighter.fighter_type {
                                    applies = true;
                                }
                            }

                            if applies && trait_instance.attribute == StatAttribute::Level {
                                total_level_mod_for_render += trait_instance.modifier;
                            }
                        }

                        fighter.draw_stats_display(oc, g, &mut glyphs, total_level_mod_for_render);

                        // --- Render Active Field Traits ---
                        let active_traits = field_trait_manager
                            .get_active_traits_for_field(&sbrx_map_system.current_field_id);
                        if !active_traits.is_empty() {
                            let trait_font_size = 15;
                            let trait_color = [0.1, 1.0, 0.1, 1.0]; // green
                            let mut current_y = 20.0; // Position below stats, above fuel
                            let start_x = 600.0; //350

                            for trait_instance in active_traits {
                                text::Text::new_color(trait_color, trait_font_size)
                                    .draw(
                                        &trait_instance.description,
                                        &mut glyphs,
                                        &oc.draw_state,
                                        oc.transform.trans(start_x, current_y),
                                        g,
                                    )
                                    .ok();
                                current_y += trait_font_size as f64 + 5.0;
                            }
                        }

                        // --- NEW: Render Group Icon UI ---
                        fighter_hp_map.insert(fighter.fighter_type, fighter.current_hp);
                        // INCREASE THIS VALUE FOR MORE HORIZONTAL SPACING BETWEEN GROUP ICONS
                        let mut current_x = 25.0;
                        let start_y = 70.0;
                        let padding = 10.0;
                        let font_size = 14;
                        let text_color = [1.0, 1.0, 1.0, 1.0];

                        let fighter_types_in_group = [
                            FighterType::Racer,
                            FighterType::Soldier,
                            FighterType::Raptor,
                        ];

                        for ft in &fighter_types_in_group {
                            let has_joined = match ft {
                                FighterType::Racer => true,
                                FighterType::Soldier => soldier_has_joined,
                                FighterType::Raptor => raptor_has_joined,
                            };

                            // BUG FIX 2: Check if fighter is downed
                            if has_joined && !downed_fighters.contains(ft) {
                                let is_selected = fighter.fighter_type == *ft;
                                let icon_to_draw = if is_selected {
                                    group_icons_selected.get(ft)
                                } else {
                                    group_icons.get(ft)
                                };

                                if let Some(icon) = icon_to_draw {
                                    let icon_width = icon.get_width() as f64;
                                    let icon_height = icon.get_height() as f64;

                                    image(icon, oc.transform.trans(current_x, start_y), g);

                                    // Draw '+' sign if level up points are available
                                    if fighter.stat_points_to_spend.get(ft).unwrap_or(&0) > &0 {
                                        let plus_text = "+";
                                        let plus_font_size = 20;
                                        let plus_color = [1.0, 0.5, 0.0, 1.0]; // Orange
                                                                               // Position near top-right of the icon
                                        let plus_x = current_x + icon_width - 10.0;
                                        let plus_y_baseline = start_y + 10.0;
                                        text::Text::new_color(plus_color, plus_font_size)
                                            .draw(
                                                plus_text,
                                                &mut glyphs,
                                                &oc.draw_state,
                                                oc.transform.trans(plus_x, plus_y_baseline),
                                                g,
                                            )
                                            .ok();
                                    }

                                    let (_max_hp, current_hp) = match ft {
                                        FighterType::Racer => (
                                            RACER_LVL1_STATS.defense.hp,
                                            *fighter_hp_map.get(ft).unwrap_or(&0.0),
                                        ),
                                        FighterType::Soldier => (
                                            SOLDIER_LVL1_STATS.defense.hp,
                                            *fighter_hp_map.get(ft).unwrap_or(&0.0),
                                        ),
                                        FighterType::Raptor => (
                                            RAPTOR_LVL1_STATS.defense.hp,
                                            *fighter_hp_map.get(ft).unwrap_or(&0.0),
                                        ),
                                    };

                                    let hp_text = format!("{:.0}", current_hp.max(0.0));

                                    let text_y = start_y + icon_height + font_size as f64;

                                    text::Text::new_color(text_color, font_size)
                                        .draw(
                                            &hp_text,
                                            &mut glyphs,
                                            &oc.draw_state,
                                            oc.transform.trans(current_x, text_y),
                                            g,
                                        )
                                        .ok();

                                    // Draw strike animation AFTER all other icon elements
                                    if let Some(timer) = group_animation_timers.get(ft) {
                                        if *timer > 0.0 {
                                            let strike_tex = match ft {
                                                FighterType::Racer => &racer_textures.strike[2], // strike3.png
                                                FighterType::Soldier => &soldier_textures.strike[2],
                                                FighterType::Raptor => &raptor_textures.strike[2],
                                            };

                                            let strike_w = strike_tex.get_width() as f64;
                                            let strike_h = strike_tex.get_height() as f64;
                                            let strike_x =
                                                current_x + (icon_width / 2.0) - (strike_w / 2.0);
                                            let strike_y =
                                                start_y + (icon_height / 2.0) - (strike_h / 2.0);

                                            image(
                                                strike_tex,
                                                oc.transform.trans(strike_x, strike_y),
                                                g,
                                            );
                                        }
                                    }

                                    current_x += icon_width + padding;
                                }
                            }
                        }
						
                        if fighter.show_gear {
                            let gear_h = gear_texture.get_height() as f64;
                            // Drawing here ensures it is beneath the cursor logic that follows
                            image(&gear_texture, tc.transform.trans(fighter.x + 175.0, fighter.y - gear_h / 2.5), g);
                        }						

                        let (wmxc, wmxyc) = screen_to_world(&camera, mouse_x, mouse_y);
                        let dx_c = wmxc - fixed_crater.x;
                        let dy_c = wmxyc - fixed_crater.y;
                        let hr_c = fixed_crater.radius;
                        let vr_c = fixed_crater.radius * 0.75;
                        let ds_c = (dx_c * dx_c) / (hr_c * hr_c) + (dy_c * dy_c) / (vr_c * vr_c);
                        let cur_tex = match fighter.combat_mode {
                            CombatMode::CloseCombat => &strike_texture,
                            CombatMode::Ranged => &aim_texture,
                            CombatMode::Balanced => {
                                if ds_c <= 1.0 {
                                    &strike_texture
                                } else {
                                    &aim_texture
                                }
                            }
                        };
                        let cur_w = cur_tex.get_width() as f64;
                        let cur_h = cur_tex.get_height() as f64;
                        let can_shoot_ranged = fighter.combat_mode == CombatMode::Ranged
                            || (fighter.combat_mode == CombatMode::Balanced && ds_c > 1.0);
                        if can_shoot_ranged && shoot.cooldown > 0.0 {
                            piston_window::rectangle(
                                [1.0, 0.0, 0.0, 0.5],
                                [wmxc - cur_w / 2.0, wmxyc - cur_h / 2.0, cur_w, cur_h],
                                tc.transform,
                                g,
                            );
                        }
                        image(
                            cur_tex,
                            tc.transform.trans(wmxc - cur_w / 2.0, wmxyc - cur_h / 2.0),
                            g,
                        );
                        if strike.visible {
                            let strike_color = if block_system.is_kinetic_strike_active() {
                                [0.0, 1.0, 0.2, 0.8]
                            } else if combo_system.is_combo_strike_active() {
                                [0.0, 1.0, 0.2, 0.8]
                            } else {
                                [1.0, 1.0, 1.0, 0.8]
                            };
                            let num_slashes = if block_system.is_kinetic_strike_active() && fighter.fighter_type == FighterType::Raptor {
                                // RAPTOR kinetic strike uses triple slash visual
                                3
                            } else if combo_system.is_combo_strike_active() {
                                combo_finisher_slash_count
                            } else if rush_active && fighter.fighter_type == FighterType::Raptor {
                                // raptor rush attack uses triple slash visual
                                3
                            } else {
                                1
                            };
                            let angle_rad = strike.angle.to_radians();
                            let slash_base_half_width = 52.1;
                            let local_tip = [strike.length, 0.0];
                            let local_base1 = [0.0, slash_base_half_width];
                            let local_base2 = [0.0, -slash_base_half_width];
                            let local_polygon = [local_tip, local_base1, local_base2];
                            if num_slashes <= 1 {
                                polygon(
                                    strike_color,
                                    &local_polygon,
                                    tc.transform.trans(strike.x, strike.y).rot_rad(angle_rad),
                                    g,
                                );
                            } else if num_slashes == 2 {
                                polygon(
                                    strike_color,
                                    &local_polygon,
                                    tc.transform.trans(strike.x, strike.y).rot_rad(angle_rad),
                                    g,
                                );
                                let perpendicular_angle = angle_rad + std::f64::consts::PI / 2.0;
                                polygon(
                                    strike_color,
                                    &local_polygon,
                                    tc.transform
                                        .trans(strike.x, strike.y)
                                        .rot_rad(perpendicular_angle),
                                    g,
                                );
                            } else if num_slashes == 3 {
                                let base_transform =
                                    tc.transform.trans(strike.x, strike.y).rot_rad(angle_rad);
                                polygon(strike_color, &local_polygon, base_transform, g);
                                let back_offset_1 = -25.0;
                                let back_offset_2 = 25.0;
                                let side_offset = 18.0;
                                let first_add_transform =
                                    base_transform.trans(back_offset_1, side_offset);
                                polygon(strike_color, &local_polygon, first_add_transform, g);
                                let second_add_transform =
                                    base_transform.trans(back_offset_2, -side_offset);
                                polygon(strike_color, &local_polygon, second_add_transform, g);
                            }
                        }
                        if shoot.visible {
                            image(
                                current_ranged_marker_texture,
                                tc.transform.trans(
                                    wmxc - (current_ranged_marker_texture.get_width() as f64 / 2.0),
                                    wmxyc
                                        - (current_ranged_marker_texture.get_height() as f64 / 2.0),
                                ),
                                g,
                            );
                        }
                        if shoot.line_visible {
                            let sx = shoot.start_x;
                            let sy = shoot.start_y;
                            let ex = shoot.target_x;
                            let ey = shoot.target_y;
                            let ldxs = ex - sx;
                            let ldys = ey - sy;
                            let bls = (ldxs * ldxs + ldys * ldys).sqrt();
                            let blx = sx + ldxs * 0.5;
                            let bly = sy + ldys * 0.5;
                            image(
                                current_ranged_blur_texture,
                                tc.transform.trans(
                                    blx - (current_ranged_blur_texture.get_width() as f64 / 2.0),
                                    bly - (current_ranged_blur_texture.get_height() as f64 / 2.0),
                                ),
                                g,
                            );
                            // No bullet line  - only visual effects remain
                            //                            if fighter.fighter_type != FighterType::Racer {
                            //                                line([1.0, 1.0, 1.0, 0.8], 2.0, [sx, sy, ex, ey], tc.transform, g);
                            //                            }
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
                                    line(col, w, [sx + ox, sy + oy, efx, efy], tc.transform, g);
                                }
                            }
                        }
                        for text in &damage_texts {
                            let final_color = text.color;
                            let text_width = glyphs.width(16, &text.text).unwrap_or(0.0);
                            let text_x = text.x - text_width / 2.0;
                            text::Text::new_color(final_color, 16)
                                .draw(
                                    &text.text,
                                    &mut glyphs,
                                    &tc.draw_state,
                                    tc.transform.trans(text_x, text.y),
                                    g,
                                )
                                .unwrap_or_else(|e| eprintln!("Failed to draw damage text: {}", e));
                        }
						let field_display_text = if let Some(area_ref) = current_area.as_ref() {
							match area_ref.area_type {
                                AreaType::RaptorNest => format!(
                                    "FLATLINE_field.x[1]y[0] RAPTOR NEST[{}]",
                                    area_ref.floor
                                ),
                                AreaType::Bunker => format!(
                                    "FLATLINE_field.x[-25]y[25] FORT SILO::BUNKER[{}]",
                                    area_ref.floor
                                ),
                            }
                        } else {
                            let mut base_text = sbrx_map_system.get_display_string();
                            let current_field = sbrx_map_system.current_field_id;
                            if current_field == SbrxFieldId(0, 0) {
                                base_text.push_str(" RACETRACK");
                            } else if current_field == SbrxFieldId(-2, 5) {
                                base_text.push_str(" ROCKETBAY");
                            } else if current_field == SbrxFieldId(-25, 25) {
                                base_text.push_str(" FORT SILO");
                            }
                            base_text
                        };
                        let field_font_size_sbrx: u32 = 14;
                        let field_text_x_pos_sbrx = 10.0;
                        let field_text_y_baseline_sbrx = 20.0;
                        let text_width_sbrx =
                            match glyphs.width(field_font_size_sbrx, &field_display_text) {
                                Ok(w) => w,
                                Err(_) => {
                                    field_display_text.chars().count() as f64
                                        * (field_font_size_sbrx as f64 * 0.6)
                                }
                            };
                        let padding_sbrx = 2.0;
                        let bg_rect_x_sbrx = field_text_x_pos_sbrx - padding_sbrx;
                        let bg_rect_y_sbrx =
                            field_text_y_baseline_sbrx - field_font_size_sbrx as f64 - padding_sbrx;
                        let bg_rect_width_sbrx = text_width_sbrx + 2.0 * padding_sbrx;
                        let bg_rect_height_sbrx = field_font_size_sbrx as f64 + 2.0 * padding_sbrx;
                        let black_color_sbrx = [0.0, 0.0, 0.0, 1.0];
                        rectangle(
                            black_color_sbrx,
                            [
                                bg_rect_x_sbrx,
                                bg_rect_y_sbrx,
                                bg_rect_width_sbrx,
                                bg_rect_height_sbrx,
                            ],
                            oc.transform,
                            g,
                        );
                        text(
                            [1.0, 1.0, 1.0, 1.0],
                            field_font_size_sbrx,
                            &field_display_text,
                            &mut glyphs,
                            oc.transform
                                .trans(field_text_x_pos_sbrx, field_text_y_baseline_sbrx),
                            g,
                        )
                        .unwrap_or_else(|e| eprintln!("Failed to draw field text: {:?}", e));
                        if FOG_OF_WAR_ENABLED {
                            if fog_of_war.is_fog_enabled(sbrx_map_system.current_field_id) {
                                let (explored, total, percentage) = fog_of_war
                                    .get_exploration_stats(sbrx_map_system.current_field_id);
                                let exploration_text = format!(
                                    "EXPLORED: {:.1}% ({}/{})",
                                    percentage, explored, total
                                );
                                let exploration_font_size = 14;
                                let exploration_color = [0.8, 0.8, 0.8, 1.0];
                                text(
                                    exploration_color,
                                    exploration_font_size,
                                    &exploration_text,
                                    &mut glyphs,
                                    oc.transform.trans(10.0, 60.0),
                                    g,
                                )
                                .ok();
                            }
                        }
                        if show_raptor_interaction_prompt
                            || show_info_post_prompt
                            || show_survivor_interaction_prompt
                            || show_fort_silo_survivor_prompt
                            || show_grand_commander_prompt
                            || show_finale_info_post_prompt
                            || show_soldier_interaction_prompt
                            || show_racetrack_soldier_prompt
                            || show_finale_info_post_prompt
                            || show_soldier_interaction_prompt
                        {
                            let prompt_text = "INTERACT [E]";
                            let prompt_font_size = 24;
                            let prompt_color = [1.0, 1.0, 1.0, 1.0];
                            let text_width =
                                glyphs.width(prompt_font_size, prompt_text).unwrap_or(0.0);
                            let text_x = (screen_width - text_width) / 2.0;
                            let text_y = screen_height - 100.0;
                            text::Text::new_color(prompt_color, prompt_font_size)
                                .draw(
                                    prompt_text,
                                    &mut glyphs,
                                    &oc.draw_state,
                                    oc.transform.trans(text_x, text_y),
                                    g,
                                )
                                .ok();
                        }

                        if show_fighter_jet_prompt {
                            let prompt_text1 = "[E] board FIGHTERJET";
                            let prompt_font_size = 20;
                            let prompt_color = [0.0, 1.0, 0.0, 1.0];
                            let base_prompt_y = 100.0;
                            let text_x1 = (screen_width
                                - glyphs.width(prompt_font_size, prompt_text1).unwrap_or(0.0))
                                / 2.0;
                            let text_y1 = base_prompt_y;
                            text(
                                prompt_color,
                                prompt_font_size,
                                prompt_text1,
                                &mut glyphs,
                                oc.transform
                                    .trans(text_x1, text_y1 + prompt_font_size as f64),
                                g,
                            )
                            .unwrap_or_else(|err| {
                                eprintln!("Failed to draw fighterjet prompt text1: {}", err)
                            });
                        }
                        if show_raptor_nest_prompt {
                            let prompt_text = "enter RAPTOR NEST [E]";
                            let prompt_font_size = 20;
                            let prompt_color = [0.0, 1.0, 0.0, 1.0];
                            let text_width =
                                glyphs.width(prompt_font_size, prompt_text).unwrap_or(0.0);
                            let text_x = (screen_width - text_width) / 2.0;
                            let text_y = 100.0;
                            text(
                                prompt_color,
                                prompt_font_size,
                                prompt_text,
                                &mut glyphs,
                                oc.transform.trans(text_x, text_y + prompt_font_size as f64),
                                g,
                            )
                            .unwrap_or_else(|err| {
                                eprintln!("Failed to draw raptor nest prompt text: {}", err)
                            });
                        }

                        if bunker_entry_choice == BunkerEntryChoice::AwaitingInput {
                            let prompt_text1 = "[1] ENTER";
                            let prompt_text2 = "[2] RESTART WAVES";
                            let prompt_font_size = 20;
                            let prompt_color = [0.0, 1.0, 0.0, 1.0];
                            let text1_width =
                                glyphs.width(prompt_font_size, prompt_text1).unwrap_or(0.0);
                            let text2_width =
                                glyphs.width(prompt_font_size, prompt_text2).unwrap_or(0.0);

                            let text1_x = (screen_width - text1_width) / 2.0;
                            let text2_x = (screen_width - text2_width) / 2.0;
                            let text_y1 = 250.0;
                            let text_y2 = text_y1 + 30.0;

                            text(
                                prompt_color,
                                prompt_font_size,
                                prompt_text1,
                                &mut glyphs,
                                oc.transform
                                    .trans(text1_x, text_y1 + prompt_font_size as f64),
                                g,
                            )
                            .ok();
                            text(
                                prompt_color,
                                prompt_font_size,
                                prompt_text2,
                                &mut glyphs,
                                oc.transform
                                    .trans(text2_x, text_y2 + prompt_font_size as f64),
                                g,
                            )
                            .ok();
                        }

                        if show_bunker_prompt {
                            let prompt_text = "enter BUNKER [E]";
                            let prompt_font_size = 20;
                            let prompt_color = [0.0, 1.0, 0.0, 1.0];
                            let text_width =
                                glyphs.width(prompt_font_size, prompt_text).unwrap_or(0.0);
                            let text_x = (screen_width - text_width) / 2.0;
                            let text_y = 100.0;
                            text(
                                prompt_color,
                                prompt_font_size,
                                prompt_text,
                                &mut glyphs,
                                oc.transform.trans(text_x, text_y + prompt_font_size as f64),
                                g,
                            )
                            .unwrap_or_else(|err| {
                                eprintln!("Failed to draw bunker prompt text: {}", err)
                            });
                        }

                        if show_raptor_nest_exit_prompt {
                            let prompt_text = "EXIT [E]";
                            let prompt_font_size = 20;
                            let prompt_color = [0.0, 1.0, 0.0, 1.0];
                            let text_width =
                                glyphs.width(prompt_font_size, prompt_text).unwrap_or(0.0);
                            let text_x = (screen_width - text_width) / 2.0;
                            let text_y = 100.0;
                            text(
                                prompt_color,
                                prompt_font_size,
                                prompt_text,
                                &mut glyphs,
                                oc.transform.trans(text_x, text_y + prompt_font_size as f64),
                                g,
                            )
                            .unwrap_or_else(|err| {
                                eprintln!("Failed to draw raptor nest exit prompt text: {}", err)
                            });
                        }

                        if show_bunker_exit_prompt {
                            let prompt_text = "EXIT [E]";
                            let prompt_font_size = 20;
                            let prompt_color = [0.0, 1.0, 0.0, 1.0];
                            let text_width =
                                glyphs.width(prompt_font_size, prompt_text).unwrap_or(0.0);
                            let text_x = (screen_width - text_width) / 2.0;
                            let text_y = 100.0;
                            text(
                                prompt_color,
                                prompt_font_size,
                                prompt_text,
                                &mut glyphs,
                                oc.transform.trans(text_x, text_y + prompt_font_size as f64),
                                g,
                            )
                            .unwrap_or_else(|err| {
                                eprintln!("Failed to draw bunker exit prompt text: {}", err)
                            });
                        }

                        if show_bunker_floor_transition_prompt {
                            if let Some(target_floor) = target_floor_from_prompt {
								let prompt_text = match current_area.as_ref() {
									Some(area) if target_floor > area.floor => "ASCEND [E]",
									_ => "DESCEND [E]",
								};
                                let prompt_font_size = 20;
                                let prompt_color = [0.0, 1.0, 0.0, 1.0];
                                let text_width =
                                    glyphs.width(prompt_font_size, prompt_text).unwrap_or(0.0);
                                let text_x = (screen_width - text_width) / 2.0;
                                let text_y = 100.0;
                                text(
                                    prompt_color,
                                    prompt_font_size,
                                    prompt_text,
                                    &mut glyphs,
                                    oc.transform.trans(text_x, text_y + prompt_font_size as f64),
                                    g,
                                )
                                .unwrap_or_else(|err| {
                                    eprintln!(
                                        "Failed to draw bunker transition prompt text: {}",
                                        err
                                    )
                                });
                            }
                        }

                        if is_paused {
                            let (w, h) = (
                                pause_screen_texture.get_width() as f64,
                                pause_screen_texture.get_height() as f64,
                            );
                            let x = (screen_width - w) / 2.0;
                            let y = (screen_height - h) / 2.0;
                            image(&pause_screen_texture, c.transform.trans(x, y), g);

                            // Draw player on top of pause screen
                            let (wmxr, _) = screen_to_world(&camera, mouse_x, mouse_y);
                            let flip = wmxr < fighter.x;
                            let sw = current_racer_texture.get_width() as f64;
                            let sh = current_racer_texture.get_height() as f64;
                            let idx = fighter.x - sw / 2.0;
                            let idy = fighter.y - sh / 2.0;
                            let mut dtf = tc.transform.trans(idx, idy);
                            if flip {
                                dtf = tc.transform.trans(idx + sw, idy).scale(-1.0, 1.0);
                            }
                            image(current_racer_texture, dtf, g);
                            fighter.draw_health_bar(tc, g);

                            fighter.draw_inputs_display(oc, g, &inputs_display_texture);
                        }										

                        // --- Find hovered CPU entity ---
                        let mut hovered_cpu_name: Option<String> = None;
                        if !is_paused {
                            // Only check for hover when not paused
                            let (world_mouse_x, world_mouse_y) =
                                screen_to_world(&camera, mouse_x, mouse_y);

                            for cpu in &cpu_entities {
                                // Only check visible entities
                                if !FOG_OF_WAR_ENABLED
                                    || fog_of_war.should_render_entity(
                                        sbrx_map_system.current_field_id,
                                        cpu.x,
                                        cpu.y,
                                    )
                                {
                                    let dx = world_mouse_x - cpu.x;
                                    let dy = world_mouse_y - cpu.y;
                                    let distance_sq = dx * dx + dy * dy;
                                    let hover_radius = 50.0;

                                    if distance_sq < hover_radius * hover_radius {
                                        let name = match cpu.variant {
                                            CpuVariant::GiantMantis => "GIANT MANTIS",
                                            CpuVariant::BloodIdol => "BLOOD IDOL",
                                            CpuVariant::Rattlesnake => "RATTLESNAKE",
                                            CpuVariant::GiantRattlesnake => "GIANT RATTLESNAKE",
                                            CpuVariant::Raptor => "RAPTOR",
                                            CpuVariant::TRex => "T-REX",
                                            CpuVariant::VoidTempest => "VOID TEMPEST",
                                            CpuVariant::LightReaver => "LIGHT REAVER",
                                            CpuVariant::NightReaver => "NIGHT REAVER",
                                            CpuVariant::RazorFiend => "RAZOR FIEND",
                                        };
                                        hovered_cpu_name = Some(format!("{}", name));
                                        break;
                                    }
                                }
                            }
                        }

                        // --- Render CPU Tooltip ---
                        if let Some(name) = hovered_cpu_name {
                            let font_size = 14;
                            let text_color = [1.0, 0.27, 0.0, 1.0]; // red orange
                            let padding = 5.0;

                            let text_width = glyphs.width(font_size, &name).unwrap_or(0.0);
                            let bg_width = text_width + padding * 2.0;
                            let bg_height = font_size as f64 + padding * 2.0;

                            // Position tooltip above and to the right of the cursor
                            let tooltip_x = mouse_x + 15.0;
                            let tooltip_y = mouse_y - bg_height - 5.0;

                            // Background
                            rectangle(
                                [0.1, 0.1, 0.1, 0.8], // Dark semi-transparent
                                [tooltip_x, tooltip_y, bg_width, bg_height],
                                oc.transform, // Use original context for screen-space UI
                                g,
                            );

                            // Text is drawn with y as baseline.
                            let text_baseline_y = tooltip_y + padding + font_size as f64;

                            text::Text::new_color(text_color, font_size)
                                .draw(
                                    &name,
                                    &mut glyphs,
                                    &oc.draw_state,
                                    oc.transform.trans(tooltip_x + padding, text_baseline_y),
                                    g,
                                )
                                .ok();
                        }

                        // Draw the chatbox on top of everything including pause screen
                        chatbox.draw(oc, g, &mut glyphs);

                        // Draw on-screen warnings on top of everything
                        chatbox.draw_warnings(oc, g, &mut glyphs);

                        glyphs.factory.encoder.flush(device);
                    });
                }

                if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
                    if !block_system.is_stun_locked() && fighter.stun_timer <= 0.0 {
                        if fighter.is_reloading {
                            // Disable LMB input during reload
                        } else {
                            // Apply combat action slowdown when on bike
                            if fighter.state == RacerState::OnBike {
                                fighter.combat_action_slowdown_timer = 0.25;
                            }

                            if block_system.rmb_held {
                                if block_system.kinetic_intake_count > 0
                                    && !block_system.block_fatigue
                                    && !block_system.block_broken
                                {
                                    let intake_count = block_system.kinetic_intake_count;
                                    // RANGED KINETIC STRIKE = off. remove fighter.x,.y,
                                    // let (wmx, wmy) = screen_to_world(&camera, mouse_x, mouse_y);
                                    block_system.perform_kinetic_strike(
                                        //    wmx,
                                        //     wmy,
                                        fighter.x,
                                        fighter.y,
                                        &mut fighter,
                                        &mut cpu_entities,
                                        &mut strike,
                                        &audio_manager,
                                        line_y,
                                        &mut damage_texts,
                                        &mut task_system,
                                        &mut combo_system,
										is_paused,
                                    );
									
                                    // Soldier Reward: 1.5s Invincibility on Kinetic Strike
                                    if fighter.fighter_type == FighterType::Soldier {
                                        fighter.invincible_timer = 2.0;
                                        chatbox.add_interaction(vec![
                                            ("ATOMIC-STATE", MessageType::Warning),
                                        ]);
                                    }									

                                    let texture_index = if intake_count <= 10 {
                                        0 // Green
                                    } else if intake_count <= 17 {
                                        1 // Yellow
                                    } else {
                                        2 // Red
                                    };

                                    active_kinetic_strike_effects.push(
                                        KineticStrikeEffectInstance {
                                            x: fighter.x,
                                            y: fighter.y,
                                            lifetime: 0.25,
                                            max_lifetime: 0.25,
                                            texture_index,
                                        },
                                    );

                                    strike_animation_timer = 0.25;
                                    if !current_strike_textures.is_empty() {
                                        current_racer_texture = &current_strike_textures
                                            [strike_frame % current_strike_textures.len()];
                                        strike_frame =
                                            (strike_frame + 1) % current_strike_textures.len();
                                    }
                                    movement_active = false;
                                    backpedal_active = false;
                                    rush_active = false;
                                }
                            } else {
                                lmb_held = true; // Set held flag for continuous strikes
                                if block_system.active {
                                    block_system.deactivate();
                                    if !is_high_priority_animation_active(
                                        rush_active,
                                        strike_animation_timer,
                                        false,
                                        false,
                                    ) && !movement_active
                                        && !backpedal_active
                                    {
                                        current_racer_texture = current_idle_texture;
                                    }
                                }
                                let (wmx, wmy) = screen_to_world(&camera, mouse_x, mouse_y);
                                let dx = wmx - fixed_crater.x;
                                let dy = wmy - fixed_crater.y;
                                let hr = fixed_crater.radius;
                                let vr = fixed_crater.radius * 0.75;
                                let dsq = (dx * dx) / (hr * hr) + (dy * dy) / (vr * vr);

                                let perform_melee = match fighter.combat_mode {
                                    CombatMode::CloseCombat => true,
                                    CombatMode::Ranged => false,
                                    CombatMode::Balanced => dsq <= 1.0,
                                };

                                if perform_melee {
                                    handle_melee_strike(
                                        &mut fighter,
                                        &camera,
                                        mouse_x,
                                        mouse_y,
                                        &mut fixed_crater,
                                        &mut combo_system,
                                        &mut strike,
                                        &audio_manager,
										&mut chatbox,
                                        &current_strike_textures,
                                        &mut current_racer_texture,
                                        &mut strike_frame,
                                        &mut strike_animation_timer,
                                        &mut movement_active,
                                        &mut backpedal_active,
                                        &mut frontal_strike_timer,
                                        &mut frontal_strike_angle,
                                        &mut frontal_strike_color,
                                        &mut frontal_strike_is_special,
                                        &mut combo_finisher_slash_count,
                                        &mut cpu_entities,
                                        &mut damage_texts,
										is_paused,
                                    );
                                    melee_rapid_fire_timer = MELEE_RAPID_FIRE_RATE;
                                } else {
                                    // Ranged Attack
                                    if fighter.fighter_type != FighterType::Raptor {
                                        if fighter.fighter_type == FighterType::Soldier {
                                            //lmb_held = true;
                                            soldier_rapid_fire_timer = 0.0;
                                        } else if fighter.fighter_type == FighterType::Racer
                                            && shoot.cooldown <= 0.0
                                        {
                                            audio_manager.play_sound_effect("ranged").ok();
                                            shoot.trigger(fighter.x, fighter.y, wmx, wmy);
                                            shoot.cooldown = RACER_RANGED_COOLDOWN;
                                            current_racer_texture = current_ranged_texture;
                                            strike_animation_timer = 0.25;
                                            movement_active = false;
                                            backpedal_active = false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(Button::Mouse(MouseButton::Right)) = e.press_args() {
                    if !block_system.is_stun_locked() && fighter.stun_timer <= 0.0 && !lmb_held && !fighter.is_reloading {
					// Force raptor out of flight mode when blocking
					if fighter.fighter_type == FighterType::Raptor && fighter.state == RacerState::OnBike {
						fighter.state = RacerState::OnFoot;
						shift_override_active = false;
						
						// Update textures for foot mode
						let tex_set = &raptor_textures;
						update_current_textures(
							&fighter,
							tex_set,
							&mut current_idle_texture,
							&mut current_fwd_texture,
							&mut current_backpedal_texture,
							&mut current_block_texture,
							&mut current_block_break_texture,
							&mut current_ranged_texture,
							&mut current_ranged_marker_texture,
							&mut current_ranged_blur_texture,
							&mut current_rush_texture,
							&mut current_strike_textures,
							shift_held,
						);
					}						
                        if block_system.activate(&audio_manager) {
                            current_racer_texture = current_block_texture;
                            movement_active = false;
                            movement_timer = 0.0;
                            backpedal_active = false;
                            backpedal_timer = 0.0;
                        }
                    }
                }
                if let Some(Button::Mouse(MouseButton::Right)) = e.release_args() {
                    block_system.deactivate();
                    if !block_break_animation_active
                        && !is_high_priority_animation_active(
                            rush_active,
                            strike_animation_timer,
                            false,
                            false,
                        )
                        && !movement_active
                        && !backpedal_active
                    {
                        current_racer_texture = current_idle_texture;
                    }
                }

                if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
                    lmb_held = false;
                    soldier_rapid_fire_timer = 0.0;
                    melee_rapid_fire_timer = 0.0;
                }

                if let Some(Button::Keyboard(key)) = e.press_args() {
                    //
                    let mut key_handled_by_lvl_up = false;
                    match lvl_up_state {
                        LvlUpState::PendingTab {
                            fighter_type: pending_fighter,
                        } => {
                            if key == Key::Tab && pending_fighter == fighter.fighter_type {
                                let points = fighter
                                    .stat_points_to_spend
                                    .get(&fighter.fighter_type)
                                    .unwrap_or(&0);
                                if *points > 0 {
                                    lvl_up_state = LvlUpState::SelectingStat;
                                    let fighter_name = match fighter.fighter_type {
                                        FighterType::Racer => "RACER",
                                        FighterType::Soldier => "SOLDIER",
                                        FighterType::Raptor => "RAPTOR",
                                    };
                                    chatbox.add_interaction(vec![(
                                        &format!(
                                            "!! +[1] to DEF[Z] ATK[X] SPD[C] [{}] !!",
                                            fighter_name
                                        ),
                                        MessageType::Dialogue,
                                    )]);
                                }
                                key_handled_by_lvl_up = true;
                            }
                        }
                        LvlUpState::SelectingStat => {
                            let mut choice: Option<StatChoice> = None;
                            let mut stat_name = "";
                            match key {
                                Key::Z => {
                                    choice = Some(StatChoice::Def);
                                    stat_name = "DEF";
                                    key_handled_by_lvl_up = true;
                                }
                                Key::X => {
                                    choice = Some(StatChoice::Atk);
                                    stat_name = "ATK";
                                    key_handled_by_lvl_up = true;
                                }
                                Key::C => {
                                    choice = Some(StatChoice::Spd);
                                    stat_name = "SPD";
                                    key_handled_by_lvl_up = true;
                                }
                                _ => {}
                            }
                            if let Some(stat_choice) = choice {
                                let fighter_name = match fighter.fighter_type {
                                    FighterType::Racer => "RACER",
                                    FighterType::Soldier => "SOLDIER",
                                    FighterType::Raptor => "RAPTOR",
                                };
                                chatbox.add_interaction(vec![(
                                    &format!(
                                        "!! CONFIRM: +1 {} [{}]  y/n !!",
                                        stat_name, fighter_name
                                    ),
                                    MessageType::Dialogue,
                                )]);
                                lvl_up_state = LvlUpState::ConfirmingStat {
                                    stat_to_increase: stat_choice,
                                    fighter_type: fighter.fighter_type,
                                };
                            }
                        }
                        LvlUpState::ConfirmingStat {
                            stat_to_increase,
                            fighter_type,
                        } => {
                            match key {
                                Key::Y => {
                                    let points = fighter
                                        .stat_points_to_spend
                                        .entry(fighter_type)
                                        .or_insert(0);
                                    if *points > 0 {
                                        *points -= 1;

                                        // 1. Modify ONLY the base stats map for the permanent increase.
                                        if let Some(base_stats) =
                                            base_fighter_stats_map.get_mut(&fighter_type)
                                        {
                                            match stat_to_increase {
                                                StatChoice::Def => {
                                                    base_stats.defense.hp +=
                                                        combat::stats::HP_PER_DEFENSE_POINT
                                                }
                                                StatChoice::Atk => {
                                                    base_stats.attack.melee_damage +=
                                                        combat::stats::DAMAGE_PER_ATTACK_POINT;
                                                    base_stats.attack.ranged_damage +=
                                                        combat::stats::DAMAGE_PER_ATTACK_POINT;
                                                }
                                                StatChoice::Spd => {
                                                    base_stats.speed.run_speed +=
                                                        combat::stats::SPEED_PER_SPEED_POINT
                                                }
                                            }
                                        }

                                        // 2. Recalculate active stats by applying traits to the new base stats.
                                        if let Some(active_stats_to_update) =
                                            fighter_stats_map.get_mut(&fighter_type)
                                        {
                                            if let Some(new_base_stats) =
                                                base_fighter_stats_map.get(&fighter_type)
                                            {
                                                // Start with the new permanent stats.
                                                *active_stats_to_update = *new_base_stats;

                                                // Re-apply any active field traits.
                                                let active_traits = field_trait_manager
                                                    .get_active_traits_for_field(
                                                        &sbrx_map_system.current_field_id,
                                                    );
                                                let mut total_level_mod = 0;
                                                for trait_instance in &active_traits {
                                                    if let TraitTarget::Fighter(target_ft) =
                                                        trait_instance.target
                                                    {
                                                        if target_ft == fighter_type {
                                                            if trait_instance.attribute
                                                                == StatAttribute::Level
                                                            {
                                                                total_level_mod +=
                                                                    trait_instance.modifier;
                                                            }
                                                        }
                                                    }
                                                }

                                                if total_level_mod != 0 {
                                                    active_stats_to_update.defense.hp +=
                                                        total_level_mod as f64
                                                            * combat::stats::HP_PER_DEFENSE_POINT;
                                                    active_stats_to_update.attack.melee_damage +=
                                                        total_level_mod as f64
                                                            * combat::stats::DAMAGE_PER_ATTACK_POINT;
                                                    active_stats_to_update.attack.ranged_damage +=
                                                        total_level_mod as f64
                                                            * combat::stats::DAMAGE_PER_ATTACK_POINT;
                                                    active_stats_to_update.speed.run_speed +=
                                                        total_level_mod as f64
                                                            * combat::stats::SPEED_PER_SPEED_POINT;
                                                }
                                            }
                                        }

                                        // 3. If the leveled-up fighter is active, update their stats immediately for responsiveness.
                                        if fighter_type == fighter.fighter_type {
                                            if let Some(recalculated_stats) =
                                                fighter_stats_map.get(&fighter_type)
                                            {
                                                fighter.stats = *recalculated_stats;
                                                fighter.max_hp = recalculated_stats.defense.hp;
                                                fighter.melee_damage =
                                                    recalculated_stats.attack.melee_damage;
                                                fighter.ranged_damage =
                                                    recalculated_stats.attack.ranged_damage;
                                                fighter.run_speed =
                                                    recalculated_stats.speed.run_speed;
                                                fighter.current_hp = fighter.max_hp;
                                                // Heal to new max HP
                                            }
                                        }

                                        if *points > 0 {
                                            lvl_up_state = LvlUpState::SelectingStat;
                                            let fighter_name_str = match fighter_type {
                                                FighterType::Racer => "RACER",
                                                FighterType::Soldier => "SOLDIER",
                                                FighterType::Raptor => "RAPTOR",
                                            };
                                            chatbox.add_interaction(vec![(
                                                &format!(
                                                    "!! +[1] to DEF[Z] ATK[X] SPD[C] [{}] !!",
                                                    fighter_name_str
                                                ),
                                                MessageType::Dialogue,
                                            )]);
                                        } else {
                                            lvl_up_state = LvlUpState::None;
                                            chatbox.add_interaction(vec![(
                                                "STAT INCREASED!",
                                                MessageType::Info,
                                            )]);
                                        }
                                    }
                                    key_handled_by_lvl_up = true;
                                }
                                Key::N => {
                                    lvl_up_state = LvlUpState::SelectingStat;
                                    let fighter_name = match fighter.fighter_type {
                                        FighterType::Racer => "RACER",
                                        FighterType::Soldier => "SOLDIER",
                                        FighterType::Raptor => "RAPTOR",
                                    };
                                    chatbox.add_interaction(vec![(
                                        &format!(
                                            "!! +[1] to DEF[Z] ATK[X] SPD[C] [{}] !!",
                                            fighter_name
                                        ),
                                        MessageType::Dialogue,
                                    )]);
                                    key_handled_by_lvl_up = true;
                                }
                                _ => {}
                            }
                        }
                        LvlUpState::None => {}
                    }

                    if key_handled_by_lvl_up {
                        continue; // Skip other key handlers if the level-up system used the key
                    }

                    match key {
                        Key::G => {
                            fighter.show_gear = !fighter.show_gear;
                        }						
                        Key::D1 => {
                            if shift_held {
                                println!("Shift + 1 was pressed!");
                                // ADD YOUR SHIFT+1 LOGIC HERE
                            } else if bunker_entry_choice == BunkerEntryChoice::AwaitingInput {
                                // Enter bunker without enemies/waves (peaceful entry).
                                //println!("[BUNKER] Entering peacefully.");

                                // Reset the choice state
                                bunker_entry_choice = BunkerEntryChoice::None;

                                // --- BUNKER ENTRY LOGIC (PEACEFUL) ---
                                area_entrance_x = fighter.x;
                                area_entrance_y = fighter.y;

                                // Waves are NOT active, so pass false.
                                let (new_area, player_start_pos) =
                                    AreaState::new(AreaType::Bunker, 1, false, true);
                                current_area = Some(new_area);
                                fighter.x = player_start_pos.0;
                                fighter.y = player_start_pos.1;
                                camera.x = fighter.x;
                                camera.y = fighter.y;
                                cpu_entities.clear(); // No CPUs
                                wave_manager.reset(); // No waves
                            }
                        }
                        Key::D2 => {
                            if shift_held {
                                println!("Shift + 2 was pressed!");
                                // ADD YOUR SHIFT+2 LOGIC HERE
                            } else if bunker_entry_choice == BunkerEntryChoice::AwaitingInput {
                                // Enter bunker and restart waves.
                                //println!("[BUNKER] Restarting waves.");

                                // Reset the choice state
                                bunker_entry_choice = BunkerEntryChoice::None;

                                // Reset wave progress
                                completed_bunker_waves.clear();
								explored_bunker_floors.clear();
                                bunker_waves_fully_completed = false;

                                // Reset Razor Fiend and Grand Commander state for the restart
                                razor_fiend_defeated_flag = false;
                                task_system.razor_fiend_defeated = 0;
                                grand_commander_dialogue_triggered = false;
                                //println!("[BUNKER] Razor Fiend boss state has been reset.");

                                // --- BUNKER ENTRY LOGIC (WAVES RESTARTED) ---
                                area_entrance_x = fighter.x;
                                area_entrance_y = fighter.y;

                                let mut bunker_wave_definitions: HashMap<i32, u32> = HashMap::new();
                                bunker_wave_definitions.insert(1, 2);
                                bunker_wave_definitions.insert(0, 2);
                                bunker_wave_definitions.insert(-1, 2);
                                bunker_wave_definitions.insert(-2, 3);

                                let waves_for_floor_1 =
                                    *bunker_wave_definitions.get(&1).unwrap_or(&0);
                                // Lockdown is active because we are restarting waves.
                                let lockdown_active = waves_for_floor_1 > 0;

                                if waves_for_floor_1 > 0 {
                                    wave_manager.start_encounter(waves_for_floor_1);
                                }
                                let (new_area, player_start_pos) =
                                    AreaState::new(AreaType::Bunker, 1, lockdown_active, false);

                                let should_spawn_reavers = new_area.spawn_night_reavers;
                                current_area = Some(new_area);
                                fighter.x = player_start_pos.0;
                                fighter.y = player_start_pos.1;
                                camera.x = fighter.x;
                                camera.y = fighter.y;
                                cpu_entities.clear();
                                if should_spawn_reavers && CPU_ENABLED {
                                    println!(
                                        "Spawning Night Reavers in bunker floor 1 (Wave Restart)"
                                    );
                                    for _ in 0..4 {
                                        if cpu_entities.len() < 10 {
                                            let reaver_x = safe_gen_range(
                                                BUNKER_ORIGIN_X + 50.0,
                                                BUNKER_ORIGIN_X + BUNKER_WIDTH - 50.0,
                                                "NightReaver x",
                                            );
                                            let reaver_y = safe_gen_range(
                                                BUNKER_ORIGIN_Y + 50.0,
                                                BUNKER_ORIGIN_Y + BUNKER_HEIGHT - 50.0,
                                                "NightReaver y",
                                            );
                                            cpu_entities.push(CpuEntity::new_night_reaver(
                                                reaver_x, reaver_y,
                                            ));
                                        }
                                    }
                                }
                            }
                        }
						
                        Key::F10 => show_collision_debug = 0, // DISABLE ALL
                        Key::F11 => show_collision_debug = 1, // ENABLE COLLISION BARRIERS
                        Key::F12 => show_collision_debug = 2, // ENABLE ALL					
                        Key::T => {
                            if task_system.active {
                                task_system.open = !task_system.open;
                                if task_system.open {
                                    task_system.auto_close_timer = 7.0;
                                }								
                            }
                        }
                        Key::R => {
                            if !is_paused
                                && fighter.fighter_type == FighterType::Soldier
                                && fighter.ammo < fighter.max_ammo
                            {
                                fighter.trigger_reload(&audio_manager);
                            }
                        }
                        Key::F => {
                            // Key::F Functionality (Racer Only): Toggle Boost Mode
                            if !is_paused && fighter.fighter_type == FighterType::Racer {
                                // Check cooldown regardless of state (OnBike or OnFoot)
                                if fighter.bike_boost_toggle_cooldown > 0.0 {
                                    chatbox.add_interaction(vec![(
                                        "TOGGLE COOLDOWN ACTIVE",
                                        MessageType::Info,
                                    )]);
                                } else {
                                    // Apply cooldown on successful toggle
                                    fighter.bike_boost_toggle_cooldown = 3.0;
                                    
                                    fighter.boost = !fighter.boost;
									
                                    // Activate indicator display
                                    fighter.boost_indicator_timer = 0.75;

                                    // Play sound effect based on new mode
                                    if fighter.boost {
                                        audio_manager.play_sound_effect("slash_combo").ok();
                                    } else {
                                        audio_manager.play_sound_effect("aim").ok();
                                    }									
									
                                    let mode = if fighter.boost { "BOOST" } else { "RANGED" };
                                    chatbox.add_interaction(vec![(
                                        &format!("[SHIFT] SET: {}", mode),
                                        MessageType::Stats,
                                    )]);
                                    // If switching to BOOST while AIM is active, force AIM off
                                    if fighter.boost && shift_override_active {
                                        fighter.combat_mode = CombatMode::CloseCombat;
                                        shift_override_active = false;
                                    }
                                }
                            }
                        }					
                        _ => {}
                    }

                    if !block_system.is_stun_locked() && fighter.stun_timer <= 0.0 {
                        let (current_min_x, current_max_x, current_min_y, current_max_y) =
                            if let Some(ref area_state) = current_area {
                                let (width, height, origin_x, origin_y) = match area_state.area_type
                                {
                                    AreaType::RaptorNest => {
                                        (AREA_WIDTH, AREA_HEIGHT, AREA_ORIGIN_X, AREA_ORIGIN_Y)
                                    }
                                    AreaType::Bunker => (
                                        BUNKER_WIDTH,
                                        BUNKER_HEIGHT,
                                        BUNKER_ORIGIN_X,
                                        BUNKER_ORIGIN_Y,
                                    ),
                                };
                                (origin_x, origin_x + width, origin_y, origin_y + height)
                            } else {
                                let min_y_world = if fighter.state == RacerState::OnBike {
                                    line_y
                                } else {
                                    MIN_Y
                                };
                                (MIN_X, MAX_X, min_y_world, MAX_Y)
                            };
                        match key {
                            Key::V => {
                                if !block_system.active && !is_paused {
                                    // Clear crash state on mount
                                    if fighter.state == RacerState::OnFoot {
                                        let dx = fighter.x - sbrx_bike.x;
                                        let dy = fighter.y - sbrx_bike.y;
                                        if (dx*dx + dy*dy).sqrt() <= bike_interaction_distance {
                                            sbrx_bike.is_crashed = false;
                                        }
                                    }									
									
                                    if fighter.fighter_type == FighterType::Raptor {
                                        if current_area.is_none() {                     
                                            if true {
                                                if fighter.state == RacerState::OnFoot {
                                                    fighter.state = RacerState::OnBike;
													audio_manager.play_sound_effect("boost").ok();
                                                } else {
                                                    fighter.state = RacerState::OnFoot;
                                                }
                                                // Update textures for raptor after state change
                                                let tex_set = &raptor_textures;
                                                update_current_textures(
                                                    &fighter,
                                                    tex_set,
                                                    &mut current_idle_texture,
                                                    &mut current_fwd_texture,
                                                    &mut current_backpedal_texture,
                                                    &mut current_block_texture,
                                                    &mut current_block_break_texture,
                                                    &mut current_ranged_texture,
                                                    &mut current_ranged_marker_texture,
                                                    &mut current_ranged_blur_texture,
                                                    &mut current_rush_texture,
                                                    &mut current_strike_textures,
													shift_held,
                                                );
                                                current_racer_texture = current_idle_texture;
                                            }
                                        } else {
                                            chatbox.add_interaction(vec![(
                                                "CAN ONLY TOGGLE OUTDOORS",
                                                MessageType::Dialogue,
                                            )]);
                                        }
                                    } else {
                                        // RACER and SOLDIER logic (original logic)
                                        let dx = fighter.x - sbrx_bike.x;
                                        let dy = fighter.y - sbrx_bike.y;
                                        let d_bike = (dx * dx + dy * dy).sqrt();
                                        match fighter.state {
                                           RacerState::OnFoot => {
                                               if d_bike <= bike_interaction_distance && sbrx_bike.visible {
                                            {
                                                audio_manager.play_sound_effect("bike_start").ok();
                                                fighter.state = RacerState::OnBike;
                                                sbrx_bike.visible = false;
                                                let tex_set = match fighter.fighter_type {
                                                    FighterType::Racer => &racer_textures,
                                                    FighterType::Soldier => &soldier_textures,
                                                    _ => unreachable!(),
                                                };
                                                update_current_textures(
                                                    &fighter,
                                                    tex_set,
                                                    &mut current_idle_texture,
                                                    &mut current_fwd_texture,
                                                    &mut current_backpedal_texture,
                                                    &mut current_block_texture,
                                                    &mut current_block_break_texture,
                                                    &mut current_ranged_texture,
                                                    &mut current_ranged_marker_texture,
                                                    &mut current_ranged_blur_texture,
                                                    &mut current_rush_texture,
                                                    &mut current_strike_textures,
													shift_held,
                                                );
                                                current_racer_texture = current_idle_texture;
                                            }
										}
                                            }																																	
                                            RacerState::OnBike => {
                                                fighter.state = RacerState::OnFoot;
                                                sbrx_bike.respawn(fighter.x, fighter.y);
                                                let tex_set = match fighter.fighter_type {
                                                    FighterType::Racer => &racer_textures,
                                                    FighterType::Soldier => &soldier_textures,
                                                    _ => unreachable!(),
                                                };
                                                update_current_textures(
                                                    &fighter,
                                                    tex_set,
                                                    &mut current_idle_texture,
                                                    &mut current_fwd_texture,
                                                    &mut current_backpedal_texture,
                                                    &mut current_block_texture,
                                                    &mut current_block_break_texture,
                                                    &mut current_ranged_texture,
                                                    &mut current_ranged_marker_texture,
                                                    &mut current_ranged_blur_texture,
                                                    &mut current_rush_texture,
                                                    &mut current_strike_textures,
													shift_held,
                                                );
                                                current_racer_texture = current_idle_texture;
                                            }
                                        }
                                    }
                                }
                            }
                            Key::W => {
                                key_w_pressed = true;
                                if fighter.state == RacerState::OnFoot
                                    && !block_system.active
                                    && !rush_active
                                {
                                    if !is_paused {
                                        fighter.y = (fighter.y - 10.0).max(current_min_y);
                                    }
                                    if !movement_active || movement_timer < movement_buffer_duration
                                    {
                                        current_racer_texture = current_fwd_texture;
                                        movement_active = true;
                                        movement_timer = 0.25;
                                        current_movement_direction = MovementDirection::Forward;
                                        backpedal_active = false;
                                    }
                                }
                            }
                            Key::S => {
                                key_s_pressed = true;
                                if fighter.state == RacerState::OnFoot
                                    && !block_system.active
                                    && !rush_active
                                {
                                    if !is_paused {
                                        fighter.y = (fighter.y + 10.0).min(current_max_y);
                                    }
                                    if !movement_active || movement_timer < movement_buffer_duration
                                    {
                                        current_racer_texture = current_fwd_texture;
                                        movement_active = true;
                                        movement_timer = 0.25;
                                        current_movement_direction = MovementDirection::Forward;
                                        backpedal_active = false;
                                    }
                                }
                            }
                            Key::A => {
                                key_a_pressed = true;
                                if fighter.state == RacerState::OnFoot
                                    && !block_system.active
                                    && !rush_active
                                {
                                    if !is_paused {
                                        fighter.x = (fighter.x - 10.0).max(current_min_x);
                                    }
                                    let (wmx, _) = screen_to_world(&camera, mouse_x, mouse_y);
                                    let m_to_m = wmx < fighter.x;
                                    let n_dir = if m_to_m {
                                        MovementDirection::Forward
                                    } else {
                                        MovementDirection::Backward
                                    };
                                    if current_movement_direction != n_dir
                                        || (!movement_active && !backpedal_active)
                                        || (movement_active
                                            && movement_timer < movement_buffer_duration)
                                        || (backpedal_active
                                            && backpedal_timer < movement_buffer_duration)
                                    {
                                        current_movement_direction = n_dir;
                                        if m_to_m {
                                            current_racer_texture = current_fwd_texture;
                                            movement_active = true;
                                            movement_timer = 0.25;
                                            backpedal_active = false;
                                        } else {
                                            current_racer_texture = current_backpedal_texture;
                                            backpedal_active = true;
                                            backpedal_timer = 0.25;
                                            movement_active = false;
                                        }
                                    }
                                }
                            }
                            Key::D => {
                                key_d_pressed = true;
                                if fighter.state == RacerState::OnFoot
                                    && !block_system.active
                                    && !rush_active
                                {
                                    if !is_paused {
                                        fighter.x = (fighter.x + 10.0).min(current_max_x);
                                    }
                                    let (wmx, _) = screen_to_world(&camera, mouse_x, mouse_y);
                                    let m_to_m = wmx > fighter.x;
                                    let n_dir = if m_to_m {
                                        MovementDirection::Forward
                                    } else {
                                        MovementDirection::Backward
                                    };
                                    if current_movement_direction != n_dir
                                        || (!movement_active && !backpedal_active)
                                        || (movement_active
                                            && movement_timer < movement_buffer_duration)
                                        || (backpedal_active
                                            && backpedal_timer < movement_buffer_duration)
                                    {
                                        current_movement_direction = n_dir;
                                        if m_to_m {
                                            current_racer_texture = current_fwd_texture;
                                            movement_active = true;
                                            movement_timer = 0.25;
                                            backpedal_active = false;
                                        } else {
                                            current_racer_texture = current_backpedal_texture;
                                            backpedal_active = true;
                                            backpedal_timer = 0.25;
                                            movement_active = false;
                                        }
                                    }
                                }
                            }
                            Key::Space => {
                               // KINETIC_RUSH: [RMB] + [SPACEBAR] with kinetic_intake
                                if rush_cooldown <= 0.0
                                    && block_system.rmb_held
                                    && block_system.kinetic_intake_count > 0
                                    && !block_system.block_fatigue
                                    && !block_system.block_broken
                                {
                                    let intake_count = block_system.kinetic_intake_count;
                                    let effectiveness_multiplier = if intake_count >= 1 && intake_count <= 20 {
                                        KINETIC_STRIKE_MULTIPLIERS[intake_count as usize]
                                    } else {
                                        1.0
                                   };
 
                                    // Apply combat action slowdown when on bike
                                    if fighter.state == RacerState::OnBike {
                                        fighter.combat_action_slowdown_timer = 0.25;
                                    }
 
                                    audio_manager.play_sound_effect("rush").ok();
									audio_manager.play_sound_effect("death").ok();
                                    let (wmx, wmy) = screen_to_world(&camera, mouse_x, mouse_y);
                                    let dx = wmx - fighter.x;
                                    let dy = wmy - fighter.y;
                                    let dist = (dx * dx + dy * dy).sqrt();
 
                                    if dist > 0.0 {
                                        let ix = fighter.x;
                                        let iy = fighter.y;
                                        let ndx = dx / dist;
                                        let ndy = dy / dist;
 
                                        // KINETIC_RUSH distance scales with kinetic_intake
                                        let base_rush_distance = crate::config::movement::RUSH_DISTANCE;
                                        let fighter_multiplier = match fighter.fighter_type {
                                            FighterType::Racer => 1.0,
                                            FighterType::Raptor => 0.85,
                                            FighterType::Soldier => 0.65,
                                        };
                                        let rush_distance = base_rush_distance * fighter_multiplier * effectiveness_multiplier * KINETIC_RUSH_BASE_DISTANCE_MULTIPLIER;
 
                                        let rex = (ix + ndx * rush_distance).clamp(current_min_x, current_max_x);
                                        let rey = (iy + ndy * rush_distance).clamp(current_min_y, current_max_y);
 
                                        if !is_paused {
                                            let cex = ix + ndx * (rush_distance * 1.5);
                                            let cey = iy + ndy * (rush_distance * 1.5);
 
                                            // Grant immunity during kinetic rush
                                            fighter.invincible_timer = KINETIC_STRIKE_DAMAGE_IMMUNITY_DURATION;
											
                                            // Raptor Reward: 1.5s Invincibility on Kinetic Rush
                                            if fighter.fighter_type == FighterType::Raptor {
                                                fighter.invincible_timer = 1.5;
                                                chatbox.add_interaction(vec![
                                                    ("ATOMIC-STATE", MessageType::Warning),
                                                ]);
                                            }											
 
                                            if CPU_ENABLED {
                                                for cpu in cpu_entities.iter_mut() {
																						if check_line_collision(ix, iy, cex, cey, cpu.x, cpu.y) {
														let mut rush_damage = fighter.melee_damage; // * effectiveness_multiplier;
														// 25% damage boost while ATOMIC-STATE is active
														if fighter.invincible_timer > 1.0 {
															rush_damage *= 1.25;
														}
                                                        cpu.current_hp -= rush_damage;
 
                                                        damage_texts.push(DamageText {
                                                            text: format!("{:.0}", rush_damage),
                                                            x: cpu.x,
                                                            y: cpu.y - 50.0,
                                                            color: if fighter.invincible_timer > 1.0 { [0.7, 1.0, 0.0, 1.0] } else { [1.0, 1.0, 0.0, 1.0] },
                                                            lifetime: 0.25,
                                                        });
 
                                                        use crate::entities::cpu_entity::BleedEffect;
                                                        cpu.bleed_effect = Some(BleedEffect::new(50.0 * effectiveness_multiplier));
                                                        damage_texts.push(DamageText {
                                                            text: "BLEED".to_string(),
                                                            x: cpu.x,
                                                            y: cpu.y - 70.0,
                                                            color: [1.0, 0.0, 0.0, 1.0],
                                                            lifetime: 0.5,
                                                        });
 
                                                        if !cpu.is_dead() {
                                                            cpu.apply_knockback(ix, iy, 400.0 * effectiveness_multiplier);
                                                        }
                                                    }
                                                }
                                            }
                                            fighter.x = rex;
                                            fighter.y = rey;
                                        }
 
                                        current_racer_texture = current_rush_texture;
                                        rush_active = true;
                                        rush_timer = rush_duration;
                                        rush_cooldown = if fighter.state == RacerState::OnBike { 0.75 } else { 0.5 };
 
                                        let slx = fighter.x + ndx * (rush_distance * 0.5);
                                        let sly = fighter.y + ndy * (rush_distance * 0.5);
                                        strike.trigger(slx, sly);
                                        let original_angle = (-dy).atan2(dx).to_degrees();
                                        strike.angle = original_angle + 90.0 + safe_gen_range(-10.0, 10.0, "Strike angle variation");
                                        strike.timer = 0.2;
                                        movement_active = false;
                                        backpedal_active = false;
                                        current_movement_direction = MovementDirection::None;
 
                                        // Apply KineticStrikeEffect visual (same as kinetic_strike)
                                        let texture_index = if intake_count <= 10 {
                                            0 // Green
                                        } else if intake_count <= 17 {
                                            1 // Yellow
                                        } else {
                                            2 // Red
                                        };
                                        active_kinetic_strike_effects.push(KineticStrikeEffectInstance {
                                            //x: fighter.x, // end point
                                            //y: fighter.y, // end point
                                            x: ix,  // Starting position
                                            y: iy,  // Starting position											
                                            lifetime: 0.25,
                                            max_lifetime: 0.25,
                                            texture_index,
                                        });
										
                                        // Add speed lines from start to end point
                                        kinetic_rush_lines.push(KineticRushLine {
                                            start_x: ix,
                                            start_y: iy,
                                            end_x: fighter.x,
                                            end_y: fighter.y,
                                            lifetime: 0.25,
                                            max_lifetime: 0.25,
                                        });										
 
                                        // Consume blocks and trigger fatigue (same as kinetic_strike)
                                        block_system.block_count = 0;
                                        block_system.block_count_float = 0.0;
                                        block_system.block_fatigue = true;
                                        block_system.fatigue_timer = 1.5;
                                        block_system.kinetic_intake_count = 0;
                                        block_system.deactivate();
 
                                        combo_system.start_timer_after_kinetic_strike();
                                    }
                                } else if rush_cooldown <= 0.0 && !block_system.active {
                                    // Normal RUSH (original code)
                                    // Apply combat action slowdown when on bike
                                    if fighter.state == RacerState::OnBike {
                                        fighter.combat_action_slowdown_timer = 0.25;
                                    }

                                    if block_system.active {
                                        block_system.deactivate();
                                    }
                                    audio_manager.play_sound_effect("rush").ok();
                                    let (wmx, wmy) = screen_to_world(&camera, mouse_x, mouse_y);
                                    let dx = wmx - fighter.x;
                                    let dy = wmy - fighter.y;
                                    let dist = (dx * dx + dy * dy).sqrt();
                                    if dist > 0.0 {
                                        let ix = fighter.x;
                                        let iy = fighter.y;
                                        let ndx = dx / dist;
                                        let ndy = dy / dist;

                                        let base_rush_distance =
                                            crate::config::movement::RUSH_DISTANCE;
                                        let rush_distance = match fighter.fighter_type {
                                            FighterType::Racer => base_rush_distance,
                                            FighterType::Raptor => base_rush_distance * 0.85,
                                            FighterType::Soldier => base_rush_distance * 0.65,
                                        };

                                        let rex = (ix + ndx * rush_distance)
                                            .clamp(current_min_x, current_max_x);
                                        let rey = (iy + ndy * rush_distance)
                                            .clamp(current_min_y, current_max_y);
                                        if !is_paused {
                                            let cex = ix + ndx * (rush_distance * 1.5);
                                            let cey = iy + ndy * (rush_distance * 1.5);

                                            if CPU_ENABLED {
                                                for cpu in cpu_entities.iter_mut() {
                                                    if check_line_collision(
                                                        ix, iy, cex, cey, cpu.x, cpu.y,
                                                    ) {
														let mut rush_damage = fighter.melee_damage;
														// 25% damage boost while ATOMIC-STATE is active
														if fighter.invincible_timer > 1.0 {
															rush_damage *= 1.25;
														}												
                                                        cpu.current_hp -= rush_damage;

                                                        // Add damage text for rush attack
                                                        damage_texts.push(DamageText {
                                                            text: format!("{:.0}", rush_damage),
                                                            x: cpu.x,
                                                            y: cpu.y - 50.0,
                                                            color: if fighter.invincible_timer > 1.0 { [0.7, 1.0, 0.0, 1.0] } else { [1.0, 1.0, 0.0, 1.0] }, // Cyan in ATOMIC-STATE, else Yellow
                                                            lifetime: 0.25,
                                                        });

                                                        // Apply bleed effect
                                                        use crate::entities::cpu_entity::BleedEffect;
                                                        cpu.bleed_effect =
                                                            Some(BleedEffect::new(50.0));
                                                        damage_texts.push(DamageText {
                                                            text: "BLEED".to_string(),
                                                            x: cpu.x,
                                                            y: cpu.y - 70.0,
                                                            color: [1.0, 0.0, 0.0, 1.0], // Red
                                                            lifetime: 0.5,
                                                        });

                                                        // Death is handled in the main update loop.
                                                        // Apply knockback only if not dead.
                                                        if !cpu.is_dead() {
                                                            cpu.apply_knockback(ix, iy, 400.0);
                                                        }
                                                    }
                                                }
                                            }
                                            fighter.x = rex;
                                            fighter.y = rey;
                                        }
                                        current_racer_texture = current_rush_texture;
                                        rush_active = true;
                                        rush_timer = rush_duration;
                                        // rush cooldown
                                        rush_cooldown = if fighter.state == RacerState::OnBike {
                                            0.75
                                        } else {
                                            0.5
                                        };
                                        let slx = fighter.x + ndx * (rush_distance * 0.5);
                                        let sly = fighter.y + ndy * (rush_distance * 0.5);
                                        strike.trigger(slx, sly);
                                        let original_angle = (-dy).atan2(dx).to_degrees();
                                        strike.angle = original_angle
                                            + 90.0
                                            + safe_gen_range(-10.0, 10.0, "Strike angle variation");
                                        strike.timer = 0.2;
                                        movement_active = false;
                                        backpedal_active = false;
                                        current_movement_direction = MovementDirection::None;
                                    }
                                }
                            }
                            Key::F1 => {
                                if !is_paused
                                    && !block_system.active
                                    && !block_system.rmb_held
                                    && fighter.fighter_type != FighterType::Racer
                                    && !downed_fighters.contains(&FighterType::Racer)
                                    && fighter.state != RacerState::OnBike
                                {
                                    fighter_hp_map.insert(fighter.fighter_type, fighter.current_hp);
                                    // Store previous combat mode before switching
                                    let new_radius =
                                        fighter.switch_fighter_type(FighterType::Racer);
                                    fixed_crater.radius = new_radius;
                                    fighter.stats =
										fighter_stats_map
											.get(&FighterType::Racer)
											.copied()
											.unwrap_or(combat::stats::RACER_LVL1_STATS);
                                    fighter.max_hp = fighter.stats.defense.hp;
                                    fighter.melee_damage = fighter.stats.attack.melee_damage;
                                    fighter.ranged_damage = fighter.stats.attack.ranged_damage;
                                    fighter.run_speed = fighter.stats.speed.run_speed;
                                    fighter.current_hp = *fighter_hp_map
                                        .entry(FighterType::Racer)
                                        .or_insert(fighter.max_hp);
                                    // Reset combo stun state when switching to Racer
                                    combo_system.is_combo3_stun_disabled = false;
                                    // For RACER, switch to CLOSE COMBAT mode
                                    fighter.combat_mode = CombatMode::CloseCombat;
                                    // chatbox.add_interaction(vec![("COMBAT MODE: CLOSE COMBAT", MessageType::Info)]);
                                    shift_override_active = false;

                                    update_current_textures(
                                        &fighter,
                                        &racer_textures,
                                        &mut current_idle_texture,
                                        &mut current_fwd_texture,
                                        &mut current_backpedal_texture,
                                        &mut current_block_texture,
                                        &mut current_block_break_texture,
                                        &mut current_ranged_texture,
                                        &mut current_ranged_marker_texture,
                                        &mut current_ranged_blur_texture,
                                        &mut current_rush_texture,
                                        &mut current_strike_textures,
										shift_held,
                                    );
                                    if !block_break_animation_active {
                                        current_racer_texture = current_idle_texture;
                                    }
                                    // --- FIX: Reset and re-check level up state for the new fighter ---
                                    lvl_up_state = LvlUpState::None; // Always reset UI state on switch
                                    let points_for_new_fighter = *fighter
                                        .stat_points_to_spend
                                        .get(&fighter.fighter_type)
                                        .unwrap_or(&0);
                                    if points_for_new_fighter > 0 {
                                        lvl_up_state = LvlUpState::PendingTab {
                                            fighter_type: fighter.fighter_type,
                                        };
                                        let fighter_name = "RACER"; // We know it's Racer here
                                        chatbox.add_interaction(vec![(
                                            &format!(
                                                "!! [TAB] TO LVL UP [{}] +[{}] !!",
                                                fighter_name, points_for_new_fighter
                                            ),
                                            MessageType::Dialogue,
                                        )]);
                                    }
                                    // --- END FIX ---
                                }
                            }
                            Key::F2 => {
                                if !is_paused
                                    && soldier_has_joined
                                    && !block_system.active
                                    && !block_system.rmb_held
                                    && fighter.fighter_type != FighterType::Soldier
                                    && !downed_fighters.contains(&FighterType::Soldier)
                                    && fighter.state != RacerState::OnBike
                                {
                                    fighter_hp_map.insert(fighter.fighter_type, fighter.current_hp);
                                    // Store previous combat mode before switching
                                    let new_radius =
                                        fighter.switch_fighter_type(FighterType::Soldier);
                                    fixed_crater.radius = new_radius;
                                    fighter.stats =
										fighter_stats_map
											.get(&FighterType::Soldier)
											.copied()
											.unwrap_or(combat::stats::SOLDIER_LVL1_STATS);
                                    fighter.max_hp = fighter.stats.defense.hp;
                                    fighter.melee_damage = fighter.stats.attack.melee_damage;
                                    fighter.ranged_damage = fighter.stats.attack.ranged_damage;
                                    fighter.run_speed = fighter.stats.speed.run_speed;
                                    fighter.current_hp = *fighter_hp_map
                                        .entry(FighterType::Soldier)
                                        .or_insert(fighter.max_hp);
                                    // Reset combo stun state when switching to Soldier
                                    combo_system.is_combo3_stun_disabled = false;
                                    // For SOLDIER, switch to CLOSE COMBAT mode
                                    fighter.combat_mode = CombatMode::CloseCombat;
                                    // chatbox.add_interaction(vec![("COMBAT MODE: CLOSE COMBAT", MessageType::Info)]);
                                    shift_override_active = false;

                                    update_current_textures(
                                        &fighter,
                                        &soldier_textures,
                                        &mut current_idle_texture,
                                        &mut current_fwd_texture,
                                        &mut current_backpedal_texture,
                                        &mut current_block_texture,
                                        &mut current_block_break_texture,
                                        &mut current_ranged_texture,
                                        &mut current_ranged_marker_texture,
                                        &mut current_ranged_blur_texture,
                                        &mut current_rush_texture,
                                        &mut current_strike_textures,
										shift_held,
                                    );
                                    if !block_break_animation_active {
                                        current_racer_texture = current_idle_texture;
                                    }

                                    // --- FIX: Reset and re-check level up state for the new fighter ---
                                    lvl_up_state = LvlUpState::None; // Always reset UI state on switch
                                    let points_for_new_fighter = *fighter
                                        .stat_points_to_spend
                                        .get(&fighter.fighter_type)
                                        .unwrap_or(&0);
                                    if points_for_new_fighter > 0 {
                                        lvl_up_state = LvlUpState::PendingTab {
                                            fighter_type: fighter.fighter_type,
                                        };
                                        let fighter_name = "SOLDIER"; // We know it's Soldier here
                                        chatbox.add_interaction(vec![(
                                            &format!(
                                                "!! [TAB] TO LVL UP [{}] +[{}] !!",
                                                fighter_name, points_for_new_fighter
                                            ),
                                            MessageType::Dialogue,
                                        )]);
                                    }
                                    // --- END FIX ---
                                }
                            }
                            Key::F3 => {
                                if !is_paused
                                    && raptor_has_joined
                                    && !block_system.active
                                    && !block_system.rmb_held
                                    && fighter.fighter_type != FighterType::Raptor
                                    && !downed_fighters.contains(&FighterType::Raptor)
                                    && fighter.state != RacerState::OnBike
                                {
                                    fighter_hp_map.insert(fighter.fighter_type, fighter.current_hp);
                                    let new_radius =
                                        fighter.switch_fighter_type(FighterType::Raptor);
                                    fixed_crater.radius = new_radius;
                                    fighter.stats =
										fighter_stats_map
											.get(&FighterType::Raptor)
											.copied()
											.unwrap_or(combat::stats::RAPTOR_LVL1_STATS);
                                    fighter.max_hp = fighter.stats.defense.hp;
                                    fighter.melee_damage = fighter.stats.attack.melee_damage;
                                    fighter.ranged_damage = fighter.stats.attack.ranged_damage;
                                    fighter.run_speed = fighter.stats.speed.run_speed;
                                    fighter.current_hp = *fighter_hp_map
                                        .entry(FighterType::Raptor)
                                        .or_insert(fighter.max_hp);
                                    // Reset combo stun state when switching to raptor
                                    combo_system.is_combo3_stun_disabled = false;

                                    // Force raptor to use Close Combat mode
                                    fighter.combat_mode = CombatMode::CloseCombat;
                                    // chatbox.add_interaction(vec![("raptor COMBAT MODE: CLOSE COMBAT [LOCKED]", MessageType::Info)]);
                                    shift_override_active = false;

                                    update_current_textures(
                                        &fighter,
                                        &raptor_textures,
                                        &mut current_idle_texture,
                                        &mut current_fwd_texture,
                                        &mut current_backpedal_texture,
                                        &mut current_block_texture,
                                        &mut current_block_break_texture,
                                        &mut current_ranged_texture,
                                        &mut current_ranged_marker_texture,
                                        &mut current_ranged_blur_texture,
                                        &mut current_rush_texture,
                                        &mut current_strike_textures,
										shift_held,
                                    );
                                    if !block_break_animation_active {
                                        current_racer_texture = current_idle_texture;
                                    }
                                    // --- FIX: Reset and re-check level up state for the new fighter ---
                                    lvl_up_state = LvlUpState::None; // Always reset UI state on switch
                                    let points_for_new_fighter = *fighter
                                        .stat_points_to_spend
                                        .get(&fighter.fighter_type)
                                        .unwrap_or(&0);
                                    if points_for_new_fighter > 0 {
                                        lvl_up_state = LvlUpState::PendingTab {
                                            fighter_type: fighter.fighter_type,
                                        };
                                        let fighter_name = "RAPTOR"; 
                                        chatbox.add_interaction(vec![(
                                            &format!(
                                                "!! [TAB] TO LVL UP [{}] +[{}] !!",
                                                fighter_name, points_for_new_fighter
                                            ),
                                            MessageType::Dialogue,
                                        )]);
                                    }
                                    // --- END FIX ---
                                }
                            }
                            Key::E => {
                                // If waiting for bunker entry choice, pressing E again cancels it.
                                if bunker_entry_choice == BunkerEntryChoice::AwaitingInput {
                                    bunker_entry_choice = BunkerEntryChoice::None;
                                } else if !is_paused && show_survivor_interaction_prompt {
                                    if let Some(index) = nearby_survivor_index {
                                        if let Some(field_survivors) =
                                            survivors.get_mut(&sbrx_map_system.current_field_id)
                                        {
                                            if let Some(survivor) = field_survivors.get_mut(index) {
                                                if !survivor.is_rescued {
                                                    survivor.is_rescued = true;

                                                    match task_system.survivors_found {
                                                        0 => {
                                                            // First survivor
                                                            chatbox.add_interaction(vec![
															("-SURVIVOR-", MessageType::Info), 
															("THE ROCKETBAY IS DONE FOR. HURRY, TAKE THIS 
															FIGHTERJET PASSKEY. IT WILL GIVE YOU 
															ACCESS TO THE ONE ON THE HANGAR ROOF. 
															GET ANY SURVIRORS OVER TO FORT SILO.", MessageType::Dialogue),
															
															("SURVIVOR DIES", MessageType::Notification), 
															
															("RECEIVED:", MessageType::Info), 
															("[FIGHTERJET_PASSKEY]", MessageType::Dialogue),
															]);
                                                        }
                                                        //    9 => {
                                                        // 10th survivor (0-indexed)
                                                        //        chatbox.add_interaction(vec![("MessageType::Notification)]);
                                                        //    }
                                                        _ => {
                                                            // Survivors 2-9 (indices 1-8)
                                                            chatbox.add_interaction(vec![
																("-SURVIVOR-", MessageType::Info), 
																("thanks..", MessageType::Dialogue), 
                                                                (
                                                                    "SURVIVOR RETREATS TO THE FIGHTERJET.",
                                                                    MessageType::Notification,
                                                                ),
                                                            ]);
                                                        }
                                                    }

                                                    task_system.survivors_found += 1;
                                                    if task_system.survivors_found >= 10 {
                                                        chatbox.add_interaction(vec![
														("-SURVIVOR-", MessageType::Info), 
														("LOOK OUT!", MessageType::Dialogue), 
														("SURVIVOR GETS INCINERATED BY ELECTRICITY", MessageType::Notification), 
														("A MALEVOLENT PRESENCE APPEARS", MessageType::Notification)
														]);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else if !is_paused && show_grand_commander_prompt {
                                    show_grand_commander_prompt = false;
                                    grand_commander_dialogue_triggered = true;
                                    task_system.mark_grand_commander_spoken_to();

                                    // Revive all downed fighters
                                    if !downed_fighters.is_empty() {
                                        //println!("[DIALOGUE REVIVAL] Reviving all downed fighters for Grand Commander interaction.");
                                        let fighters_to_revive = downed_fighters.clone();
                                        downed_fighters.clear();
                                        revival_kill_score = 0;

                                        let mut revival_messages = Vec::new();
                                        for fighter_to_revive in fighters_to_revive {
                                            let max_hp = match fighter_to_revive {
                                                FighterType::Racer => RACER_LVL1_STATS.defense.hp,
                                                FighterType::Soldier => {
                                                    SOLDIER_LVL1_STATS.defense.hp
                                                }
                                                FighterType::Raptor => RAPTOR_LVL1_STATS.defense.hp,
                                            };
                                            // Revive with 100% health for this major story event
                                            fighter_hp_map.insert(fighter_to_revive, max_hp);

                                            let message = format!(
                                                "{} IS BACK IN THE FIGHT!",
                                                match fighter_to_revive {
                                                    FighterType::Racer => "RACER",
                                                    FighterType::Soldier => "SOLDIER",
                                                    FighterType::Raptor => "RAPTOR",
                                                }
                                            );
                                            revival_messages.push(message);
                                        }
                                        for msg in revival_messages {
                                            chatbox.add_interaction(vec![(
                                                &msg,
                                                MessageType::Notification,
                                            )]);
                                        }
                                    }

                                    chatbox.add_interaction(vec![
										("-GRAND COMMANDER-", MessageType::Info), 
										("SOLDIER, YOU AND THESE FIGHTERS \nMADE IT JUST IN TIME.", MessageType::Dialogue),

										("-SOLDIER-", MessageType::Info), 
										("WHAT HAPPENED HERE?", MessageType::Dialogue),

										("-GRAND COMMANDER-", MessageType::Info),  
										("WE SENT UP A COMMUNICATION SIGNAL TO FIND 
										ALIEN LIFE. NOW WE'RE BEING EXTERMINATED 
										BY THEM.", MessageType::Dialogue),
																	   
										("-RACER-", MessageType::Info), 
										("WHAT CAN WE DO?", MessageType::Dialogue),

										("-GRAND COMMANDER-", MessageType::Info),  
										("LIBERATE THE SOUTHEAST MISSILE RANGE. 
										YOU'LL NEED TO FUEL UP THE FIGHTERJET 
										FOR SUCH A DISTANCE. UNFORTUNATELY, OUR 
										FUEL TANKS HAVE ALL BEEN DESTROYED.", MessageType::Dialogue),

										("-SOLDIER-", MessageType::Info), 
										("I SAW A FUEL STATION AT THE SABERCROSS 
										TRACK I NEARLY DIED ON. THAT WOULD DO 
										THE JOB.", MessageType::Dialogue),


										("-GRAND COMMANDER-", MessageType::Info), 
										("GO AT ONCE. I WILL SEND YOU THE MISSILE  
										RANGE COORDINATES ONCE YOU'RE THERE.", MessageType::Dialogue),

										("-SOLDIER-", MessageType::Info), 
										("UNDERSTOOD. WE ALSO BROUGHT SURVIVORS FROM
										THE ROCKETBAY.", MessageType::Dialogue),	

										("-GRAND COMMANDER-", MessageType::Info), 
										("NICE WORK. ONCE WE GET THIS PLACE BACK
										IN ORDER, I'LL SEND OUT REINFORCEMENTS.", MessageType::Dialogue),											

										("-RAPTOR-", MessageType::Info), 
										("GRRR.", MessageType::Dialogue),																												 							  
									]);
								   
                                } else if !is_paused && show_fort_silo_survivor_prompt {
                                    fort_silo_survivor.interaction_triggered = true;
                                    show_fort_silo_survivor_prompt = false; // Hide prompt immediately

                                    // --- REVIVAL LOGIC ---
                                    if !downed_fighters.is_empty() {
                                        //println!("[DIALOGUE REVIVAL] Reviving all downed fighters for Fort Silo survivor interaction.");
                                        let fighters_to_revive = downed_fighters.clone();
                                        downed_fighters.clear();
                                        revival_kill_score = 0;

                                        let mut revival_messages = Vec::new();
                                        for fighter_to_revive in fighters_to_revive {
                                            let max_hp = match fighter_to_revive {
                                                FighterType::Racer => RACER_LVL1_STATS.defense.hp,
                                                FighterType::Soldier => {
                                                    SOLDIER_LVL1_STATS.defense.hp
                                                }
                                                FighterType::Raptor => RAPTOR_LVL1_STATS.defense.hp,
                                            };
                                            // Revive with 100% health for this story event
                                            fighter_hp_map.insert(fighter_to_revive, max_hp);

                                            let message = format!(
                                                "{} IS BACK IN THE FIGHT!",
                                                match fighter_to_revive {
                                                    FighterType::Racer => "RACER",
                                                    FighterType::Soldier => "SOLDIER",
                                                    FighterType::Raptor => "RAPTOR",
                                                }
                                            );
                                            revival_messages.push(message);
                                        }
                                        for msg in revival_messages {
                                            chatbox.add_interaction(vec![(
                                                &msg,
                                                MessageType::Notification,
                                            )]);
                                        }
                                    }
                                    // --- END REVIVAL LOGIC ---

                                    chatbox.add_interaction(vec![
										("-SURVIVOR-", MessageType::Info), 
										("THE FIELD BARRIER DEACTIVATED AFTER YOU
										SHOT DOWN THAT FLYING SUACER. WE'VE BEEN 
										TRAPPED HERE. IF YOU'RE LOOKING FOR THE
										THE GRAND COMMANDER, CHECK THE BUNKER.", MessageType::Dialogue), 
										("SURVIVOR DIES", MessageType::Notification),
									]);
									
                                } else if !is_paused && show_raptor_interaction_prompt {
                                    show_raptor_interaction_prompt = false;

                                    // TASK: GROUP REVIVAL DIALOGUE POINT
                                    if !downed_fighters.is_empty() {
                                        //println!("[DIALOGUE REVIVAL] Reviving all downed fighters for raptor interaction.");
                                        let fighters_to_revive = downed_fighters.clone();
                                        downed_fighters.clear(); // Clear the list
                                        revival_kill_score = 0; // Reset counter

                                        let mut revival_messages = Vec::new();

                                        for fighter_to_revive in fighters_to_revive {
                                            let max_hp = match fighter_to_revive {
                                                FighterType::Racer => RACER_LVL1_STATS.defense.hp,
                                                FighterType::Soldier => {
                                                    SOLDIER_LVL1_STATS.defense.hp
                                                }
                                                FighterType::Raptor => RAPTOR_LVL1_STATS.defense.hp,
                                            };
                                            // Revive with 25% health to be consistent with kill-based revival
                                            let revived_hp = max_hp * 0.25;
                                            fighter_hp_map.insert(fighter_to_revive, revived_hp);

                                            let message = format!(
                                                "{} IS BACK IN THE FIGHT!",
                                                match fighter_to_revive {
                                                    FighterType::Racer => "RACER",
                                                    FighterType::Soldier => "SOLDIER",
                                                    FighterType::Raptor => "RAPTOR",
                                                }
                                            );
                                            revival_messages.push(message);
                                        }

                                        // Now add all messages
                                        for msg in revival_messages {
                                            chatbox.add_interaction(vec![(
                                                &msg,
                                                MessageType::Notification,
                                            )]);
                                        }
                                    }

                                    raptor_has_joined = true;
                                    fighter_hp_map
                                        .entry(FighterType::Raptor)
                                        .or_insert(RAPTOR_LVL1_STATS.defense.hp);
                                    show_raptor_in_nest_graphic = false; // Hide raptor graphic on rescue
                                    raptor_is_trapped_in_nest = false;
                                    task_system.populate_taskbar3();
                                    chatbox.add_interaction(vec![
										("THE HURT RAPTOR LIFTS ITS HEAD AND 
										LOOKS AT YOU.", MessageType::Notification),
										
										("-RAPTOR-", MessageType::Info), 
										("Grh..", MessageType::Dialogue),										
										
										("-SOLDIER-", MessageType::Info), 
										("LOOKS LIKE THIS ONE STRAYED FROM IT'S PACK.
										TOO BAD WE COULDN'T SAVE THE OTHERS.", MessageType::Dialogue),
										
										("THE RAPTOR GETS UP AND STARTS TO FOLLOW YOU.", MessageType::Notification),										
										
										("-RACER-", MessageType::Info),  
										("I THINK IT WANTS TO JOIN US.", MessageType::Dialogue),
										
                                        ("-SOLDIER-", MessageType::Info),  
										("GOOD. BASECAMP IS JUST NORTHWEST FROM HERE.
										LET'S START HEADING THAT WAY.", MessageType::Dialogue),
										
                                        ("RAPTOR HAS JOINED THE GROUP. KEY F3 TO SWITCH", MessageType::Notification),
										("RAPTOR HAS JOINED THE GROUP. KEY F3 TO SWITCH", MessageType::Warning),
                                    ]);
                                } else if !is_paused && show_info_post_prompt {
                                    task_system.populate_taskbar1();
									
                                    chatbox.add_interaction(vec![
										("DUE TO THE HOSTILE WILDLIFE, THE RACE 
										IS POSTPONED. ANY RACERS THAT SHOW UP 
										BEFORE ARE TO CLEAR THE TRACK OF ALL 
										THREATS. THANKS!", MessageType::Info)
									]);
									
                                    racetrack_info_post_interacted = true;
                                } else if !is_paused && show_finale_info_post_prompt {
                                    chatbox.add_interaction(vec![
									("THIS ENDS THE COMBAT DEMO. 
									FUTURE UPDATES: ", MessageType::Info),
									("RACING MODE 
									ITEM DROPS 
									INVENTORY 
									SKILLS 
									PET SYSTEM
									VENDORS", MessageType::Dialogue),
									
									("PROCEED TO THE STARTING LINE TO TRIGGER", MessageType::Info),
									("ARENA MODE", MessageType::Dialogue),
									]);
                                    finale_info_post_interacted = true;
                                } else if !is_paused && show_racetrack_soldier_prompt {
                                    racetrack_soldier_dialogue_triggered = true;
                                    chatbox.add_interaction(vec![
										("-SOLDIER-", MessageType::Info), 
										("IT'LL TAKE A WHILE FOR THE FIGHTERJET TO 
										FUEL UP. GO ATTEND THE RACE WHILE WE WAIT.", MessageType::Dialogue),

										("THE RAPTOR SNIFFS AROUND.", MessageType::Notification), 
										
										("-RACER-", MessageType::Info),  
										("I'LL WIN US SOME PRIZE MONEY.", MessageType::Dialogue),
									]);
                                } else if !is_paused
                                    && show_soldier_interaction_prompt
                                    && !soldier_has_joined
                                {
                                    show_soldier_interaction_prompt = false;
                                    soldier_has_joined = true;
                                    fighter_hp_map
                                        .entry(FighterType::Soldier)
                                        .or_insert(SOLDIER_LVL1_STATS.defense.hp);
                                    soldier_visible = false;
                                    task_system.populate_taskbar2();
                                    chatbox.add_interaction(vec![
                                        ("-SOLDIER-", MessageType::Info), 
										("RACER, MY SQUAD WAS AMBUSHED BY RAPTORS. 
										SOME GOT DRAGGED AWAY TO A CAVE IN 
										THE FIELD JUST TO THE RIGHT. HELP ME UP 
										AND LET'S GO SAVE THEM!", MessageType::Dialogue),
										
                                        ("SOLDIER HAS JOINED THE GROUP", MessageType::Notification),
                                        ("KEY F1 AND F2 TO SWITCH FIGHTERS", MessageType::Notification),
										("SOLDIER HAS JOINED THE GROUP KEY F1 AND F2 TO SWITCH FIGHTERS", MessageType::Warning),
                                    ]);
                                } else if let Some(area_state) = &current_area.clone() {
                                    // Clone to avoid borrow issues
                                    if area_state.is_player_at_world_exit(fighter.x, fighter.y)
                                        && !wave_manager.is_active()
                                    {
                                        // Capture the area type before we drop the borrow
                                        let exiting_area_type = area_state.area_type;

                                        let exit_field = match exiting_area_type {
                                            AreaType::RaptorNest => {
                                                println!("Exiting Raptor Nest area.");
                                                SbrxFieldId(1, 0)
                                            }
                                            AreaType::Bunker => {
                                                println!(
                                                    "Exiting Bunker area. Resetting wave system."
                                                );
                                                // --- FIX: Reset wave state on exit ---
                                                wave_manager.reset();
                                                completed_bunker_waves.clear();
												explored_bunker_floors.clear();
                                                // --- END FIX ---
                                                SbrxFieldId(-25, 25)
                                            }
                                        };
                                        current_area = None;
                                        cpu_entities.clear();
                                        sbrx_map_system.current_field_id = exit_field;
                                        // Return player to entrance position
                                        fighter.x = area_entrance_x;
                                        fighter.y = area_entrance_y;
                                        camera.x = fighter.x;
                                        camera.y = fighter.y;
                                        show_raptor_in_nest_graphic = false;

                                        // Respawn appropriate enemies for the field when exiting area
                                        if exiting_area_type == AreaType::RaptorNest && CPU_ENABLED
                                        {
                                            if !task_system
                                                .is_task_complete("CLEAR RAPTOR NEST: FIELD[X1 Y0]")
                                            {
                                                // Nest not cleared yet - respawn the 2 guard raptors
                                                println!("Respawning guard raptors in field x1 y0 (nest not cleared)");
                                                if let Some(nest) = raptor_nests.first() {
                                                    if cpu_entities.len() < 10 {
                                                        cpu_entities.push(CpuEntity::new_raptor(
                                                            nest.x - 75.0,
                                                            nest.y,
                                                        ));
                                                    }
                                                    if cpu_entities.len() < 10 {
                                                        cpu_entities.push(CpuEntity::new_raptor(
                                                            nest.x + 75.0,
                                                            nest.y,
                                                        ));
                                                    }
                                                }

                                                // Respawn standard field enemies
                                                println!(
                                                    "Respawning standard enemies in field x1 y0"
                                                );

                                                // Spawn Giant Mantis
                                                if cpu_entities.len() < 10 {
                                                    cpu_entities
                                                        .push(CpuEntity::new_giant_mantis(line_y));
                                                }

                                                // Spawn 3 Rattlesnakes (standard for non-origin fields)
                                                for _ in 0..3 {
                                                    if cpu_entities.len() < 10 {
                                                        cpu_entities.push(
                                                            CpuEntity::new_rattlesnake(line_y),
                                                        );
                                                    }
                                                }

                                                // Spawn Giant Rattlesnakes based on score
                                                let current_score = fighter.score;
                                                let mut score_tier = 5;
                                                while score_tier <= current_score {
                                                    if spawned_giant_rattlesnake_scores
                                                        .contains(&score_tier)
                                                    {
                                                        if cpu_entities.len() < 10 {
                                                            cpu_entities.push(
                                                                CpuEntity::new_giant_rattlesnake(
                                                                    line_y,
                                                                ),
                                                            );
                                                            println!("Respawned Giant Rattlesnake for score tier {}", score_tier);
                                                            break; // Only spawn one
                                                        }
                                                    }
                                                    score_tier += 5;
                                                }
                                            }
                                        }

                                        if t_rex_spawn_pending
                                            && exiting_area_type == AreaType::RaptorNest
                                        {
                                            println!("Spawning T-REX!");
                                            task_system.populate_taskbar4();
                                            if cpu_entities.len() < 10 {
                                                let t_rex_x =
                                                    safe_gen_range(MIN_X, MAX_X, "T-Rex spawn x");
                                                let t_rex_y =
                                                    safe_gen_range(MIN_Y, MAX_Y, "T-Rex spawn y");
                                                cpu_entities
                                                    .push(CpuEntity::new_t_rex(t_rex_x, t_rex_y));
                                            }
                                            t_rex_spawn_pending = false;
                                            t_rex_is_active = true;
                                            chatbox.add_interaction(vec![(
                                                "WARNING: STRONG ENCOUNTER",
                                                MessageType::Warning,
                                            )]);											
                                        }
                                        // Spawn T-Rex if already active (subsequent exits)
                                        else if t_rex_is_active
                                            && exiting_area_type == AreaType::RaptorNest
                                            && CPU_ENABLED
                                        {
                                            let t_rex_exists = cpu_entities.iter().any(|e| {
                                                e.variant == CpuVariant::TRex && !e.is_dead()
                                            });
                                            if !t_rex_exists && cpu_entities.len() < 10 {
                                                let t_rex_x = safe_gen_range(
                                                    MIN_X,
                                                    MAX_X,
                                                    "T-Rex nest exit x",
                                                );
                                                let t_rex_y = safe_gen_range(
                                                    MIN_Y,
                                                    MAX_Y,
                                                    "T-Rex nest exit y",
                                                );
                                                cpu_entities
                                                    .push(CpuEntity::new_t_rex(t_rex_x, t_rex_y));
                                                println!("Respawned T-Rex when exiting raptor nest (already active)");
                                                chatbox.add_interaction(vec![(
                                                    "WARNING: STRONG ENCOUNTER",
                                                    MessageType::Warning,
                                                )]);												
                                            }
                                        }
                                    } else if show_bunker_floor_transition_prompt
                                        && !wave_manager.is_active()
                                    {
                                        if let Some(target_floor) = target_floor_from_prompt {
                                            let current_floor = area_state.floor;
                                            let is_peaceful_mode = area_state.is_peaceful;

                                            // --- WAVE SYSTEM TRIGGER ---
                                            let mut bunker_wave_definitions: HashMap<i32, u32> =
                                                HashMap::new();
                                            bunker_wave_definitions.insert(1, 2);
                                            bunker_wave_definitions.insert(0, 2);
                                            bunker_wave_definitions.insert(-1, 2);
                                            bunker_wave_definitions.insert(-2, 3);

                                            let waves_for_target_floor = *bunker_wave_definitions
                                                .get(&target_floor)
                                                .unwrap_or(&0);

                                            // Lockdown is only active if waves exist AND the bunker hasn't been fully cleared before.
                                            let lockdown_active = waves_for_target_floor > 0
                                                && !bunker_waves_fully_completed
                                                && !is_peaceful_mode;

                                            if waves_for_target_floor > 0 && !is_peaceful_mode {
                                                wave_manager
                                                    .start_encounter(waves_for_target_floor);
                                            } else {
                                                wave_manager.reset();
                                            }
                                            // --- END WAVE TRIGGER ---

                                            cpu_entities.clear();

                                            // Create a new area state for the new floor
                                            let (new_area, _player_start_pos) = AreaState::new(
                                                AreaType::Bunker,
                                                target_floor,
                                                lockdown_active,
                                                is_peaceful_mode,
                                            );

                                            if target_floor < current_floor {
                                                // Descending
                                                // Find the ascent point in the new (lower) level's transitions
                                                if let Some(ascent_point) = new_area
                                                    .floor_transitions
                                                    .iter()
                                                    .find(|t| t.target_floor > target_floor)
                                                {
                                                    fighter.x = ascent_point.rect.x
                                                        + ascent_point.rect.width / 2.0;
                                                    fighter.y = ascent_point.rect.y + 50.0;
                                                    // Spawn just below the point
                                                }
                                            } else {
                                                // Ascending
                                                // Find the descent point in the new (higher) level's transitions
                                                if let Some(descent_point) = new_area
                                                    .floor_transitions
                                                    .iter()
                                                    .find(|t| t.target_floor < target_floor)
                                                {
                                                    fighter.x = descent_point.rect.x - 50.0; // Spawn just to the left
                                                    fighter.y = descent_point.rect.y
                                                        + descent_point.rect.height / 2.0;
                                                }
                                            }

                                            // Spawn Night Reavers on the new floor
                                            if new_area.spawn_night_reavers
                                                && CPU_ENABLED
                                                && target_floor != -3
                                                && !completed_bunker_waves.contains(&target_floor)
                                                && !is_peaceful_mode
                                            {
                                                println!(
                                                    "Spawning Night Reavers in bunker floor: {}",
                                                    target_floor
                                                );
                                                for _ in 0..4 {
                                                    if cpu_entities.len() < 10 {
                                                        let reaver_x = safe_gen_range(
                                                            BUNKER_ORIGIN_X + 50.0,
                                                            BUNKER_ORIGIN_X + BUNKER_WIDTH - 50.0,
                                                            "NightReaver x",
                                                        );
                                                        let reaver_y = safe_gen_range(
                                                            BUNKER_ORIGIN_Y + 50.0,
                                                            BUNKER_ORIGIN_Y + BUNKER_HEIGHT - 50.0,
                                                            "NightReaver y",
                                                        );
                                                        cpu_entities.push(
                                                            CpuEntity::new_night_reaver(
                                                                reaver_x, reaver_y,
                                                            ),
                                                        );
                                                    }
                                                }
                                            }

                                            // Spawn RazorFiend on floor -3
                                            if target_floor == -3
                                                && task_system.razor_fiend_defeated == 0
                                                && !is_peaceful_mode
                                            {
                                                println!("Spawning RazorFiend in bunker floor: -3");
                                                if cpu_entities.len() < 10 {
                                                    // Check entity limit
                                                    let fiend_x = safe_gen_range(
                                                        BUNKER_ORIGIN_X + 50.0,
                                                        BUNKER_ORIGIN_X + BUNKER_WIDTH - 50.0,
                                                        "RazorFiend x",
                                                    );
                                                    let fiend_y = safe_gen_range(
                                                        BUNKER_ORIGIN_Y + 50.0,
                                                        BUNKER_ORIGIN_Y + BUNKER_HEIGHT - 50.0,
                                                        "RazorFiend y",
                                                    );
                                                    cpu_entities.push(CpuEntity::new_razor_fiend(
                                                        fiend_x, fiend_y,
                                                    ));
                                                    chatbox.add_interaction(vec![(
                                                        "DANGER! DEADLY ENCOUNTER",
                                                        MessageType::Warning,
                                                    )]);													
                                                }
                                            }
                                            // Must clone here due to borrow checker
                                            current_area = Some(new_area.clone());
											
                                            // --- FOG OF WAR: Reset fog for unexplored bunker floors ---
                                            if sbrx_map_system.current_field_id == SbrxFieldId(-25, 25)
                                                && !explored_bunker_floors.contains(&target_floor)
                                            {
                                                fog_of_war.reset_field_fog(SbrxFieldId(-25, 25));
                                                explored_bunker_floors.insert(target_floor);
                                                println!(
                                                    "Fog refreshed for FLATLINE_field.x0y0 FORT SILO BUNKER[{}]",
                                                    target_floor
                                                );
                                            }											

                                            camera.x = fighter.x;
                                            camera.y = fighter.y;
                                            println!("Transitioned to bunker floor: {}. Player spawned at ({}, {})", target_floor, fighter.x, fighter.y);
                                        }
                                    }
                                } else if !is_paused && show_fighter_jet_prompt {
                                    // Check if VoidTempest is alive and blocking
                                    let void_tempest_alive = cpu_entities.iter().any(|e| {
                                        e.variant == CpuVariant::VoidTempest && !e.is_dead()
                                    });

                                    if endless_arena_mode_active {
                                        chatbox.add_interaction(vec![(
                                            "CANNOT BOARD FIGHTERJET DURING ARENA MODE",
                                            MessageType::Notification,
                                        )]);
                                    } else if void_tempest_alive {
                                        chatbox.add_interaction(vec![(
                                            "DEFEAT THE VOID TEMPEST BEFORE BOARDING",
                                            MessageType::Notification,
                                        )]);
                                    } else {
                                        println!("Boarding FIGHTERJET to FIRMAMENT...");
                                        if let Some(sink) = bike_accelerate_sound_sink.take() {
                                            sink.stop();
                                        }
                                        if let Some(sink) = bike_idle_sound_sink.take() {
                                            sink.stop();
                                        }
                                        if let Some(sink) = crickets_sound_sink.take() {
                                            sink.stop();
                                        }

                                        if task_system.mark_fighter_jet_as_boarded() {
                                            award_kill_score(
                                                &mut fighter,
                                                10,
                                                &mut chatbox,
                                                &mut lvl_up_state,
                                                "completing BOARD THE FIGHTERJET task",
                                            );
                                        }

                                        game_state = GameState::LoadingFirmament;
                                    }
                                } else if !is_paused && show_raptor_nest_prompt {
                                    if fighter.state == RacerState::OnBike {
                                        chatbox.add_interaction(vec![(
                                            "YOU MUST DISMOUNT TO ENTER",
                                            MessageType::Notification,
                                        )]);
                                    } else {
                                        println!("Key E pressed near raptor nest. Entering...");
                                        // Save entrance position before entering
                                        area_entrance_x = fighter.x;
                                        area_entrance_y = fighter.y;
                                        // Set raptor as trapped when first entering nest (if not already joined)
                                        if !raptor_has_joined {
                                            raptor_is_trapped_in_nest = true;
                                        }
                                        // Only show raptor graphic if raptor hasn't joined yet (first time rescue)
                                        show_raptor_in_nest_graphic =
                                            !raptor_has_joined && raptor_is_trapped_in_nest;
                                        let (new_area, player_start_pos) =
                                            AreaState::new(AreaType::RaptorNest, 1, false, false); // No waves in raptor nest
                                        current_area = Some(new_area);
                                        fighter.x = player_start_pos.0;
                                        fighter.y = player_start_pos.1;
                                        camera.x = fighter.x;
                                        camera.y = fighter.y;
                                        cpu_entities.clear();
                                        t_rex_spawn_pending = false;
                                        for _ in 0..12 {
                                            let raptor_x = safe_gen_range(
                                                AREA_ORIGIN_X + 50.0,
                                                AREA_ORIGIN_X + AREA_WIDTH - 50.0,
                                                "raptor x",
                                            );
                                            let raptor_y = safe_gen_range(
                                                AREA_ORIGIN_Y + 50.0,
                                                AREA_ORIGIN_Y + AREA_HEIGHT - 50.0,
                                                "raptor y",
                                            );
                                            if cpu_entities.len() < 10 {
                                                cpu_entities.push(CpuEntity::new_raptor(
                                                    raptor_x, raptor_y,
                                                ));
                                            }
                                        }
                                    }
                                } else if !is_paused && show_bunker_prompt {
                                    if task_system.is_task_complete("SPEAK TO THE GRAND COMMANDER")
                                    {
                                        bunker_entry_choice = BunkerEntryChoice::AwaitingInput;
                                    } else if fighter.state == RacerState::OnBike {
                                        chatbox.add_interaction(vec![(
                                            "YOU MUST DISMOUNT TO ENTER",
                                            MessageType::Notification,
                                        )]);
                                    } else {
                                        println!("Key E pressed near bunker. Entering...");
                                        // save entrance position before entering
                                        area_entrance_x = fighter.x;
                                        area_entrance_y = fighter.y;

                                        // --- WAVE SYSTEM TRIGGER on first entry ---
                                        let mut bunker_wave_definitions: HashMap<i32, u32> =
                                            HashMap::new();
                                        bunker_wave_definitions.insert(1, 2);
                                        bunker_wave_definitions.insert(0, 2);
                                        bunker_wave_definitions.insert(-1, 2);
                                        bunker_wave_definitions.insert(-2, 3);

                                        let waves_for_floor_1 =
                                            *bunker_wave_definitions.get(&1).unwrap_or(&0);
                                        // Lockdown is only active if waves exist AND the bunker hasn't been fully cleared before.
                                        let lockdown_active =
                                            waves_for_floor_1 > 0 && !bunker_waves_fully_completed;

                                        if waves_for_floor_1 > 0 {
                                            wave_manager.start_encounter(waves_for_floor_1);
                                        }
                                        // --- END WAVE TRIGGER ---
                                        let (new_area, player_start_pos) = AreaState::new(
                                            AreaType::Bunker,
                                            1,
                                            lockdown_active,
                                            false,
                                        );

                                        let should_spawn_reavers = new_area.spawn_night_reavers;
                                        current_area = Some(new_area);
                                        fighter.x = player_start_pos.0;
                                        fighter.y = player_start_pos.1;
                                        camera.x = fighter.x;
                                        camera.y = fighter.y;
                                        cpu_entities.clear();
                                        // Spawn Night Reavers in bunker
                                        // Only spawn if waves for floor 1 haven't been completed yet
                                        if should_spawn_reavers
                                            && CPU_ENABLED
                                            && !completed_bunker_waves.contains(&1)
                                        {
                                            println!("Spawning Night Reavers in bunker floor 1");
                                            for _ in 0..4 {
                                                if cpu_entities.len() < 10 {
                                                    let reaver_x = safe_gen_range(
                                                        BUNKER_ORIGIN_X + 50.0,
                                                        BUNKER_ORIGIN_X + BUNKER_WIDTH - 50.0,
                                                        "NightReaver x",
                                                    );
                                                    let reaver_y = safe_gen_range(
                                                        BUNKER_ORIGIN_Y + 50.0,
                                                        BUNKER_ORIGIN_Y + BUNKER_HEIGHT - 50.0,
                                                        "NightReaver y",
                                                    );
                                                    cpu_entities.push(CpuEntity::new_night_reaver(
                                                        reaver_x, reaver_y,
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // COMBAT MODES
                            /*
                                                        Key:: => { // toggle through combat modes
                                                            if fighter.fighter_type == FighterType::Raptor {
                                                                chatbox.add_interaction(vec![(
                                                                    "raptor MUST USE CLOSE COMBAT",
                                                                    MessageType::Notification,
                                                                )]);
                                                            } else {
                                                                // If shift is overriding, pressing Q should cancel it and cycle normally
                                                                if shift_override_active {
                                                                    shift_override_active = false;
                                                                    // The current mode is Ranged due to the override.
                                                                    // Q will cycle it to Balanced, which becomes the new permanent mode.
                                                                    // Releasing shift will now do nothing because shift_override_active is false.
                                                                }
                                                                fighter.combat_mode = match fighter.combat_mode {
                                                                    CombatMode::CloseCombat => {
                                                                        chatbox.add_interaction(vec![(
                                                                            "COMBAT MODE: RANGED",
                                                                            MessageType::Info,
                                                                        )]);
                                                                        CombatMode::Ranged
                                                                    }
                                                                    CombatMode::Ranged => {
                                                                        chatbox.add_interaction(vec![(
                                                                            "COMBAT MODE: BALANCED",
                                                                            MessageType::Info,
                                                                        )]);
                                                                        CombatMode::Balanced
                                                                    }
                                                                    CombatMode::Balanced => {
                                                                        chatbox.add_interaction(vec![(
                                                                            "COMBAT MODE: CLOSE COMBAT",
                                                                            MessageType::Info,
                                                                        )]);
                                                                        CombatMode::CloseCombat
                                                                    }
                                                                };
                                                            }
                                                        }
                            */
                            // COMAT MODES
                            _ => (),
                        }
                    }
                }
                if let Some(Button::Keyboard(key)) = e.release_args() {
                    match key {
                        Key::W => key_w_pressed = false,
                        Key::S => key_s_pressed = false,
                        Key::A => key_a_pressed = false,
                        Key::D => key_d_pressed = false,
                        _ => (),
                    }
                    if fighter.state == RacerState::OnFoot
                        && !key_w_pressed
                        && !key_s_pressed
                        && !key_a_pressed
                        && !key_d_pressed
                        && !block_break_animation_active
                        && !is_high_priority_animation_active(
                            rush_active,
                            strike_animation_timer,
                            block_system.active,
                            block_system.rmb_held,
                        )
                        && !movement_active
                        && !backpedal_active
                    {
                        current_racer_texture = current_idle_texture;
                        current_movement_direction = MovementDirection::None;
                    } else if fighter.state == RacerState::OnBike
                        && !key_w_pressed
                        && !key_s_pressed
                        && !key_a_pressed
                        && !key_d_pressed
                        && !block_break_animation_active
                        && !is_high_priority_animation_active(
                            rush_active,
                            strike_animation_timer,
                            block_system.active,
                            block_system.rmb_held,
                        )
                    {
                        current_racer_texture = current_idle_texture;
                        movement_active = false;
                        backpedal_active = false;
                        current_movement_direction = MovementDirection::None;
                    }
                }
            }
            GameState::DeathScreen(death_type) => {
                if let Some(args) = e.update_args() {
                    if death_screen_cooldown > 0.0 {
                        death_screen_cooldown -= args.dt;
                    }
                }

                // Render the death screen
                if let Some(_) = e.render_args() {
                    window.draw_2d(&e, |c, g, device| {
                        clear([0.0, 0.0, 0.0, 1.0], g);

                        let death_message_text = match death_type {
							DeathType::Crashed => "CRASHED",
                            DeathType::Meteorite => "OBLITERATED BY METEORITE",
                            DeathType::GiantMantis => "EATEN BY GIANT MANTIS",
                            DeathType::Rattlesnake => "BITTEN BY RATTLESNAKE",
                            DeathType::GiantRattlesnake => "SWALLOWED WHOLE BY GIANT RATTLESNAKE",
                            DeathType::BloodIdol => "TORN APART BY BLOOD IDOL",
                            DeathType::VoidTempest => "ANNIHILATED BY VOID TEMPEST",
                            DeathType::Raptor => "MAULED BY A RAPTOR",
                            DeathType::TRex => "DEVOURED BY A T-REX",
                            DeathType::FlyingSaucer => "VAPORIZED BY FLYING SAUCER",
                            DeathType::LightReaver => "ABDUCTED BY A LIGHT REAVER",
                            DeathType::NightReaver => "MUTILATED BY A NIGHT REAVER",
                            DeathType::RazorFiend => "SHREDDED BY A RAZOR FIEND",
                        };

                        let font_size = 32;
                        let red = [1.0, 0.1, 0.1, 1.0];
                        let white = [1.0, 1.0, 1.0, 1.0];

                        let line1 = "";
                        let line1_width = glyphs.width(font_size, line1).unwrap_or(0.0);
                        let y_pos = screen_height / 2.0 - 50.0;

                        text(
                            white,
                            font_size,
                            line1,
                            &mut glyphs,
                            c.transform.trans((screen_width - line1_width) / 2.0, y_pos),
                            g,
                        )
                        .ok();

                        let line2_width =
                            glyphs.width(font_size, death_message_text).unwrap_or(0.0);
                        text(
                            red,
                            font_size,
                            death_message_text,
                            &mut glyphs,
                            c.transform
                                .trans((screen_width - line2_width) / 2.0, y_pos + 50.0),
                            g,
                        )
                        .ok();

                        let line3 = "PRESS ANY KEY TO CONTINUE";
                        let line3_width = glyphs.width(font_size, line3).unwrap_or(0.0);
                        text(
                            white,
                            font_size,
                            line3,
                            &mut glyphs,
                            c.transform
                                .trans((screen_width - line3_width) / 2.0, y_pos + 150.0),
                            g,
                        )
                        .ok();

                        glyphs.factory.encoder.flush(device);
                    });
                }

                if let Some(Button::Keyboard(_)) = e.press_args() {
                    if death_screen_cooldown > 0.0 {
                        continue;
                    }
                    println!("Resuming game from last field entry point after party wipe.");
					
                    // BUG FIX: Reset lmb_held on respawn to prevent stuck melee strike
                    lmb_held = false;
                    melee_rapid_fire_timer = 0.0;
                    soldier_rapid_fire_timer = 0.0;					

                    // Reset flying saucer defeated flag if died to flying saucer
                    if death_type == DeathType::FlyingSaucer {
                        firmament_boss_defeated = false;
                        task_system.flying_saucer_defeated = false;
                        println!("Reset flying saucer defeated flag after death");
                    }
                    game_state = GameState::Playing;

                    // --- PRESERVE PROGRESS ---
                    let saved_score = fighter.score;

                    // --- BUG FIX: Preserve progression data across the reset ---
                    let saved_kill_counters = fighter.kill_counters.clone();
                    let saved_levels = fighter.levels.clone();
                    let saved_stat_points = fighter.stat_points_to_spend.clone();

                    // Use fixed spawn point for Racetrack, otherwise use last entry point
                    let spawn_point = if sbrx_map_system.current_field_id == SbrxFieldId(0, 0) {
                        RACETRACK_SPAWN_POINT
                    } else {
                        last_field_entry_point
                    };
                    fighter = Fighter::new(spawn_point.0, spawn_point.1);

                    fighter.score = saved_score;
                    fighter.kill_counters = saved_kill_counters;
                    fighter.levels = saved_levels;
                    fighter.stat_points_to_spend = saved_stat_points;

                    // Reload the Racer's leveled-up stats from the persistent map
					let racer_stats = base_fighter_stats_map
						.get(&FighterType::Racer)
						.copied()
						.unwrap_or(combat::stats::RACER_LVL1_STATS);
                    fighter.stats = racer_stats;
                    fighter.max_hp = racer_stats.defense.hp;
                    fighter.melee_damage = racer_stats.attack.melee_damage;
                    fighter.ranged_damage = racer_stats.attack.ranged_damage;
                    fighter.run_speed = racer_stats.speed.run_speed;
                    // --- END FIX ---

                    sbrx_bike.respawn(fighter.x + 100.0, fighter.y);
                    fighter.state = RacerState::OnFoot;
                    fixed_crater.radius = 125.0;

                    if endless_arena_mode_active {
                        // Deactivate SOLDIER and raptor field assets on first defeat
                        show_racetrack_soldier_raptor_assets = false;
                        
                        if !soldier_has_joined {
                            soldier_has_joined = true;
                            chatbox.add_interaction(vec![(
                                "SOLDIER REJOINS THE GROUP",
                                MessageType::Notification,
                            )]);
                        }
                        if !raptor_has_joined {
                            raptor_has_joined = true;
                            chatbox.add_interaction(vec![(
                                "RAPTOR REJOINS THE GROUP",
                                MessageType::Notification,
                            )]);
                        }
                        chatbox.add_interaction(vec![(
                            "SOLDIER AND RAPTOR REJOIN THE GROUP",
                            MessageType::Notification,
                        )]);
                    }

                    buffed_fighters.clear(); // Reset buff state on full party wipe
                    downed_fighters.clear();
//                    revival_kill_score = 0;

                    fighter_hp_map.clear();
                    fighter_hp_map.insert(
                        FighterType::Racer,
                        base_fighter_stats_map[&FighterType::Racer].defense.hp,
                    );
                    fighter_hp_map.insert(
                        FighterType::Soldier,
                        base_fighter_stats_map[&FighterType::Soldier].defense.hp,
                    );
                    fighter_hp_map.insert(
                        FighterType::Raptor,
                        base_fighter_stats_map[&FighterType::Raptor].defense.hp,
                    );
                    fighter.current_hp = fighter.max_hp;
					
                    update_current_textures(
                        &fighter,
                        &racer_textures,
                        &mut current_idle_texture,
                        &mut current_fwd_texture,
                        &mut current_backpedal_texture,
                        &mut current_block_texture,
                        &mut current_block_break_texture,
                        &mut current_ranged_texture,
                        &mut current_ranged_marker_texture,
                        &mut current_ranged_blur_texture,
                        &mut current_rush_texture,
                        &mut current_strike_textures,
						shift_held,
                    );
                    current_racer_texture = current_idle_texture;

                    fixed_crater.x = fighter.x;
                    fixed_crater.y = fighter.y;
                    camera.x = fighter.x;
                    camera.y = fighter.y;

                    // --- BUG FIX: Restore LVL_UP prompt after full party wipe ---
                    lvl_up_state = LvlUpState::None; // Reset UI state
                                                     // On full wipe, player always respawns as Racer. Check their points.
                    let points_for_racer = *fighter
                        .stat_points_to_spend
                        .get(&FighterType::Racer)
                        .unwrap_or(&0);
                    if points_for_racer > 0 {
                        lvl_up_state = LvlUpState::PendingTab {
                            fighter_type: FighterType::Racer,
                        };
                        chatbox.add_interaction(vec![(
                            &format!("!! [TAB] TO LVL UP [RACER] +[{}] !!", points_for_racer),
                            MessageType::Dialogue,
                        )]);
                    }
                    // --- END FIX ---

                    cpu_entities.clear();
                    if CPU_ENABLED {
                        if cpu_entities.len() < 10 {
                            // Add a default enemy to kickstart the spawn logic in the update loop.
                            cpu_entities.push(CpuEntity::new_giant_mantis(line_y));
                        }
                    }

                    // Respawn task-dependent entities if they should be present
                    let current_field = sbrx_map_system.current_field_id;

                    // Respawn T-Rex if it should be active
                    if task_system.is_task_complete("CLEAR RAPTOR NEST: FIELD[X1 Y0]")
                        && !task_system.is_task_complete("DEFEAT THE T-REX")
                        && current_field == SbrxFieldId(1, 0)
                        && CPU_ENABLED
                    {
                        if cpu_entities.len() < 10 {
                            let t_rex_x = safe_gen_range(MIN_X, MAX_X, "T-Rex respawn x");
                            let t_rex_y = safe_gen_range(MIN_Y, MAX_Y, "T-Rex respawn y");
                            cpu_entities.push(CpuEntity::new_t_rex(t_rex_x, t_rex_y));
                            println!("Respawned T-Rex after party wipe in field x1 y0");
                        }
                    }

                    // Respawn VoidTempest if it should be active
                    if task_system.survivors_found >= 10
                        && current_field == SbrxFieldId(-2, 5)
                        && CPU_ENABLED
                    {
                        let void_tempest_exists = cpu_entities
                            .iter()
                            .any(|e| e.variant == CpuVariant::VoidTempest);
                        if !void_tempest_exists && cpu_entities.len() < 10 {
                            let (base_hp, base_speed) = (250.0, 150.0);
                            cpu_entities
                                .push(CpuEntity::new_void_tempest(line_y, base_hp, base_speed));
                            println!("Respawned VoidTempest after party wipe");
                        }
                    }

                    // Respawn Rattlesnakes based on score
                    if CPU_ENABLED {
                        let should_spawn_rattlesnakes = if current_field == SbrxFieldId(0, 0) {
                            // In field (0,0), only spawn if score >= 3 and they were spawned before
                            fighter.score >= 3 && rattlesnakes_spawned_in_field0_score3
                        } else {
                            // In any other field, always spawn rattlesnakes
                            true
                        };

                        if should_spawn_rattlesnakes {
                            println!(
                                "Respawning 3 rattlesnakes after party wipe in field {:?}",
                                current_field
                            );
                            for _ in 0..3 {
                                if cpu_entities.len() < 10 {
                                    cpu_entities.push(CpuEntity::new_rattlesnake(line_y));
                                }
                            }
                        }
                    }

                    // Respawn BLOOD IDOL if in fog-enabled field
                    if FOG_OF_WAR_ENABLED
                        && fog_of_war.is_fog_enabled(current_field)
                        && has_blood_idol_fog_spawned_once
                        && CPU_ENABLED
                    {
                        let blood_idol_exists = cpu_entities
                            .iter()
                            .any(|e| e.variant == CpuVariant::BloodIdol);
                        if !blood_idol_exists && cpu_entities.len() < 10 {
                            let (base_hp, base_speed) = (250.0, 150.0);
                            cpu_entities
                                .push(CpuEntity::new_blood_idol(line_y, base_hp, base_speed));
                            println!("Respawned BloodIdol after party wipe in fog field");
                        }
                    }

                    // Respawn Giant Rattlesnakes based on score
                    if CPU_ENABLED {
                        let current_score = fighter.score;
                        let mut score_tier = 5;
                        while score_tier <= current_score {
                            if spawned_giant_rattlesnake_scores.contains(&score_tier) {
                                let giant_rattlesnake_exists = cpu_entities
                                    .iter()
                                    .any(|e| e.variant == CpuVariant::GiantRattlesnake);
                                if !giant_rattlesnake_exists && cpu_entities.len() < 10 {
                                    cpu_entities.push(CpuEntity::new_giant_rattlesnake(line_y));
                                    println!("Respawned Giant Rattlesnake after party wipe for score tier {}", score_tier);
                                    break; // Only spawn one
                                }
                            }
                            score_tier += 5;
                        }
                    }

                    // Respawn Raptors if in raptor nest area
                    if let Some(ref area_state) = current_area {
                        if area_state.area_type == AreaType::RaptorNest && CPU_ENABLED {
                            println!("Respawning raptors after party wipe in raptor nest");
                            for _ in 0..12 {
                                let raptor_x = safe_gen_range(
                                    AREA_ORIGIN_X + 50.0,
                                    AREA_ORIGIN_X + AREA_WIDTH - 50.0,
                                    "raptor respawn x",
                                );
                                let raptor_y = safe_gen_range(
                                    AREA_ORIGIN_Y + 50.0,
                                    AREA_ORIGIN_Y + AREA_HEIGHT - 50.0,
                                    "raptor respawn y",
                                );
                                if cpu_entities.len() < 10 {
                                    cpu_entities.push(CpuEntity::new_raptor(raptor_x, raptor_y));
                                }
                            }
                        }
                    }
                    // Respawn Raptors if in field x1 y0 and nest not cleared
                    else if current_field == SbrxFieldId(1, 0)
                        && !task_system.is_task_complete("CLEAR RAPTOR NEST: FIELD[X1 Y0]")
                        && CPU_ENABLED
                    {
                        // Check if there's a raptor nest in the field
                        if !raptor_nests.is_empty() {
                            println!(
                                "Respawning raptors after party wipe near nest in field x1 y0"
                            );
                            if let Some(nest) = raptor_nests.first() {
                                if cpu_entities.len() < 10 {
                                    cpu_entities.push(CpuEntity::new_raptor(nest.x - 75.0, nest.y));
                                }
                                if cpu_entities.len() < 10 {
                                    cpu_entities.push(CpuEntity::new_raptor(nest.x + 75.0, nest.y));
                                }
                            }
                        }
                    }

                    // Reset T-Rex active flag if defeated during party wipe
                    let t_rex_exists = cpu_entities.iter().any(|e| e.variant == CpuVariant::TRex);
                    if !t_rex_exists && t_rex_is_active {
                        t_rex_is_active = false;
                    }

                    damage_texts.clear();
                    current_area = None;
                    pulse_orbs.clear();
					spheres.clear();

                    wave_manager.reset(); // BUG FIX: Reset wave state on full party wipe.

                    movement_active = false;
                    movement_timer = 0.0;
                    backpedal_active = false;
                    backpedal_timer = 0.0;
                    current_movement_direction = MovementDirection::None;
                    block_system = BlockSystem::new(20);
                    block_system.needs_dismount = false;
                    block_break_animation_active = false;

                    rush_active = false;
                    rush_timer = 0.0;
                    rush_cooldown = 0.0;
                    combo_system = ComboSystem::new();
                    strike_animation_timer = 0.0;
                    strike_frame = 0;
                    key_w_pressed = false;
                    key_s_pressed = false;
                    key_a_pressed = false;
                    key_d_pressed = false;

                    if let Some(sink) = bike_accelerate_sound_sink.take() {
                        sink.stop();
                    }
                    if let Some(sink) = bike_idle_sound_sink.take() {
                        sink.stop();
                    }
					
					arena_kill_count = 0; // Reset arena counter on full party wipe
					last_arena_milestone = 0;

                    // Clear downed state on full party wipe restart
                    downed_fighters.clear();
                    revival_kill_score = 0;
					
                    // Reset Endless Arena Mode Timer
                    if endless_arena_mode_active {
                        endless_arena_timer = 0.0;
                        endless_arena_stage = 1;
                        println!("Resetting Endless Arena timer after party wipe.");
                    }					
                }
            }
            GameState::DeathScreenGroup {
                death_type,
                downed_fighter_type,
            } => {
                if let Some(args) = e.update_args() {
                    if death_screen_cooldown > 0.0 {
                        death_screen_cooldown -= args.dt;
                    }
                }
                if let Some(button_args) = e.press_args() {
                    if let Button::Keyboard(key) = button_args {
                        if death_screen_cooldown > 0.0 {
                            continue;
                        }
                        let mut desired_fighter: Option<FighterType> = None;
                        match key {
                            Key::D1 => desired_fighter = Some(FighterType::Racer),
                            Key::D2 => {
                                if soldier_has_joined {
                                    desired_fighter = Some(FighterType::Soldier)
                                }
                            }
                            Key::D3 => {
                                if raptor_has_joined {
                                    desired_fighter = Some(FighterType::Raptor)
                                }
                            }
                            _ => {}
                        }

                        if let Some(target_fighter) = desired_fighter {
                            if downed_fighters.contains(&target_fighter) {
                                println!("Cannot select {:?} - fighter is downed.", target_fighter);
                            } else {
                                // Fighter is not downed, proceed with the switch
								
                                // BUG FIX: Reset lmb_held on group swap respawn
                                lmb_held = false;
                                melee_rapid_fire_timer = 0.0;
                                soldier_rapid_fire_timer = 0.0;								
								
                                audio_manager.play_sound_effect("death").ok();
                                fighter_hp_map.insert(fighter.fighter_type, fighter.current_hp);
                                let new_radius = fighter.switch_fighter_type(target_fighter);
                                fixed_crater.radius = new_radius;

                                // --- BUG FIX ---
                                // Reload the correct, leveled-up stats for the selected fighter.
								fighter.stats = fighter_stats_map
									.get(&target_fighter)
									.copied()
									.unwrap_or_else(|| match target_fighter {
										FighterType::Racer => combat::stats::RACER_LVL1_STATS,
										FighterType::Soldier => combat::stats::SOLDIER_LVL1_STATS,
										FighterType::Raptor => combat::stats::RAPTOR_LVL1_STATS,
									});
                                fighter.max_hp = fighter.stats.defense.hp;
                                fighter.melee_damage = fighter.stats.attack.melee_damage;
                                fighter.ranged_damage = fighter.stats.attack.ranged_damage;
                                fighter.run_speed = fighter.stats.speed.run_speed;
                                // --- END FIX ---

								fighter.current_hp = fighter_hp_map
									.get(&target_fighter)
									.copied()
									.unwrap_or(fighter.max_hp);

                                // --- BUG FIX: Restore LVL_UP prompt after respawn ---
                                lvl_up_state = LvlUpState::None; // Reset UI state first
                                let points_for_new_fighter = *fighter
                                    .stat_points_to_spend
                                    .get(&target_fighter)
                                    .unwrap_or(&0);
                                if points_for_new_fighter > 0 {
                                    lvl_up_state = LvlUpState::PendingTab {
                                        fighter_type: target_fighter,
                                    };
                                    let fighter_name = match target_fighter {
                                        FighterType::Racer => "RACER",
                                        FighterType::Soldier => "SOLDIER",
                                        FighterType::Raptor => "RAPTOR",
                                    };
                                    chatbox.add_interaction(vec![(
                                        &format!(
                                            "!! [TAB] TO LVL UP [{}] +[{}] !!",
                                            fighter_name, points_for_new_fighter
                                        ),
                                        MessageType::Warning,
                                    )]);
                                }
                                // --- END FIX ---

                                // Set combat mode based on fighter type
                                if fighter.fighter_type == FighterType::Raptor {
                                    fighter.combat_mode = CombatMode::CloseCombat;
                                }

                                // BUG FIX: Ensure the new fighter always spawns OnFoot.
                                fighter.state = RacerState::OnFoot;

                                let tex_set = match target_fighter {
                                    FighterType::Racer => &racer_textures,
                                    FighterType::Soldier => &soldier_textures,
                                    FighterType::Raptor => &raptor_textures,
                                };
                                update_current_textures(
                                    &fighter,
                                    tex_set,
                                    &mut current_idle_texture,
                                    &mut current_fwd_texture,
                                    &mut current_backpedal_texture,
                                    &mut current_block_texture,
                                    &mut current_block_break_texture,
                                    &mut current_ranged_texture,
                                    &mut current_ranged_marker_texture,
                                    &mut current_ranged_blur_texture,
                                    &mut current_rush_texture,
                                    &mut current_strike_textures,
									shift_held,
                                );
                                current_racer_texture = &tex_set.strike[2]; // Use strike3.png as starting frame
                                strike_animation_timer = 0.25; // Animate for a short duration

                                game_state = GameState::Playing;
								
                                // Reset milestone tracking if we are in Arena mode to ensure buffs resume correctly
                                if endless_arena_mode_active {
                                    last_arena_milestone = arena_kill_count / 50;
                                }								

                                key_w_pressed = false;
                                key_s_pressed = false;
                                key_a_pressed = false;
                                key_d_pressed = false;
                                movement_active = false;
                                backpedal_active = false;

                                for _ in 0..1 {
                                    let strike_x = fighter.x;
                                    let strike_y = fighter.y;
                                    strike.trigger(strike_x, strike_y);

                                    if !is_paused && CPU_ENABLED {
                                        for (_cpu_index, cpu) in cpu_entities.iter_mut().enumerate()
                                        {
                                            let sdx = strike_x - cpu.x;
                                            let sdy = strike_y - cpu.y;
                                            if (sdx * sdx + sdy * sdy).sqrt() < COLLISION_THRESHOLD
                                            {
                                                // Respawn strike: knockback only, no damage
                                                cpu.apply_knockback(strike_x, strike_y, 3000.0);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(_) = e.render_args() {
                    window.draw_2d(&e, |c, g, device| {
                        clear([0.0, 0.0, 0.0, 1.0], g);

                        let downed_fighter_name = match downed_fighter_type {
                            FighterType::Racer => "RACER",
                            FighterType::Soldier => "SOLDIER",
                            FighterType::Raptor => "RAPTOR",
                        };

                        let death_message_text = match death_type {
							DeathType::Crashed => "CRASHED",
                            DeathType::Meteorite => "WAS OBLITERATED BY METEORITE",
                            DeathType::GiantMantis => "WAS EATEN BY GIANT MANTIS",
                            DeathType::Rattlesnake => "WAS BITTEN BY RATTLESNAKE",
                            DeathType::GiantRattlesnake => "WAS SWALLOWED WHOLE BY GIANT RATTLESNAKE",
                            DeathType::BloodIdol => "WAS TORN APART BY BLOOD IDOL",
                            DeathType::VoidTempest => "WAS ANNIHILATED BY VOID TEMPEST",
                            DeathType::Raptor => "WAS MAULED BY A RAPTOR",
                            DeathType::TRex => "WAS DEVOURED BY A T-REX",
                            DeathType::FlyingSaucer => "VAPORIZED BY FLYING SAUCER",
                            DeathType::LightReaver => "WAS ABDUCTED BY A LIGHT REAVER",
                            DeathType::NightReaver => "WAS MUTILATED BY A NIGHT REAVER",
                            DeathType::RazorFiend => "WAS SHREDDED BY A RAZOR FIEND",
                        };

                        let font_size = 32;
                        let white = [1.0, 1.0, 1.0, 1.0];
                        let red = [1.0, 0.1, 0.1, 1.0];
                        let green = [0.1, 1.0, 0.1, 1.0];
                        let line_height = font_size as f64 + 15.0;

                        let mut current_y = screen_height / 2.0 - 180.0;

                        let line1 = format!("{}", downed_fighter_name);
                        let line1_width = glyphs.width(font_size, &line1).unwrap_or(0.0);
                        text(
                            white,
                            font_size,
                            &line1,
                            &mut glyphs,
                            c.transform
                                .trans((screen_width - line1_width) / 2.0, current_y),
                            g,
                        )
                        .ok();
                        current_y += line_height;

                        let line2_width =
                            glyphs.width(font_size, death_message_text).unwrap_or(0.0);
                        text(
                            red,
                            font_size,
                            death_message_text,
                            &mut glyphs,
                            c.transform
                                .trans((screen_width - line2_width) / 2.0, current_y),
                            g,
                        )
                        .ok();
                        current_y += line_height * 2.0;

                        let line3 = "SWAP TO:";
                        let line3_width = glyphs.width(font_size, line3).unwrap_or(0.0);
                        text(
                            white,
                            font_size,
                            line3,
                            &mut glyphs,
                            c.transform
                                .trans((screen_width - line3_width) / 2.0, current_y),
                            g,
                        )
                        .ok();
                        current_y += line_height;

                        let fighters_to_show = [
                            (
                                FighterType::Racer,
                                "RACER",
                                RACER_LVL1_STATS.defense.hp,
                                true,
                            ),
                            (
                                FighterType::Soldier,
                                "SOLDIER",
                                SOLDIER_LVL1_STATS.defense.hp,
                                soldier_has_joined,
                            ),
                            (
                                FighterType::Raptor,
                                "RAPTOR",
                                RAPTOR_LVL1_STATS.defense.hp,
                                raptor_has_joined,
                            ),
                        ];

                        for (ft, name, max_hp, has_joined) in fighters_to_show.iter() {
                            if *has_joined {
                                let current_hp = fighter_hp_map.get(ft).unwrap_or(&max_hp);
                                let hp_value = current_hp.floor() as i32;

                                let is_downed = downed_fighters.contains(ft);

                                let (name_color, hp_color) = if is_downed || hp_value <= 0 {
                                    (red, red)
                                } else {
                                    (white, green)
                                };

                                let key_num = match ft {
                                    FighterType::Racer => 1,
                                    FighterType::Soldier => 2,
                                    FighterType::Raptor => 3,
                                };

                                let entry_text = format!("{} [{}]", name, key_num);
                                let hp_text = format!("{}", hp_value.max(0));

                                let entry_width =
                                    glyphs.width(font_size, &entry_text).unwrap_or(0.0);
                                let hp_width = glyphs.width(font_size, &hp_text).unwrap_or(0.0);
                                let total_width = entry_width + hp_width + 20.0;
                                let start_x = (screen_width - total_width) / 2.0;

                                text(
                                    name_color,
                                    font_size,
                                    &entry_text,
                                    &mut glyphs,
                                    c.transform.trans(start_x, current_y),
                                    g,
                                )
                                .ok();
                                text(
                                    hp_color,
                                    font_size,
                                    &hp_text,
                                    &mut glyphs,
                                    c.transform.trans(start_x + entry_width + 20.0, current_y),
                                    g,
                                )
                                .ok();

                                current_y += line_height;
                            }
                        }
						
						

                        glyphs.factory.encoder.flush(device);
                    });
                }
            }

            GameState::LoadingFirmament => {
                // This state's only job is to draw the loading screen and then request the load.
                if let Some(_) = e.render_args() {
                    window.draw_2d(&e, |c, g, device| {
                        clear([0.0, 0.0, 0.0, 1.0], g);
                        let (w, h) = (
                            loading_screen_texture.get_width() as f64,
                            loading_screen_texture.get_height() as f64,
                        );
                        let x = (screen_width - w) / 2.0;
                        let y = (screen_height - h) / 2.0;
                        image(&loading_screen_texture, c.transform.trans(x, y), g);
                        glyphs.factory.encoder.flush(device);
                    });
                    firmament_load_requested = true;
                }
            }

            GameState::FirmamentMode(ref mut firmament_game) => {
                if let Some(args) = e.update_args() {
                    firmament_game.update(args.dt);
					chatbox.update(args.dt, enter_key_held);
					
                    // Update background_track notification lifetime in Firmament mode
                    if let Some(ref mut notif) = track_notification {
                        notif.lifetime -= args.dt;
                        if notif.lifetime <= 0.0 {
                            track_notification = None;
                        }
                    }					
					
                    // Ensure ambient track is playing in Firmament mode (respects [M] key state)
                    if !is_paused {
                        match ambient_track_state {
                            AmbientTrackState::Background => {
                                // FIRMAMENT audio now uses the same shared playlist logic as standard PLAYING mode
                                if let Some(track_name) = handle_ambient_playlist(&audio_manager, &mut current_bgm_sink, &mut ambient_playlist_index) {
                                    track_notification = Some(TrackNotification {
                                        track_name,
                                        lifetime: 3.0,
                                    });
                                }
                            }
                            AmbientTrackState::Crickets => {
                                if current_bgm_sink.is_some() {
                                    if let Some(sink) = current_bgm_sink.take() {
                                        sink.stop();
                                    }
                                }
                                if crickets_sound_sink.is_none() {
                                    match audio_manager.play_sfx_loop("crickets") {
                                        Ok(sink) => crickets_sound_sink = Some(sink),
                                        Err(e) => eprintln!("Failed to play crickets sound in Firmament: {}", e),
                                    }
                                }
                            }
                            AmbientTrackState::Muted => {
                                if let Some(sink) = current_bgm_sink.take() {
                                    sink.stop();
                                }
                                if let Some(sink) = crickets_sound_sink.take() {
                                    sink.stop();
                                }
                            }
                        }
                    }					

                    // Sync taskbar state from firmament and reset timer if opened
                    let prev_open = task_system.open;
                    task_system.open = firmament_game.task_bar_open;
                    if task_system.open && !prev_open {
                        task_system.auto_close_timer = 7.0; // Reset timer when taskbar is opened
                    }

                    // Check for boss defeat to update tasks and state
                    if !firmament_boss_defeated && firmament_game.is_boss_defeated() {
                        println!("Flying saucer defeated! Updating task system and main state.");
                        firmament_boss_defeated = true;
                        task_system.flying_saucer_defeated = true;
                    }

                    // Check if entered Fort Silo field in FIRMAMENT
                    let current_fm_field = firmament_game.get_current_field_id();
                    if current_fm_field.0 == -25
                        && current_fm_field.1 == 25
                        && !fort_silo_gravity_message_shown
                    {
                        chatbox.add_interaction(vec![
                            (
                                "ENTRAPPED BY A GRAVITATIONAL FORCE",
                                MessageType::Notification,
                            ),
                            //	("ENTRAPPED BY A GRAVITATIONAL FORCE", MessageType::Warning),
                        ]);
                        fort_silo_gravity_message_shown = true;
                    } else if current_fm_field.0 != -25 || current_fm_field.1 != 25 {
                        // Reset flag when leaving Fort Silo field
                        fort_silo_gravity_message_shown = false;
                    }

                    // Check if fighter_jet has entered Fort Silo field
                    let current_fm_field = firmament_game.get_current_field_id();
                    if current_fm_field.0 == -25 && current_fm_field.1 == 25 {
                        if !task_system.fort_silo_reached {
                            println!("fighter_jet entered Fort Silo airfield! Marking task complete.");
                            task_system.mark_fort_silo_reached();
                        }
                    }

                    // Update task system in firmament mode
                    task_system.update();
					task_system.update_timer(args.dt);
					// Sync back: if timer closed the taskbar, update firmament's state too
					firmament_game.task_bar_open = task_system.open;					
                }
                if let Some(_args) = e.render_args() {
                    window.draw_2d(&e, |c, g, device| {
                        firmament_game.render(c, g, &mut glyphs);
                        if is_paused {
                            let (w, h) = (
                                pause_screen_texture.get_width() as f64,
                                pause_screen_texture.get_height() as f64,
                            );
                            let x = (screen_width - w) / 2.0;
                            let y = (screen_height - h) / 2.0;
                            image(&pause_screen_texture, c.transform.trans(x, y), g);
                        }

                        // Draw task system in Firmament mode
                        task_system.draw(c, g, &mut glyphs);

                        // Draw fighter group icons in Firmament mode
                        fighter_hp_map.insert(fighter.fighter_type, fighter.current_hp);
                        let mut current_x = 25.0;
                        let start_y = 70.0;
                        let padding = 10.0;
                        let font_size = 14;
                        let text_color = [1.0, 1.0, 1.0, 1.0];

                        let fighter_types_in_group = [
                            FighterType::Racer,
                            FighterType::Soldier,
                            FighterType::Raptor,
                        ];

                        for ft in &fighter_types_in_group {
                            let has_joined = match ft {
                                FighterType::Racer => true,
                                FighterType::Soldier => soldier_has_joined,
                                FighterType::Raptor => raptor_has_joined,
                            };

                            if has_joined && !downed_fighters.contains(ft) {
                                let is_selected = fighter.fighter_type == *ft;
                                let icon_to_draw = if is_selected {
                                    group_icons_selected.get(ft)
                                } else {
                                    group_icons.get(ft)
                                };

                                if let Some(icon) = icon_to_draw {
                                    let icon_width = icon.get_width() as f64;
                                    let icon_height = icon.get_height() as f64;

                                    image(icon, c.transform.trans(current_x, start_y), g);

                                    // Draw '+' sign if level up points are available
                                    if fighter.stat_points_to_spend.get(ft).unwrap_or(&0) > &0 {
                                        let plus_text = "+";
                                        let plus_font_size = 20;
                                        let plus_color = [1.0, 0.5, 0.0, 1.0]; // Orange
                                                                               // Position near top-right of the icon
                                        let plus_x = current_x + icon_width - 10.0;
                                        let plus_y_baseline = start_y + 10.0;
                                        text::Text::new_color(plus_color, plus_font_size)
                                            .draw(
                                                plus_text,
                                                &mut glyphs,
                                                &c.draw_state,
                                                c.transform.trans(plus_x, plus_y_baseline),
                                                g,
                                            )
                                            .ok();
                                    }

                                    let (_max_hp, current_hp) = match ft {
                                        FighterType::Racer => (
                                            RACER_LVL1_STATS.defense.hp,
                                            *fighter_hp_map.get(ft).unwrap_or(&0.0),
                                        ),
                                        FighterType::Soldier => (
                                            SOLDIER_LVL1_STATS.defense.hp,
                                            *fighter_hp_map.get(ft).unwrap_or(&0.0),
                                        ),
                                        FighterType::Raptor => (
                                            RAPTOR_LVL1_STATS.defense.hp,
                                            *fighter_hp_map.get(ft).unwrap_or(&0.0),
                                        ),
                                    };


                                    let hp_text = format!("{:.0}", current_hp.max(0.0));

                                    let text_y = start_y + icon_height + font_size as f64;

                                    text::Text::new_color(text_color, font_size)
                                        .draw(
                                            &hp_text,
                                            &mut glyphs,
                                            &c.draw_state,
                                            c.transform.trans(current_x, text_y),
                                            g,
                                        )
                                        .ok();

                                    current_x += icon_width + padding;
                                }
                            }
                        }

                        chatbox.draw(c, g, &mut glyphs); // Draw chatbox in Firmament mode if open

                        // Draw on-screen warnings on top of everything
                        chatbox.draw_warnings(c, g, &mut glyphs);

                        // --- TRACK NOTIFICATION (bottom right) - Firmament Mode ---
                        if let Some(ref notif) = track_notification {
                            let text = format!("track: {}", notif.track_name);
                            let font_size = 18;
                            let text_width = glyphs.width(font_size, &text).unwrap_or(150.0);
                            let padding = 8.0;
                            let box_width = text_width + padding * 2.0;
                            let box_height = font_size as f64 + padding * 2.0;
                            let box_x = screen_width - box_width - 20.0;
                            let box_y = screen_height - box_height - 20.0;
                            
                            // Dark gray background padding
                            rectangle(
                                [0.2, 0.2, 0.2, 0.8],
                                [box_x, box_y, box_width, box_height],
                                c.transform,
                                g,
                            );
                            
                            // Green text
                            text::Text::new_color([0.0, 1.0, 0.0, 1.0], font_size)
                                .draw(
                                    &text,
                                    &mut glyphs,
                                    &c.draw_state,
                                    c.transform.trans(box_x + padding, box_y + padding + font_size as f64 - 2.0),
                                    g,
                                )
                                .ok();
                        }

                        glyphs.factory.encoder.flush(device);
                    });
                }
                let mut handle_firmament_exit =
                    |target_sbrx_field_for_player: SbrxFieldId,
                     player_bike_landing_x: f64,
                     player_bike_landing_y: f64,
                     new_fighter_jet_sbrx_location: SbrxFieldId,
                     new_fighter_jet_world_x: f64,
                     new_fighter_jet_world_y: f64,
                     new_next_firmament_entry_id: firmament_lib::FieldId3D| {
                        sbrx_map_system.current_field_id = target_sbrx_field_for_player;
                        last_field_entry_point = (player_bike_landing_x, player_bike_landing_y);
                        rattlesnakes_spawned_in_field0_score3 = false;
                        fighter.x = player_bike_landing_x;
                        fighter.y = player_bike_landing_y;
                        fighter.state = RacerState::OnFoot;
                        fixed_crater.x = fighter.x;
                        fixed_crater.y = fighter.y;
                        camera.x = fighter.x;
                        camera.y = fighter.y;
                        sbrx_bike.respawn(player_bike_landing_x, player_bike_landing_y);
                        fighter_jet_current_sbrx_location = new_fighter_jet_sbrx_location;
                        fighter_jet_world_x = new_fighter_jet_world_x;
                        fighter_jet_world_y = new_fighter_jet_world_y;
                        next_firmament_entry_field_id = new_next_firmament_entry_id;
                        let tex_set = match fighter.fighter_type {
                            FighterType::Racer => &racer_textures,
                            FighterType::Soldier => &soldier_textures,
                            FighterType::Raptor => &raptor_textures,
                        };
                        update_current_textures(
                            &fighter,
                            tex_set,
                            &mut current_idle_texture,
                            &mut current_fwd_texture,
                            &mut current_backpedal_texture,
                            &mut current_block_texture,
                            &mut current_block_break_texture,
                            &mut current_ranged_texture,
                            &mut current_ranged_marker_texture,
                            &mut current_ranged_blur_texture,
                            &mut current_rush_texture,
                            &mut current_strike_textures,
							shift_held,
                        );
                        current_racer_texture = current_idle_texture;
                        movement_active = false;
                        movement_timer = 0.0;
                        backpedal_active = false;
                        backpedal_timer = 0.0;
                        strike_animation_timer = 0.0;
                        strike_frame = 0;
                        rush_active = false;
                        rush_timer = 0.0;
                        key_w_pressed = false;
                        key_s_pressed = false;
                        key_a_pressed = false;
                        key_d_pressed = false;
                        block_system.deactivate();

                        // Spawn Light Reavers when landing at Fort Silo from Firmament
                        if target_sbrx_field_for_player == SbrxFieldId(-25, 25) && CPU_ENABLED {
                            println!("Spawning Light Reavers in Fort Silo field x-25 y25 (from Firmament landing)");
                            cpu_entities.clear(); // Clear any existing entities first
                            for _ in 0..2 {
                                let reaver_x = safe_gen_range(MIN_X, MAX_X, "LightReaver x");
                                let reaver_y = safe_gen_range(MIN_Y, MAX_Y, "LightReaver y");
                                cpu_entities.push(CpuEntity::new_light_reaver(reaver_x, reaver_y));
                            }
                        }
						
                        // Heal Racer for the finale if landing on Racetrack
                        if target_sbrx_field_for_player == SbrxFieldId(0, 0) {
                             fighter.current_hp = fighter.max_hp;
                             fighter_hp_map.insert(FighterType::Racer, fighter.max_hp);

							// Ensure Racer is not marked as downed
							if downed_fighters.contains(&FighterType::Racer) {
								downed_fighters.retain(|&ft| ft != FighterType::Racer);
								println!("Revived Racer upon landing for finale.");
							}

                            // Task logic: Soldier and raptor leave the group for the race (Handle inside closure now)
                            if soldier_has_joined || raptor_has_joined {
                                //println!("[FINALE] Landing on racetrack for finale. Soldier and raptor will leave the group.");
                                soldier_has_joined = false;
                                raptor_has_joined = false;
 
                                // Remove them from downed list if they are downed
                                downed_fighters.retain(|&ft| ft != FighterType::Soldier && ft != FighterType::Raptor);
                            }  							
                        }						

                        next_game_state_after_event = Some(GameState::Playing);
                    };
                if let Some(Button::Keyboard(key)) = e.press_args() {
                    match key {
                        Key::F5 //key test
                            if !is_paused
                                && (!firmament_game.is_boss_fight_active()
                                    || firmament_game.is_boss_defeated()) =>
                        {
                            println!("F5 (EJECT) pressed in FirmamentMode.");
							audio_manager.play_sound_effect("death").ok();
                            let current_fm_field = firmament_game.get_current_field_id();		
							
                            // Special Case: Ejecting on Racetrack during Finale Task acts as Landing (F1 behavior)
                            if current_fm_field.0 == 0 && current_fm_field.1 == 0
                                && task_system.has_task("LAND THE FIGHTERJET ON THE RACETRACK")
                                && !task_system.is_task_complete("LAND THE FIGHTERJET ON THE RACETRACK")
                            {
                                println!("F5 pressed on Racetrack during Finale. Treating as Landing.");
                                task_system.mark_racer_returned_to_racetrack();
                                
                                let target_sbrx_field_for_all = SbrxFieldId(0, 0);
                                // Specific landing zone for the Racetrack (top-left corner)
                                let (common_landing_x, common_landing_y) = (MIN_X + 50.0, MIN_Y + 50.0);
                                
                                println!("Player, Bike, and fighter_jet landing at ({:.2}, {:.2}) in SBRX field x[0],y[0]", common_landing_x, common_landing_y);
                                handle_firmament_exit(
                                    target_sbrx_field_for_all,
                                    common_landing_x,
                                    common_landing_y,
                                    target_sbrx_field_for_all,
                                    common_landing_x,
                                    common_landing_y,
                                    current_fm_field,
                                );
                                // Skip the standard eject logic below
                                continue;
                            }							
							
                            // Check for ejecting at Fort Silo to complete task
                            if current_fm_field.0 == -25 && current_fm_field.1 == 25 {
                                if task_system.has_task("LAND ON FORT SILO") {
                                    println!("Ejected onto Fort Silo. Updating task system.");
                                    task_system.landed_on_fort_silo = true;
                                }
                            }
                            let firmament_score = firmament_game.get_score();
                            println!("Firmament score upon eject: {}", firmament_score);
                            println!(
                                "Ejecting from FIRMAMENT field x[{}], y[{}], z[{}]",
                                current_fm_field.0, current_fm_field.1, current_fm_field.2
                            );
                            let target_sbrx_field_for_player =
                                SbrxFieldId(current_fm_field.0, current_fm_field.1);
                            let player_bike_landing_x =
                                safe_gen_range(MIN_X, MAX_X, "Eject landing x");
                            let player_bike_landing_y =
                                safe_gen_range(MIN_Y, MAX_Y, "Eject landing y");							
                            // Add a crashed fighter_jet site at the landing location
                            crashed_fighter_jet_sites.push(CrashedFighterJetSite {
                                sbrx_field_id: target_sbrx_field_for_player,
                                world_x: player_bike_landing_x,
                                world_y: player_bike_landing_y,
                            });
                            println!("fighter_jet ejected. Crash site registered in SBRX field ({},{}) at world coords ({:.2}, {:.2})", 
                                target_sbrx_field_for_player.0, target_sbrx_field_for_player.1, player_bike_landing_x, player_bike_landing_y
                            );
                            let fighter_jet_basecamp_fm_id = firmament_lib::FieldId3D(-2, 5, 0);
                            let fighter_jet_basecamp_sbrx_id = SbrxFieldId(-2, 5);
                            let fighter_jet_visual_base_x = DEFAULT_FIGHTER_JET_WORLD_X;
                            let fighter_jet_visual_base_y = DEFAULT_FIGHTER_JET_WORLD_Y;
                            println!(
                                "Player/Bike landing at ({:.2}, {:.2}) in SBRX field x[{}],y[{}]",
                                player_bike_landing_x,
                                player_bike_landing_y,
                                target_sbrx_field_for_player.0,
                                target_sbrx_field_for_player.1
                            );
                            println!("fighter_jet auto-piloting to FIRMAMENT base x[{}],y[{}],z[{}]. fighter_jet visual in SBRX to field x[{}],y[{}] at ({:.2}, {:.2})", fighter_jet_basecamp_fm_id.0, fighter_jet_basecamp_fm_id.1, fighter_jet_basecamp_fm_id.2, fighter_jet_basecamp_sbrx_id.0, fighter_jet_basecamp_sbrx_id.1, fighter_jet_visual_base_x, fighter_jet_visual_base_y);
                            handle_firmament_exit(
                                target_sbrx_field_for_player,
                                player_bike_landing_x,
                                player_bike_landing_y,
                                fighter_jet_basecamp_sbrx_id,
                                fighter_jet_visual_base_x,
                                fighter_jet_visual_base_y,
                                fighter_jet_basecamp_fm_id,
                            );
                        }
                        Key::F1 // key test
                            if !is_paused
                                && (!firmament_game.is_boss_fight_active()
                                    || firmament_game.is_boss_defeated()) =>
                        {
                            println!("F1 (LAND) pressed in FirmamentMode.");
                            let current_fm_field = firmament_game.get_current_field_id();
                            let target_sbrx_field_for_all =
                                SbrxFieldId(current_fm_field.0, current_fm_field.1);

                            // Check for landing at Fort Silo to complete task
                            if current_fm_field.0 == -25 && current_fm_field.1 == 25 {
                                if task_system.has_task("LAND ON FORT SILO") {
                                    println!("Landed on Fort Silo. Updating task system.");
                                    task_system.landed_on_fort_silo = true;
                                }
                            }

                            // Check for returning to racetrack task
                            if task_system.has_task("LAND THE FIGHTERJET ON THE RACETRACK")
                                && !task_system.is_task_complete("LAND THE FIGHTERJET ON THE RACETRACK")
                                && target_sbrx_field_for_all == SbrxFieldId(0, 0)
                            {
                                println!("Landed on Racetrack. Completing task.");
                                task_system.mark_racer_returned_to_racetrack();								
                            }

                            let firmament_score = firmament_game.get_score();
                            println!("Firmament score upon land: {}", firmament_score);
                            println!(
                                "Landing from FIRMAMENT field x[{}], y[{}], z[{}]",
                                current_fm_field.0, current_fm_field.1, current_fm_field.2
                            );

                            let (common_landing_x, common_landing_y) =
                                if current_fm_field.0 == -25 && current_fm_field.1 == 25 {
                                    // Specific landing zone for Fort Silo
                                    (250.0, 900.0)
                                } else if target_sbrx_field_for_all == SbrxFieldId(0, 0) {
                                    // Specific landing zone for the Racetrack (top-left corner)
                                    (MIN_X + 50.0, MIN_Y + 50.0)
                                } else {
                                    // Random landing for other fields
                                    (
                                        safe_gen_range(MIN_X, MAX_X, "Land landing x"),
                                        safe_gen_range(MIN_Y, MAX_Y, "Land landing y"),
                                    )
                                };
                            println!("Player, Bike, and fighter_jet landing at ({:.2}, {:.2}) in SBRX field x[{}],y[{}]", common_landing_x, common_landing_y, target_sbrx_field_for_all.0, target_sbrx_field_for_all.1);
                            println!("Next Firmament entry will be from FIRMAMENT field x[{}],y[{}],z[{}]", current_fm_field.0, current_fm_field.1, current_fm_field.2);
                            handle_firmament_exit(
                                target_sbrx_field_for_all,
                                common_landing_x,
                                common_landing_y,
                                target_sbrx_field_for_all,
                                common_landing_x,
                                common_landing_y,
                                current_fm_field,
                            );
                        }
                        _ => {
                            firmament_game.key_pressed(key);
                        }
                    }
                }
                if let Some(Button::Mouse(button)) = e.press_args() {
                    firmament_game.mouse_pressed(button);
                }
                if let Some(Button::Keyboard(key)) = e.release_args() {
                    firmament_game.key_released(key);
                }
                if !is_paused && firmament_game.is_game_over() {
                    if firmament_game.death_cause
                        == Some(firmament_lib::FirmamentDeathCause::FlyingSaucer)
                    {
                        game_state = GameState::DeathScreen(DeathType::FlyingSaucer);
                        death_screen_cooldown = DEATH_SCREEN_COOLDOWN_TIME;
                        audio_manager.play_sound_effect("death").ok();
                        if let Some(sink) = bike_accelerate_sound_sink.take() {
                            sink.stop();
                        }
                        if let Some(sink) = bike_idle_sound_sink.take() {
                            sink.stop();
                        }
                        if let Some(sink) = crickets_sound_sink.take() {
                            sink.stop();
                        }
					
                        lmb_held = false; // Stop rapid fire on death
                        continue; // Skip the rest of the exit logic for this frame
                    }

                    audio_manager.play_sound_effect("death").ok();
                    println!(
                        "Firmament game over. Returning to SBRX (fighter_jet destroyed scenario)."
                    );
                    lmb_held = false; // Stop rapid fire on death
                    let current_fm_field = firmament_game.get_current_field_id();
                    let target_sbrx_field_for_player =
                        SbrxFieldId(current_fm_field.0, current_fm_field.1);
                    let player_bike_landing_x = safe_gen_range(MIN_X, MAX_X, "FM death landing x");
                    let player_bike_landing_y = safe_gen_range(MIN_Y, MAX_Y, "FM death landing y");
                    crashed_fighter_jet_sites.push(CrashedFighterJetSite {
                        sbrx_field_id: target_sbrx_field_for_player,
                        world_x: player_bike_landing_x,
                        world_y: player_bike_landing_y,
                    });
                    println!("fighter_jet crashed in FIRMAMENT field ({},{},{}). Crash site registered in SBRX field ({},{}) at world coords ({:.2}, {:.2})", current_fm_field.0, current_fm_field.1, current_fm_field.2, target_sbrx_field_for_player.0, target_sbrx_field_for_player.1, player_bike_landing_x, player_bike_landing_y);
                    let fighter_jet_basecamp_fm_id = firmament_lib::FieldId3D(-2, 5, 0);
                    let fighter_jet_basecamp_sbrx_id = SbrxFieldId(-2, 5);
                    let fighter_jet_visual_base_x = DEFAULT_FIGHTER_JET_WORLD_X;
                    let fighter_jet_visual_base_y = DEFAULT_FIGHTER_JET_WORLD_Y;
                    handle_firmament_exit(
                        target_sbrx_field_for_player,
                        player_bike_landing_x,
                        player_bike_landing_y,
                        fighter_jet_basecamp_sbrx_id,
                        fighter_jet_visual_base_x,
                        fighter_jet_visual_base_y,
                        fighter_jet_basecamp_fm_id,
                    );
                }
            }
        }
        if let Some(new_state) = next_game_state_after_event.take() {
            game_state = new_state;
        }
    }
    println!("sbrx0.2.16 Game loop ended.");
}