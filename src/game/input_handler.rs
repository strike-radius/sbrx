// game/input_handler.rs

use crate::audio::AudioManager;
use crate::entities::cpu_entity::CpuEntity;
use crate::entities::fighter::Fighter;
use crate::entities::fixed_crater::FixedCrater;
use crate::entities::sbrx_bike::SbrxBike;
use crate::entities::shoot::Shoot;
use crate::entities::strike::Strike;
use crate::game_state::{FighterType, MovementDirection};
use crate::graphics::camera::screen_to_world;
use crate::graphics::camera::Camera;
use crate::graphics::fighter_textures::FighterTextures;
use crate::utils::collision::check_line_collision;
use piston_window::*;

pub fn handle_mouse_press<'a>(
    button: MouseButton,
    fighter: &mut Fighter,
    camera: &Camera,
    mouse_x: f64,
    mouse_y: f64,
    fixed_crater: &mut FixedCrater,
    strike: &mut Strike,
    shoot: &mut Shoot,
    cpu_entities: &mut Vec<CpuEntity>,
    line_y: f64,
    current_racer_texture: &mut &'a G2dTexture,
    current_block_texture: &'a G2dTexture,
    current_ranged_texture: &'a G2dTexture,
    current_strike_textures: &'a Vec<G2dTexture>,
    strike_frame: &mut usize,
    strike_animation_timer: &mut f64,
    block_active: &mut bool,
    rmb_held: &mut bool,
    audio_manager: &AudioManager,
) {
    match button {
        MouseButton::Left => {
            *block_active = false;
            *rmb_held = false;
            let (world_mouse_x, world_mouse_y) = screen_to_world(camera, mouse_x, mouse_y);
            let dx = world_mouse_x - fixed_crater.x;
            let dy = world_mouse_y - fixed_crater.y;
            let horizontal_radius = fixed_crater.radius;
            let vertical_radius = fixed_crater.radius * 0.75;
            let distance_squared = (dx * dx) / (horizontal_radius * horizontal_radius)
                + (dy * dy) / (vertical_radius * vertical_radius);

            if distance_squared <= 1.0 {
                // Melee attack - play slash sound
                audio_manager
                    .play_sound_effect("death")
                    .unwrap_or_else(|e| println!("Failed to play melee sound: {}", e));

                strike.trigger(world_mouse_x, world_mouse_y);

                // Always play strike animation for melee attacks
                if !current_strike_textures.is_empty() {
                    *current_racer_texture =
                        &current_strike_textures[*strike_frame % current_strike_textures.len()];
                    *strike_frame = (*strike_frame + 1) % current_strike_textures.len();
                }
                *strike_animation_timer = 0.25;

                // Process CPU hits (reuse existing code from main.rs)
                if crate::config::CPU_ENABLED {
                    for cpu_entity in cpu_entities.iter_mut() {
                        let strike_dx = world_mouse_x - cpu_entity.x;
                        let strike_dy = world_mouse_y - cpu_entity.y;
                        if (strike_dx * strike_dx + strike_dy * strike_dy).sqrt() < 50.0 {
                            cpu_entity.current_hp -= 25.0;
                            if cpu_entity.is_dead() {
                                fighter.score = (fighter.score + 1).min(999);
                                fighter.current_hp =
                                    (fighter.current_hp + 25.0).min(fighter.max_hp);
                                cpu_entity.respawn(line_y);
                            } else {
                                cpu_entity.apply_knockback(world_mouse_x, world_mouse_y, 400.0);
                            }
                        }
                    }
                }
            } else {
                if shoot.cooldown <= 0.0 {
                    // Ranged attack - play ranged sound
                    audio_manager
                        .play_sound_effect("ranged")
                        .unwrap_or_else(|e| println!("Failed to play ranged sound: {}", e));

                    shoot.trigger(fighter.x, fighter.y, world_mouse_x, world_mouse_y);
                    *current_racer_texture = current_ranged_texture;
                    *strike_animation_timer = 0.25;
                }
            }
        }
        MouseButton::Right => {
            // Block action - play block sound
            audio_manager
                .play_sound_effect("block")
                .unwrap_or_else(|e| println!("Failed to play block sound: {}", e));

            *rmb_held = true;
            *current_racer_texture = current_block_texture;
            *block_active = true;

            // Cancel conflicting movement animations
            // (reuse existing code from main.rs)
        }
        _ => {}
    }
}

// Function to handle space key press for rush action
pub fn handle_space_key<'a>(
    fighter: &mut Fighter,
    camera: &Camera,
    mouse_x: f64,
    mouse_y: f64,
    fixed_crater: &mut FixedCrater,
    strike: &mut Strike,
    cpu_entities: &mut Vec<CpuEntity>,
    line_y: f64,
    current_racer_texture: &mut &'a G2dTexture,
    current_rush_texture: &'a G2dTexture,
    block_active: &mut bool,
    rmb_held: &mut bool,
    movement_active: &mut bool,
    backpedal_active: &mut bool,
    current_movement_direction: &mut MovementDirection,
    rush_active: &mut bool,
    rush_timer: &mut f64,
    rush_cooldown: &mut f64,
    audio_manager: &AudioManager,
) {
    if *rush_cooldown <= 0.0 && !*block_active {
        // Play rush sound effect
        audio_manager
            .play_sound_effect("rush")
            .unwrap_or_else(|e| println!("Failed to play rush sound: {}", e));

        *block_active = false;
        *rmb_held = false;

        // Reuse existing rush code from main.rs
        let (world_mouse_x, world_mouse_y) = screen_to_world(camera, mouse_x, mouse_y);
        let dx = world_mouse_x - fighter.x;
        let dy = world_mouse_y - fighter.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance > 0.0 {
            let initial_x = fighter.x;
            let initial_y = fighter.y;
            let norm_dx = dx / distance;
            let norm_dy = dy / distance;

            // Calculate end position and collision check line
            let base_rush_distance = crate::config::movement::RUSH_DISTANCE;
            let rush_distance = match fighter.fighter_type {
                FighterType::Racer => base_rush_distance,
                FighterType::Hunter => base_rush_distance * 0.85,
                FighterType::Soldier => base_rush_distance * 0.65,
            };
            let rush_end_x = (initial_x + norm_dx * rush_distance).clamp(0.0, 5000.0);
            let rush_end_y = (initial_y + norm_dy * rush_distance).clamp(line_y, 3250.0);
            let collision_end_x = initial_x + norm_dx * (rush_distance * 2.50);
            let collision_end_y = initial_y + norm_dy * (rush_distance * 2.50);

            // Check collision along the extended path
            if crate::config::CPU_ENABLED {
                for cpu_entity in cpu_entities.iter_mut() {
                    if check_line_collision(
                        initial_x,
                        initial_y,
                        collision_end_x,
                        collision_end_y,
                        cpu_entity.x,
                        cpu_entity.y,
                    ) {
                        cpu_entity.current_hp -= 25.0;
                        if cpu_entity.is_dead() {
                            fighter.score = (fighter.score + 1).min(999);
                            fighter.current_hp = (fighter.current_hp + 25.0).min(fighter.max_hp);
                            cpu_entity.respawn(line_y);
                        } else {
                            cpu_entity.apply_knockback(initial_x, initial_y, 400.0);
                        }
                    }
                }
            }

            // Move fighter
            fighter.x = rush_end_x;
            fighter.y = rush_end_y;

            // Update fixed crater position
            fixed_crater.x = fighter.x;
            fixed_crater.y = fighter.y;

            // Trigger rush animation state
            *current_racer_texture = current_rush_texture;
            *rush_active = true;
            *rush_timer = crate::config::movement::RUSH_DURATION;
            *rush_cooldown = 0.5;

            // Trigger melee strike visual effect partway through rush
            let slash_x = fighter.x + norm_dx * (rush_distance * 0.5);
            let slash_y = fighter.y + norm_dy * (rush_distance * 0.5);
            strike.trigger(slash_x, slash_y);
            strike.angle = (-dy).atan2(dx).to_degrees();
            strike.timer = 0.2;

            // Reset conflicting movement states
            *movement_active = false;
            *backpedal_active = false;
            *current_movement_direction = MovementDirection::None;
        }
    }
}

// Main key press handler function
pub fn handle_key_press<'a>(
    key: Key,
    fighter: &mut Fighter,
    camera: &Camera,
    mouse_x: f64,
    mouse_y: f64,
    fixed_crater: &mut FixedCrater,
    strike: &mut Strike,
    shoot: &mut Shoot,
    sbrx_bike: &mut SbrxBike,
    cpu_entities: &mut Vec<CpuEntity>,
    line_y: f64,
    current_racer_texture: &mut &'a G2dTexture,
    current_idle_texture: &'a G2dTexture,
    current_fwd_texture: &'a G2dTexture,
    current_backpedal_texture: &'a G2dTexture,
    current_block_texture: &'a G2dTexture,
    current_ranged_texture: &'a G2dTexture,
    current_rush_texture: &'a G2dTexture,
    current_strike_textures: &'a Vec<G2dTexture>,
    racer_textures: &FighterTextures,
    soldier_textures: &FighterTextures,
    key_w_pressed: &mut bool,
    key_s_pressed: &mut bool,
    key_a_pressed: &mut bool,
    key_d_pressed: &mut bool,
    block_active: &mut bool,
    rmb_held: &mut bool,
    movement_active: &mut bool,
    movement_timer: &mut f64,
    backpedal_active: &mut bool,
    backpedal_timer: &mut f64,
    current_movement_direction: &mut MovementDirection,
    movement_buffer_timer: f64,
    rush_active: &mut bool,
    rush_timer: &mut f64,
    rush_cooldown: &mut f64,
    audio_manager: &AudioManager,
) {
    match key {
        Key::Space => {
            handle_space_key(
                fighter,
                camera,
                mouse_x,
                mouse_y,
                fixed_crater,
                strike,
                cpu_entities,
                line_y,
                current_racer_texture,
                current_rush_texture,
                block_active,
                rmb_held,
                movement_active,
                backpedal_active,
                current_movement_direction,
                rush_active,
                rush_timer,
                rush_cooldown,
                audio_manager,
            );
        }
        _ => {}
    }
}
