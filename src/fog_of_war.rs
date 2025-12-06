// File: src/fog_of_war.rs

use crate::map_system::FieldId as SbrxFieldId;
use piston_window::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct FogTile {
    pub explored: bool,
    pub last_visit_time: f64,
}

impl Default for FogTile {
    fn default() -> Self {
        FogTile {
            explored: false,
            last_visit_time: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FogGrid {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<FogTile>>,
    pub tile_size: f64,
    pub world_bounds: (f64, f64, f64, f64), // min_x, max_x, min_y, max_y
}

impl FogGrid {
    pub fn new(world_bounds: (f64, f64, f64, f64), tile_size: f64) -> Self {
        let width = ((world_bounds.1 - world_bounds.0) / tile_size).ceil() as usize;
        let height = ((world_bounds.3 - world_bounds.2) / tile_size).ceil() as usize;

        let tiles = vec![vec![FogTile::default(); width]; height];

        FogGrid {
            width,
            height,
            tiles,
            tile_size,
            world_bounds,
        }
    }

    pub fn world_to_grid(&self, world_x: f64, world_y: f64) -> Option<(usize, usize)> {
        let grid_x = ((world_x - self.world_bounds.0) / self.tile_size).floor() as i32;
        let grid_y = ((world_y - self.world_bounds.2) / self.tile_size).floor() as i32;

        if grid_x >= 0
            && grid_y >= 0
            && (grid_x as usize) < self.width
            && (grid_y as usize) < self.height
        {
            Some((grid_x as usize, grid_y as usize))
        } else {
            None
        }
    }

    pub fn grid_to_world(&self, grid_x: usize, grid_y: usize) -> (f64, f64) {
        let world_x =
            self.world_bounds.0 + (grid_x as f64 * self.tile_size) + (self.tile_size / 2.0);
        let world_y =
            self.world_bounds.2 + (grid_y as f64 * self.tile_size) + (self.tile_size / 2.0);
        (world_x, world_y)
    }

    pub fn is_explored(&self, world_x: f64, world_y: f64) -> bool {
        if let Some((gx, gy)) = self.world_to_grid(world_x, world_y) {
            self.tiles[gy][gx].explored
        } else {
            false
        }
    }

    pub fn explore_area(&mut self, center_x: f64, center_y: f64, radius: f64, game_time: f64) {
        let grid_radius = (radius / self.tile_size).ceil() as i32;

        if let Some((center_gx, center_gy)) = self.world_to_grid(center_x, center_y) {
            for dy in -grid_radius..=grid_radius {
                for dx in -grid_radius..=grid_radius {
                    let gx = center_gx as i32 + dx;
                    let gy = center_gy as i32 + dy;

                    if gx >= 0
                        && gy >= 0
                        && (gx as usize) < self.width
                        && (gy as usize) < self.height
                    {
                        let (tile_world_x, tile_world_y) =
                            self.grid_to_world(gx as usize, gy as usize);
                        let distance = ((tile_world_x - center_x).powi(2)
                            + (tile_world_y - center_y).powi(2))
                        .sqrt();

                        if distance <= radius {
                            let tile = &mut self.tiles[gy as usize][gx as usize];
                            tile.explored = true;
                            tile.last_visit_time = game_time;
                        }
                    }
                }
            }
        }
    }
}

pub struct FogOfWar {
    pub enabled_fields: HashSet<SbrxFieldId>,

    pub grids: HashMap<SbrxFieldId, FogGrid>,
    pub visibility_radius: f64,
    pub tile_size: f64,
    pub world_bounds: (f64, f64, f64, f64), // Standard world bounds for all fields
    pub fog_color: [f32; 4],
    pub debug_mode: bool,
}

impl FogOfWar {
    pub fn new() -> Self {
        let mut enabled_fields = HashSet::new();
        // Enable fog ONLY for the rocketbay field.
        enabled_fields.insert(SbrxFieldId(-2, 5));

        FogOfWar {
            enabled_fields,
            grids: HashMap::new(),
            visibility_radius: 200.0, // Player can see 200 units around them
            tile_size: 100.0,         // Each fog tile is 100x100 units
            world_bounds: (0.0, 5000.0, 540.0, 3250.0), // Standard game bounds
            fog_color: [0.0, 0.0, 0.0, 0.9], // Semi-transparent black
            debug_mode: false,
        }
    }

    /// Gets or creates a new grid for a specific field.
    fn get_or_create_grid_mut(&mut self, field_id: SbrxFieldId) -> &mut FogGrid {
        let bounds = self.world_bounds;
        let tile_size = self.tile_size;
        self.grids
            .entry(field_id)
            .or_insert_with(|| FogGrid::new(bounds, tile_size))
    }

    pub fn is_fog_enabled(&self, field_id: SbrxFieldId) -> bool {
        self.enabled_fields.contains(&field_id)
    }

    pub fn update_player_visibility(
        &mut self,
        field_id: SbrxFieldId,
        player_x: f64,
        player_y: f64,
        game_time: f64,
    ) {
        if !self.is_fog_enabled(field_id) {
            return;
        }

        let visibility_radius = self.visibility_radius;
        let grid = self.get_or_create_grid_mut(field_id);

        grid.explore_area(player_x, player_y, visibility_radius, game_time);
    }

    pub fn is_position_visible(&self, field_id: SbrxFieldId, world_x: f64, world_y: f64) -> bool {
        if !self.is_fog_enabled(field_id) {
            return true; // No fog, everything is visible
        }

        if let Some(grid) = self.grids.get(&field_id) {
            grid.is_explored(world_x, world_y)
        } else {
            false // A field with fog enabled that hasn't been visited is completely dark
        }
    }

    pub fn should_render_entity(
        &self,
        field_id: SbrxFieldId,
        entity_x: f64,
        entity_y: f64,
    ) -> bool {
        self.is_position_visible(field_id, entity_x, entity_y)
    }

    pub fn render_fog_overlay(&mut self, field_id: SbrxFieldId, context: Context, g: &mut G2d) {
        if !self.is_fog_enabled(field_id) {
            return;
        }

        // ** FIX: Read values before the mutable borrow **
        let fog_color = self.fog_color;
        let debug_mode = self.debug_mode;

        let grid = self.get_or_create_grid_mut(field_id);

        for y in 0..grid.height {
            for x in 0..grid.width {
                if !grid.tiles[y][x].explored {
                    let (world_x, world_y) = grid.grid_to_world(x, y);
                    let tile_x = world_x - grid.tile_size / 2.0;
                    let tile_y = world_y - grid.tile_size / 2.0;

                    rectangle(
                        fog_color, // Use the local variable
                        [tile_x, tile_y, grid.tile_size, grid.tile_size],
                        context.transform,
                        g,
                    );
                }
            }
        }

        // Debug mode: show grid lines
        if debug_mode {
            // Use the local variable
            for y in 0..=grid.height {
                let world_y = grid.world_bounds.2 + (y as f64 * grid.tile_size);
                line(
                    [1.0, 0.0, 0.0, 0.3], // Red debug lines
                    1.0,
                    [grid.world_bounds.0, world_y, grid.world_bounds.1, world_y],
                    context.transform,
                    g,
                );
            }
            for x in 0..=grid.width {
                let world_x = grid.world_bounds.0 + (x as f64 * grid.tile_size);
                line(
                    [1.0, 0.0, 0.0, 0.3],
                    1.0,
                    [world_x, grid.world_bounds.2, world_x, grid.world_bounds.3],
                    context.transform,
                    g,
                );
            }
        }
    }

    pub fn toggle_debug_mode(&mut self) {
        self.debug_mode = !self.debug_mode;
        println!(
            "Fog of War debug mode: {}",
            if self.debug_mode { "ON" } else { "OFF" }
        );
    }

    pub fn get_exploration_stats(&self, field_id: SbrxFieldId) -> (usize, usize, f32) {
        if let Some(grid) = self.grids.get(&field_id) {
            let total_tiles = grid.width * grid.height;
            let explored_tiles = grid
                .tiles
                .iter()
                .flatten()
                .filter(|tile| tile.explored)
                .count();
            let exploration_percentage = if total_tiles > 0 {
                (explored_tiles as f32 / total_tiles as f32) * 10.0
            } else {
                0.0
            };
            (explored_tiles, total_tiles, exploration_percentage)
        } else {
            let width =
                ((self.world_bounds.1 - self.world_bounds.0) / self.tile_size).ceil() as usize;
            let height =
                ((self.world_bounds.3 - self.world_bounds.2) / self.tile_size).ceil() as usize;
            (0, width * height, 0.0)
        }
    }
}
