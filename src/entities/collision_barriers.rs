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
    pub fn check_collision(&self, point_x: f64, point_y: f64, threshold: f64) -> bool {
        let line_dx = self.x2 - self.x1;
        let line_dy = self.y2 - self.y1;
        let line_length_sq = line_dx * line_dx + line_dy * line_dy;

        if line_length_sq == 0.0 {
            let dx = point_x - self.x1;
            let dy = point_y - self.y1;
            return (dx * dx + dy * dy).sqrt() < threshold;
        }

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

/// Zone type for the 3-stage jump system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JumpZoneType {
    Launch,   // '1' - initiates jump towards Air
    Air,      // '2' - mid-air, launches towards Landing
    Landing,  // '3' - touchdown, ends immunity
}

/// A zone for the jump system (launch, air, or landing)
#[derive(Debug, Clone)]
pub struct JumpZone {
    pub zone_type: JumpZoneType,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub target_x: f64,
    pub target_y: f64,
}

impl JumpZone {
    pub fn new(zone_type: JumpZoneType, x: f64, y: f64, width: f64, height: f64) -> Self {
        JumpZone {
            zone_type,
            x,
            y,
            width,
            height,
            target_x: 0.0,
            target_y: 0.0,
        }
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width &&
        py >= self.y && py <= self.y + self.height
    }

    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Chain zone - '0' launches to next '0' in sequence
#[derive(Debug, Clone)]
pub struct ChainZone {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub target_x: f64,
    pub target_y: f64,
    pub is_final: bool,  // True if this is the last '0' in the chain
    pub chain_index: usize,  // Position in the chain sequence
}

impl ChainZone {
    pub fn new(x: f64, y: f64, width: f64, height: f64, chain_index: usize) -> Self {
        ChainZone {
            x,
            y,
            width,
            height,
            target_x: 0.0,
            target_y: 0.0,
            is_final: false,
            chain_index,
        }
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width &&
        py >= self.y && py <= self.y + self.height
    }

    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Result from checking chain zones
#[derive(Debug, Clone)]
pub struct ChainZoneHit {
    pub target_x: f64,
    pub target_y: f64,
    pub is_final: bool,
}

/// Rut zone - reduces player speed by 50%
#[derive(Debug, Clone)]
pub struct RutZone {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl RutZone {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        RutZone { x, y, width, height }
    }

    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width &&
        py >= self.y && py <= self.y + self.height
    }
}

/// Result from checking jump zones
#[derive(Debug, Clone)]
pub struct JumpZoneHit {
    pub zone_type: JumpZoneType,
    pub target_x: f64,
    pub target_y: f64,
    pub _source_x: f64,
    pub _source_y: f64,
}

/// Stores collision barriers for a single field.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FieldCollisionBarriers {
    pub field_id: SbrxFieldId,
    pub lines: Vec<CollisionLine>,
    pub jump_zones: Vec<JumpZone>,
    pub chain_zones: Vec<ChainZone>,
    pub rut_zones: Vec<RutZone>,
    pub boundary_width: f64,
    pub boundary_height: f64,
}

impl FieldCollisionBarriers {
    /// Creates a new empty collision barrier set for a field.
    pub fn new(field_id: SbrxFieldId, width: f64, height: f64) -> Self {
        FieldCollisionBarriers {
            field_id,
            lines: Vec::new(),
            jump_zones: Vec::new(),
            chain_zones: Vec::new(),
            rut_zones: Vec::new(),
            boundary_width: width,
            boundary_height: height,
        }
    }

    /// Parses an ASCII representation of collision barriers.
    /// Characters:
    /// '.' ' ' = free space
    /// '|'     = vertical wall
    /// '_'     = horizontal wall (bottom)
    /// '-'     = horizontal wall (middle)
    /// '/'     = diagonal (bottom-left to top-right)
    /// '\'     = diagonal (top-left to bottom-right)
    /// '['     = left edge vertical
    /// ']'     = right edge vertical
    /// '1'     = launch zone (green) - launches to '2'
    /// '2'     = air zone (gray) - launches to '3'
    /// '3'     = landing zone (blue) - ends immunity
    /// '0'     = chain zone (cyan) - launches to next '0'
    /// 'v'     = rut zone (orange) - reduces speed by 50%
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
        
        let map_lines: Vec<&str> = ascii_map.lines().collect();
        
        // First pass: collect all cells
        let mut launch_cells: Vec<(usize, usize, f64, f64)> = Vec::new();
        let mut air_cells: Vec<(usize, usize, f64, f64)> = Vec::new();
        let mut landing_cells: Vec<(usize, usize, f64, f64)> = Vec::new();
        let mut chain_cells: Vec<(usize, usize, f64, f64)> = Vec::new();
        let mut rut_cells: Vec<(usize, usize, f64, f64)> = Vec::new();
        
        for (row, line) in map_lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let x = origin_x + (col as f64 * cell_width);
                let y = origin_y + (row as f64 * cell_height);
                
                match ch {
                    '|' => {
                        barriers.lines.push(CollisionLine::new(
                            x + cell_width / 2.0,
                            y,
                            x + cell_width / 2.0,
                            y + cell_height,
                        ));
                    }
                    '_' => {
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y + cell_height,
                            x + cell_width,
                            y + cell_height,
                        ));
                    }
                    '-' => {
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y + cell_height / 2.0,
                            x + cell_width,
                            y + cell_height / 2.0,
                        ));
                    }
                    '/' => {
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y + cell_height,
                            x + cell_width,
                            y,
                        ));
                    }
                    '\\' => {
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y,
                            x + cell_width,
                            y + cell_height,
                        ));
                    }
                    '[' => {
                        barriers.lines.push(CollisionLine::new(
                            x,
                            y,
                            x,
                            y + cell_height,
                        ));
                    }
                    ']' => {
                        barriers.lines.push(CollisionLine::new(
                            x + cell_width,
                            y,
                            x + cell_width,
                            y + cell_height,
                        ));
                    }
                    '1' => {
                        launch_cells.push((row, col, x, y));
                    }
                    '2' => {
                        air_cells.push((row, col, x, y));
                    }
                    '3' => {
                        landing_cells.push((row, col, x, y));
                    }
                    '0' => {
                        chain_cells.push((row, col, x, y));
                    }
                    'v' | 'V' => {
                        rut_cells.push((row, col, x, y));
                    }
                    _ => {}
                }
            }
        }
        
        // Merge adjacent cells into zones
        let merged_launches = merge_adjacent_cells(&launch_cells, cell_width, cell_height);
        for (x, y, w, h) in merged_launches {
            barriers.jump_zones.push(JumpZone::new(JumpZoneType::Launch, x, y, w, h));
        }
        
        let merged_air = merge_adjacent_cells(&air_cells, cell_width, cell_height);
        for (x, y, w, h) in merged_air {
            barriers.jump_zones.push(JumpZone::new(JumpZoneType::Air, x, y, w, h));
        }
        
        let merged_landings = merge_adjacent_cells(&landing_cells, cell_width, cell_height);
        for (x, y, w, h) in merged_landings {
            barriers.jump_zones.push(JumpZone::new(JumpZoneType::Landing, x, y, w, h));
        }
        
        // Create chain zones with indices based on ASCII order (top-to-bottom, left-to-right)
        let merged_chains = merge_adjacent_cells(&chain_cells, cell_width, cell_height);
        for (idx, (x, y, w, h)) in merged_chains.iter().enumerate() {
            barriers.chain_zones.push(ChainZone::new(*x, *y, *w, *h, idx));
        }
        
        let merged_ruts = merge_adjacent_cells(&rut_cells, cell_width, cell_height);
        for (x, y, w, h) in merged_ruts {
            barriers.rut_zones.push(RutZone::new(x, y, w, h));
        }
        
        // Link zones: Launch -> Air, Air -> Landing
        barriers.link_jump_zones();
        
        // Link chain zones: each '0' -> next '0'
        barriers.link_chain_zones();
        
        barriers
    }

    /// Links jump zones in sequence: Launch -> nearest Air -> nearest Landing
    fn link_jump_zones(&mut self) {
        let zones_snapshot: Vec<(JumpZoneType, f64, f64)> = self.jump_zones
            .iter()
            .map(|z| (z.zone_type, z.center().0, z.center().1))
            .collect();
        
        for zone in &mut self.jump_zones {
            let zone_center = zone.center();
            
            match zone.zone_type {
                JumpZoneType::Launch => {
                    // Find nearest Air zone
                    let mut nearest_dist = f64::MAX;
                    let mut nearest_target = zone_center;
                    
                    for (zt, cx, cy) in &zones_snapshot {
                        if *zt == JumpZoneType::Air {
                            let dx = cx - zone_center.0;
                            let dy = cy - zone_center.1;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < nearest_dist {
                                nearest_dist = dist;
                                nearest_target = (*cx, *cy);
                            }
                        }
                    }
                    zone.target_x = nearest_target.0;
                    zone.target_y = nearest_target.1; // '3' LANDING
                }
                JumpZoneType::Air => {
                    // Find nearest Landing zone
                    let mut nearest_dist = f64::MAX;
                    let mut nearest_target = zone_center;
                    
                    for (zt, cx, cy) in &zones_snapshot {
                        if *zt == JumpZoneType::Landing {
                            let dx = cx - zone_center.0;
                            let dy = cy - zone_center.1;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < nearest_dist {
                                nearest_dist = dist;
                                nearest_target = (*cx, *cy);
                            }
                        }
                    }
                    zone.target_x = nearest_target.0;
                    zone.target_y = nearest_target.1;
                }
                JumpZoneType::Landing => {
                    // Landing zones don't target anything
                    zone.target_x = zone_center.0;
                    zone.target_y = zone_center.1;
                }
            }
        }
    }

/// Links chain zones in sequence: rightmost '0' first, then nearest, ending at leftmost
    fn link_chain_zones(&mut self) {
        let chain_count = self.chain_zones.len();
        
        if chain_count == 0 {
            return;
        }
        
        if chain_count == 1 {
            let center = self.chain_zones[0].center();
            self.chain_zones[0].target_x = center.0;
            self.chain_zones[0].target_y = center.1;
            self.chain_zones[0].is_final = true;
            return;
        }
        
        // Collect centers for distance calculation
        let centers: Vec<(f64, f64)> = self.chain_zones
            .iter()
            .map(|z| z.center())
            .collect();
        
        // Find the rightmost zone (highest x) to start the chain
        let mut start_idx = 0;
        let mut max_x = f64::MIN;
        for (idx, &(cx, _cy)) in centers.iter().enumerate() {
            if cx > max_x {
                max_x = cx;
                start_idx = idx;
            }
        }
        
        // Track which zones have been used
        let mut used_as_target: Vec<bool> = vec![false; chain_count];
        
        // Build chain starting from rightmost
        let mut current_idx = start_idx;
        let mut chain_order: Vec<usize> = vec![current_idx];
        used_as_target[current_idx] = true;
        
        while chain_order.len() < chain_count {
            let current_center = centers[current_idx];
            let mut nearest_idx: Option<usize> = None;
            let mut nearest_dist = f64::MAX;
            
            // Find nearest zone not yet in chain
            for (idx, &used) in used_as_target.iter().enumerate() {
                if !used {
                    let dx = centers[idx].0 - current_center.0;
                    let dy = centers[idx].1 - current_center.1;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < nearest_dist {
                        nearest_dist = dist;
                        nearest_idx = Some(idx);
                    }
                }
            }
            
            if let Some(next_idx) = nearest_idx {
                chain_order.push(next_idx);
                used_as_target[next_idx] = true;
                current_idx = next_idx;
            } else {
                break;
            }
        }
        
        // Now link zones according to chain_order
        for i in 0..chain_order.len() {
            let zone_idx = chain_order[i];
            self.chain_zones[zone_idx].chain_index = i;  // Update chain_index for rendering
            
            if i + 1 < chain_order.len() {
                // Link to next zone in chain
                let next_zone_idx = chain_order[i + 1];
                self.chain_zones[zone_idx].target_x = centers[next_zone_idx].0;
                self.chain_zones[zone_idx].target_y = centers[next_zone_idx].1;
                self.chain_zones[zone_idx].is_final = false;
            } else {
                // Last zone in chain - mark as final
                self.chain_zones[zone_idx].target_x = centers[zone_idx].0;
                self.chain_zones[zone_idx].target_y = centers[zone_idx].1;
                self.chain_zones[zone_idx].is_final = true;
            }
        }
    }


    /// Checks if a point collides with any barrier in this field.
    pub fn check_point_collision(&self, x: f64, y: f64, threshold: f64) -> Option<(f64, f64)> {
        for line in &self.lines {
            if line.check_collision(x, y, threshold) {
                return Some(((line.x1 + line.x2) / 2.0, (line.y1 + line.y2) / 2.0));
            }
        }
        None
    }

    /// Checks if player is in any jump zone. Returns zone info if so.
    pub fn check_jump_zone(&self, x: f64, y: f64) -> Option<JumpZoneHit> {
        for zone in &self.jump_zones {
            if zone.contains_point(x, y) {
                let center = zone.center();
                return Some(JumpZoneHit {
                    zone_type: zone.zone_type,
                    target_x: zone.target_x,
                    target_y: zone.target_y,
                    _source_x: center.0,
                    _source_y: center.1,
                });
            }
        }
        None
    }

    /// Checks if player is in any chain zone. Returns zone info if so.
    pub fn check_chain_zone(&self, x: f64, y: f64) -> Option<ChainZoneHit> {
        for zone in &self.chain_zones {
            if zone.contains_point(x, y) {
                return Some(ChainZoneHit {
                    target_x: zone.target_x,
                    target_y: zone.target_y,
                    is_final: zone.is_final,
                });
            }
        }
        None
    }

    /// Checks if player is in a rut zone.
    pub fn check_rut_zone(&self, x: f64, y: f64) -> bool {
        for rut in &self.rut_zones {
            if rut.contains_point(x, y) {
                return true;
            }
        }
        false
    }
}

/// Helper to merge adjacent cells into larger rectangles
fn merge_adjacent_cells(
    cells: &[(usize, usize, f64, f64)],
    cell_width: f64,
    cell_height: f64,
) -> Vec<(f64, f64, f64, f64)> {
    if cells.is_empty() {
        return Vec::new();
    }
    
    let mut sorted_cells = cells.to_vec();
    sorted_cells.sort_by(|a, b| {
        if a.0 == b.0 {
            a.1.cmp(&b.1)
        } else {
            a.0.cmp(&b.0)
        }
    });
    
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < sorted_cells.len() {
        let (row, start_col, x, y) = sorted_cells[i];
        let mut end_col = start_col;
        let mut width = cell_width;
        
        // Merge horizontally adjacent cells in same row
        while i + 1 < sorted_cells.len() {
            let (next_row, next_col, _, _) = sorted_cells[i + 1];
            if next_row == row && next_col == end_col + 1 {
                end_col = next_col;
                width += cell_width;
                i += 1;
            } else {
                break;
            }
        }
        
        result.push((x, y, width, cell_height));
        i += 1;
    }
    
    result
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
  . / .vvvvvvvvvvvvvvv  \ . . .  
   |  .vvvvvvvvvvvvvvv  .\. . .  
  .| . \__/2_ \. .    \   \ . .  
  .\. .  .1  \3   .    \.  \. .  
  / -------0-------. . / . 3\ .   
 /    0___2_______0___/| . ..\  
|    /  1  . 3 vvvvvvv\|3|/2 / 
|   /   1______vvvvvvv \  11/ 
|111/ . |   . 22__  / ./ .2./   
| 22\\ . \   3/.  \1 ./ .  1\   
 \|  \\ . \ . ------- . . . 3|.  
  .333\ .  \. . . . . . /_/22|   
  . . . .  /\. . . . . . 1   |  . 
  . . . . / .\______________/  . 
.---------                     .
"#,
            0.0,      // origin_x
            300.0,    // origin_y
            166.0,    // cell_width
            155.0,    // cell_height
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

    /// Checks if player is in a jump zone.
    pub fn check_jump(&self, field_id: &SbrxFieldId, x: f64, y: f64) -> Option<JumpZoneHit> {
        if let Some(barriers) = self.barriers.get(field_id) {
            barriers.check_jump_zone(x, y)
        } else {
            None
        }
    }

    /// Checks if player is in a chain zone.
    pub fn check_chain(&self, field_id: &SbrxFieldId, x: f64, y: f64) -> Option<ChainZoneHit> {
        if let Some(barriers) = self.barriers.get(field_id) {
            barriers.check_chain_zone(x, y)
        } else {
            None
        }
    }

    /// Checks if player is in a rut zone.
    pub fn check_rut(&self, field_id: &SbrxFieldId, x: f64, y: f64) -> bool {
        if let Some(barriers) = self.barriers.get(field_id) {
            barriers.check_rut_zone(x, y)
        } else {
            false
        }
    }
}