// src/area/area.rs

use piston_window::*;
use rand::Rng;

// Define the area's properties as constants for easy access and modification.
// The area is a 1000x1000 box placed at a specific coordinate in the larger game world.
pub const AREA_WIDTH: f64 = 1000.0;
pub const AREA_HEIGHT: f64 = 1000.0;
pub const AREA_ORIGIN_X: f64 = 2000.0; // Place it somewhere distinct in the world.
pub const AREA_ORIGIN_Y: f64 = 1500.0;

pub const BUNKER_WIDTH: f64 = 2000.0;
pub const BUNKER_HEIGHT: f64 = 1000.0;
pub const BUNKER_ORIGIN_X: f64 = 2000.0;
pub const BUNKER_ORIGIN_Y: f64 = 1500.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AreaType {
    RaptorNest,
    Bunker,
}

/// Represents the randomly generated exit point within an area.
#[derive(Debug, Clone, Copy)]
pub struct ExitPoint {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct TransitionPoint {
    pub rect: ExitPoint,
    pub target_floor: i32,
}

/// Holds the state of the currently active area, including its exit point.
#[derive(Debug, Clone)]
pub struct AreaState {
    pub exit_to_world: ExitPoint,
    pub floor_transitions: Vec<TransitionPoint>,
    pub area_type: AreaType,
    pub floor: i32,
    pub spawn_night_reavers: bool,
    pub waves_active: bool,
    pub is_peaceful: bool,
}

impl AreaState {
    /// It randomly determines an exit point on one of the four edges and calculates
    /// the corresponding starting position for the player.
    /// Returns the new state and the player's starting coordinates.
    pub fn new(
        area_type: AreaType,
        floor: i32,
        waves_active: bool,
        is_peaceful: bool,
    ) -> (AreaState, (f64, f64)) {
        let (width, height, origin_x, origin_y) = match area_type {
            AreaType::RaptorNest => (AREA_WIDTH, AREA_HEIGHT, AREA_ORIGIN_X, AREA_ORIGIN_Y),
            AreaType::Bunker => (
                BUNKER_WIDTH,
                BUNKER_HEIGHT,
                BUNKER_ORIGIN_X,
                BUNKER_ORIGIN_Y,
            ),
        };

        let mut rng = rand::rng();
        let mut spawn_night_reavers = false;

        let exit_line_thickness = 5.0;
        let exit_line_length = 60.0;
        let player_offset_from_edge = 50.0; // How far inside the player spawns from the edge.

        // --- WORLD EXIT (GREEN MARKER) ---
        // Always at the bottom for Bunker, random for RaptorNest
        let (exit_to_world, player_start_pos) = if area_type == AreaType::Bunker {
            // Bottom edge for Bunker exit
            let x_pos = origin_x + rng.random_range(0.0..width - exit_line_length);
            let ep = ExitPoint {
                x: x_pos,
                y: origin_y + height - exit_line_thickness,
                width: exit_line_length,
                height: exit_line_thickness,
            };
            // Player starts at the world exit point upon entering
            let psp = (
                x_pos + exit_line_length / 2.0,
                origin_y + height - player_offset_from_edge,
            );
            (ep, psp)
        } else {
            // Original random side logic for RaptorNest
            let side = rng.random_range(0..4); // 0: top, 1: right, 2: bottom, 3: left
            match side {
                0 => {
                    // Top edge
                    let x_pos = origin_x + rng.random_range(0.0..width - exit_line_length);
                    let ep = ExitPoint {
                        x: x_pos,
                        y: origin_y,
                        width: exit_line_length,
                        height: exit_line_thickness,
                    };
                    let psp = (
                        x_pos + exit_line_length / 2.0,
                        origin_y + player_offset_from_edge,
                    );
                    (ep, psp)
                }
                1 => {
                    // Right edge
                    let y_pos = origin_y + rng.random_range(0.0..height - exit_line_length);
                    let ep = ExitPoint {
                        x: origin_x + width - exit_line_thickness,
                        y: y_pos,
                        width: exit_line_thickness,
                        height: exit_line_length,
                    };
                    let psp = (
                        origin_x + width - player_offset_from_edge,
                        y_pos + exit_line_length / 2.0,
                    );
                    (ep, psp)
                }
                2 => {
                    // Bottom edge
                    let x_pos = origin_x + rng.random_range(0.0..width - exit_line_length);
                    let ep = ExitPoint {
                        x: x_pos,
                        y: origin_y + height - exit_line_thickness,
                        width: exit_line_length,
                        height: exit_line_thickness,
                    };
                    let psp = (
                        x_pos + exit_line_length / 2.0,
                        origin_y + height - player_offset_from_edge,
                    );
                    (ep, psp)
                }
                _ => {
                    // Left edge (3)
                    let y_pos = origin_y + rng.random_range(0.0..height - exit_line_length);
                    let ep = ExitPoint {
                        x: origin_x,
                        y: y_pos,
                        width: exit_line_thickness,
                        height: exit_line_length,
                    };
                    let psp = (
                        origin_x + player_offset_from_edge,
                        y_pos + exit_line_length / 2.0,
                    );
                    (ep, psp)
                }
            }
        };

        let mut floor_transitions = Vec::new();

        if area_type == AreaType::Bunker {
            // Add a descent point (right edge) if not at the bottom floor
            if floor > -10 {
                // Arbitrary bottom floor limit
                let descent_floor = floor - 1;
                let y_pos = origin_y + rng.random_range(0.0..height - exit_line_length);
                let rect = ExitPoint {
                    x: origin_x + width - exit_line_thickness,
                    y: y_pos,
                    width: exit_line_thickness,
                    height: exit_line_length,
                };
                floor_transitions.push(TransitionPoint {
                    rect,
                    target_floor: descent_floor,
                });
            }

            // Add an ascent point (top edge) if not on the entry floor (floor 1)
            if floor < 1 {
                let ascent_floor = floor + 1;
                let x_pos = origin_x + rng.random_range(0.0..width - exit_line_length);
                let rect = ExitPoint {
                    x: x_pos,
                    y: origin_y,
                    width: exit_line_length,
                    height: exit_line_thickness,
                };
                floor_transitions.push(TransitionPoint {
                    rect,
                    target_floor: ascent_floor,
                });
            }

            // Flag to spawn Night Reavers in bunker
            spawn_night_reavers = true;
        }

        let area_state = AreaState {
            exit_to_world,
            floor_transitions,
            area_type,
            floor,
            spawn_night_reavers,
            waves_active,
            is_peaceful,
        };
        (area_state, player_start_pos)
    }

    /// Checks if the player is within interaction distance of the exit point.
    pub fn is_player_at_world_exit(&self, player_x: f64, player_y: f64) -> bool {
        if self.waves_active {
            return false;
        } // Cannot exit while waves are active
        let exit_check_radius = 50.0;
        let exit_center_x = self.exit_to_world.x + self.exit_to_world.width / 2.0;
        let exit_center_y = self.exit_to_world.y + self.exit_to_world.height / 2.0;
        let dx = player_x - exit_center_x;
        let dy = player_y - exit_center_y;
        (dx * dx + dy * dy).sqrt() < exit_check_radius
    }

    /// Checks if the player is at a floor transition point and returns the target floor if so.
    pub fn get_player_floor_transition(&self, player_x: f64, player_y: f64) -> Option<i32> {
        let exit_check_radius = 50.0;
        for transition in &self.floor_transitions {
            let center_x = transition.rect.x + transition.rect.width / 2.0;
            let center_y = transition.rect.y + transition.rect.height / 2.0;
            let dx = player_x - center_x;
            let dy = player_y - center_y;
            if (dx * dx + dy * dy).sqrt() < exit_check_radius {
                return Some(transition.target_floor);
            }
        }
        None
    }
}
