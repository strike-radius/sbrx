// file: src/entities/collision_barriers.rs

use crate::map_system::FieldId as SbrxFieldId;
use std::collections::HashMap;

/// A single collision line segment defined by start and end points.
#[derive(Debug, Clone)]
pub struct CollisionLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl CollisionLine {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        CollisionLine { x1, y1, x2, y2 }
    }

    /// Checks if a point collides with this line segment.
    /// Returns true if the point is within `threshold` distance of the line.
    pub fn check_collision(&self, point_x: f64, point_y: f64, threshold: f64) -> bool {
        let line_dx = self.x2 - self.x1;
        let line_dy = self.y2 - self.y1;
        let line_length_sq = line_dx * line_dx + line_dy * line_dy;

        if line_length_sq == 0.0 {
            // Line is a point
            let dx = point_x - self.x1;
            let dy = point_y - self.y1;
            return (dx * dx + dy * dy).sqrt() < threshold;
        }

        // Calculate closest point on line segment
        let t = ((point_x - self.x1) * line_dx + (point_y - self.y1) * line_dy) / line_length_sq;
        let t = t.max(0.0).min(1.0);

        let closest_x = self.x1 + t * line_dx;
        let closest_y = self.y1 + t * line_dy;

        let dx = point_x - closest_x;
        let dy = point_y - closest_y;
        let distance = (dx * dx + dy * dy).sqrt();

        distance < threshold
    }
}

/// Stores collision barriers for a single field.
#[derive(Debug, Clone)]
pub struct FieldCollisionBarriers {
    pub field_id: SbrxFieldId,
    pub lines: Vec<CollisionLine>,
    pub boundary_width: f64,
    pub boundary_height: f64,
}

impl FieldCollisionBarriers {
    /// Creates a new empty collision barrier set for a field.
    pub fn new(field_id: SbrxFieldId, width: f64, height: f64) -> Self {
        FieldCollisionBarriers {
            field_id,
            lines: Vec::new(),
            boundary_width: width,
            boundary_height: height,
        }
    }

    /// Parses an ASCII representation of collision barriers.
    /// Characters: '.' = free space, '|' = vertical, '_' = horizontal, '/' = diagonal up, '\' = diagonal down
    /// 
    /// # Arguments
    /// * `ascii_map` - Multi-line string representing the collision map
    /// * `origin_x` - World X coordinate of the top-left corner
    /// * `origin_y` - World Y coordinate of the top-left corner  
    /// * `cell_width` - Width of each character cell in world units
    /// * `cell_height` - Height of each character cell in world units
    pub fn from_ascii(
        field_id: SbrxFieldId,
        ascii_map: &str,
        origin_x: f64,
        origin_y: f64,
        cell_width: f64,
        cell_height: f64,
        boundary_width: f64,
        boundary_height: f64,
    ) -> Self {
        let mut barriers = FieldCollisionBarriers::new(field_id, boundary_width, boundary_height);
        
        let lines: Vec<&str> = ascii_map.lines().collect();
        
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let x = origin_x + (col as f64 * cell_width);
                let y = origin_y + (row as f64 * cell_height);
                
                match ch {
                    '|' => {
                        // Vertical line - full cell height
                        barriers.lines.push(CollisionLine::new(
                            x + cell_width / 2.0,
                            y,
                            x + cell_width / 2.0,
                            y + cell_height,
                        ));
                    }
                    '_' => {
                        // Horizontal line at bottom of cell
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y + cell_height,
                            x + cell_width,
                            y + cell_height,
                        ));
                    }
                    '-' => {
                        // Horizontal line at middle of cell
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y + cell_height / 2.0,
                            x + cell_width,
                            y + cell_height / 2.0,
                        ));
                    }
                    '/' => {
                        // Diagonal line from bottom-left to top-right
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y + cell_height,
                            x + cell_width,
                            y,
                        ));
                    }
                    '\\' => {
                        // Diagonal line from top-left to bottom-right
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y,
                            x + cell_width,
                            y + cell_height,
                        ));
                    }
                    '[' => {
                        // Left bracket - vertical line on left side
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y,
                            x,
                            y + cell_height,
                        ));
                    }
                    ']' => {
                        // Right bracket - vertical line on right side
                        barriers.lines.push(CollisionLine::new(
                            x + cell_width,
                            y,
                            x + cell_width,
                            y + cell_height,
                        ));
                    }
                    // '.' and ' ' are free space - no collision
                    _ => {}
                }
            }
        }
        
        barriers
    }


    /// Checks if a point collides with any barrier in this field.
    /// Returns Some((collision_x, collision_y)) if collision detected.
    pub fn check_point_collision(&self, x: f64, y: f64, threshold: f64) -> Option<(f64, f64)> {
        for line in &self.lines {
            if line.check_collision(x, y, threshold) {
                // Return the midpoint of the colliding line for knockback direction
                return Some(((line.x1 + line.x2) / 2.0, (line.y1 + line.y2) / 2.0));
            }
        }
        None
    }
}

/// Manages collision barriers for all fields.
pub struct CollisionBarrierManager {
    barriers: HashMap<SbrxFieldId, FieldCollisionBarriers>,
}

impl CollisionBarrierManager {
    pub fn new() -> Self {
        let mut manager = CollisionBarrierManager {
            barriers: HashMap::new(),
        };
        
        // Initialize default barriers for specific fields
        manager.init_default_barriers();
        
        manager
    }

    fn init_default_barriers(&mut self) {
        // Racetrack field (0, 0) with stepped barriers
        let racetrack0_barriers = FieldCollisionBarriers::from_ascii(
            SbrxFieldId(0, 0),
            r#"
                            
. . . . . . . . . . . . . . . .
  .  ___________________. . . .  
  . / . . . . . . . . . \ . . .  
   |  .______________   .\. . .  
  .| . \ . . . . . .  \   \ . .  
  .\. .  . . . . .  .  \.  \. .  
  / ---------------. . / .  \ .   
 /. . ._______________/| . ..\  
| . . / . . . . . . . \| |/  / 
|    /   ____________ .\    / 
| . / . | . . . . . / ./ . ./   
|  .\\ . \ . . . . . ./ .  .\   
 \  .\\ . \ . _______/. . . .\.  
  . . \| . \. . . . . . /_/ . \   
  . . . .  /\. . . . . . . . .| . 
  . . . . / .\_______________/ . 
.________/                     .
"#,
            0.0,      // origin_x - left edge of field
            300.0,    // origin_y - below sky line
            166.0,    // cell_width (5164 / 31 chars approx)
            155.0,    // cell_height (2466 / 16 rows approx)
            5164.0,   // boundary_width
            2466.0,   // boundary_height
        );
        self.barriers.insert(SbrxFieldId(0, 0), racetrack0_barriers);
    }
	
    /// Gets collision barriers for a specific field.
    pub fn get_barriers(&self, field_id: &SbrxFieldId) -> Option<&FieldCollisionBarriers> {
        self.barriers.get(field_id)
    }	

    /// Checks for collision at a point in a specific field.
    /// Returns Some((line_center_x, line_center_y)) if collision detected.
    pub fn check_collision(
        &self,
        field_id: &SbrxFieldId,
        x: f64,
        y: f64,
        threshold: f64,
    ) -> Option<(f64, f64)> {
        if let Some(barriers) = self.barriers.get(field_id) {
            barriers.check_point_collision(x, y, threshold)
        } else {
            None
        }
    }
}